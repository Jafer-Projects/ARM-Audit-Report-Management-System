import { createHash, randomBytes } from "node:crypto";
import type { FastifyRequest } from "fastify";
import type { PoolClient } from "pg";
import { config } from "./config.js";
import { pool, inTransaction } from "./db.js";
import { hashPassword, signAccessToken, verifyAccessToken, verifyPassword, type AccessClaims } from "./crypto.js";

const REFRESH_TTL_MS = 30 * 24 * 60 * 60 * 1000;
const digest = (value: string) => createHash("sha256").update(value).digest("hex");

export type AuthenticatedRequest = FastifyRequest & { auth?: AccessClaims };

export async function authenticate(request: FastifyRequest): Promise<AccessClaims> {
  const header = request.headers.authorization;
  if (!header?.startsWith("Bearer ")) throw Object.assign(new Error("Authentication required"), { statusCode: 401 });
  let claims: AccessClaims;
  try {
    claims = verifyAccessToken(header.slice(7));
  } catch {
    throw Object.assign(new Error("Invalid or expired access token"), { statusCode: 401 });
  }
  const session = await pool.query(
    "SELECT 1 FROM auth_sessions s JOIN users u ON u.id = s.user_id WHERE s.id = $1 AND s.user_id = $2 AND s.revoked_at IS NULL AND s.expires_at > now() AND u.status = 'active'",
    [claims.sid, claims.sub]
  );
  if (session.rowCount !== 1) throw Object.assign(new Error("Session is no longer active"), { statusCode: 401 });
  return claims;
}

async function membershipsFor(client: PoolClient, userId: string) {
  await client.query("SELECT set_config('app.user_id', $1, true)", [userId]);
  const result = await client.query(
    "SELECT m.tenant_id, t.name AS tenant_name, m.role FROM tenant_memberships m JOIN tenants t ON t.id = m.tenant_id WHERE m.user_id = $1 AND m.status = 'active' AND t.status = 'active' ORDER BY t.name",
    [userId]
  );
  return result.rows;
}

export async function register(input: { email: string; displayName: string; password: string; organizationName: string }) {
  const passwordHash = await hashPassword(input.password);
  return inTransaction(async (client) => {
    const user = await client.query(
      "INSERT INTO users(email, display_name, password_hash) VALUES ($1, $2, $3) RETURNING id, email, display_name",
      [input.email.trim(), input.displayName.trim(), passwordHash]
    );
    const userId = user.rows[0].id as string;
    await client.query("SELECT set_config('app.user_id', $1, true)", [userId]);
    const slug = `${input.organizationName.toLowerCase().normalize("NFKD").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 42) || "firm"}-${randomBytes(3).toString("hex")}`;
    const tenant = await client.query(
      "INSERT INTO tenants(name, slug, created_by) VALUES ($1, $2, $3) RETURNING id, name",
      [input.organizationName.trim(), slug, userId]
    );
    const tenantId = tenant.rows[0].id as string;
    await client.query("SELECT set_config('app.tenant_id', $1, true)", [tenantId]);
    await client.query(
      "INSERT INTO tenant_memberships(tenant_id, user_id, role, status) VALUES ($1, $2, 'owner', 'active')",
      [tenantId, userId]
    );
    const sessionId = randomBytes(16).toString("hex");
    const refreshToken = randomBytes(48).toString("base64url");
    await client.query(
      "INSERT INTO auth_sessions(id, user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3, $4)",
      [sessionId, userId, digest(refreshToken), new Date(Date.now() + REFRESH_TTL_MS)]
    );
    return {
      user: { id: userId, email: user.rows[0].email, displayName: user.rows[0].display_name },
      tenant: { id: tenantId, name: tenant.rows[0].name, role: "owner" },
      accessToken: signAccessToken(userId, sessionId),
      refreshToken,
      expiresIn: config.ACCESS_TOKEN_TTL_SECONDS
    };
  });
}

export async function login(email: string, password: string) {
  const result = await pool.query(
    "SELECT id, email, display_name, password_hash, status FROM users WHERE email_normalized = lower(btrim($1))",
    [email]
  );
  const user = result.rows[0];
  if (!user || user.status !== "active" || !(await verifyPassword(password, user.password_hash as string))) {
    throw Object.assign(new Error("Email or password is incorrect"), { statusCode: 401 });
  }
  return inTransaction(async (client) => {
    const sessionId = randomBytes(16).toString("hex");
    const refreshToken = randomBytes(48).toString("base64url");
    await client.query(
      "INSERT INTO auth_sessions(id, user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3, $4)",
      [sessionId, user.id, digest(refreshToken), new Date(Date.now() + REFRESH_TTL_MS)]
    );
    const memberships = await membershipsFor(client, user.id as string);
    return {
      user: { id: user.id, email: user.email, displayName: user.display_name },
      memberships,
      accessToken: signAccessToken(user.id as string, sessionId),
      refreshToken,
      expiresIn: config.ACCESS_TOKEN_TTL_SECONDS
    };
  });
}

export async function rotateRefreshToken(refreshToken: string) {
  return inTransaction(async (client) => {
    const result = await client.query(
      "SELECT s.id, s.user_id, u.email, u.display_name FROM auth_sessions s JOIN users u ON u.id = s.user_id WHERE s.refresh_token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > now() AND u.status = 'active' FOR UPDATE",
      [digest(refreshToken)]
    );
    const session = result.rows[0];
    if (!session) throw Object.assign(new Error("Refresh token is invalid or expired"), { statusCode: 401 });
    const nextRefresh = randomBytes(48).toString("base64url");
    await client.query(
      "UPDATE auth_sessions SET refresh_token_hash = $1, rotated_at = now() WHERE id = $2",
      [digest(nextRefresh), session.id]
    );
    return {
      accessToken: signAccessToken(session.user_id as string, session.id as string),
      refreshToken: nextRefresh,
      expiresIn: config.ACCESS_TOKEN_TTL_SECONDS
    };
  });
}

export async function revokeSession(sessionId: string, userId: string) {
  await pool.query(
    "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
    [sessionId, userId]
  );
}
