import { createHash, randomBytes, randomUUID } from "node:crypto";
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
  const active = await inTransaction(async (client) => {
    await client.query("SELECT set_config('app.user_id', $1, true)", [claims.sub]);
    const session = await client.query(
      "SELECT 1 FROM auth_sessions s JOIN users u ON u.id = s.user_id WHERE s.id = $1 AND s.user_id = $2 AND s.revoked_at IS NULL AND s.expires_at > now() AND u.status = 'active'",
      [claims.sid, claims.sub]
    );
    return session.rowCount === 1;
  });
  if (!active) throw Object.assign(new Error("Session is no longer active"), { statusCode: 401 });
  return claims;
}

async function membershipsFor(client: PoolClient, userId: string) {
  await client.query("SELECT set_config('app.user_id', $1, true)", [userId]);
  const result = await client.query(
    "SELECT tenant_id, role FROM tenant_memberships WHERE user_id = $1 AND status = 'active' ORDER BY tenant_id",
    [userId]
  );
  return result.rows;
}

export async function register(input: { email: string; displayName: string; password: string; organizationName: string }) {
  const passwordHash = await hashPassword(input.password);
  return inTransaction(async (client) => {
    const userId = randomUUID();
    await client.query("SELECT set_config('app.user_id', $1, true)", [userId]);
    const user = await client.query(
      "INSERT INTO users(id, email, display_name, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, email, display_name",
      [userId, input.email.trim(), input.displayName.trim(), passwordHash]
    );
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
    const sessionId = randomUUID();
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
    "SELECT user_id AS id, email, display_name, password_hash, status FROM public.arm_lookup_user_for_login($1)",
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
    const lookup = await client.query(
      "SELECT session_id, user_id FROM public.arm_lookup_session_for_refresh($1)",
      [digest(refreshToken)]
    );
    const found = lookup.rows[0];
    if (!found) throw Object.assign(new Error("Refresh token is invalid or expired"), { statusCode: 401 });
    await client.query("SELECT set_config('app.user_id', $1, true)", [found.user_id]);
    const locked = await client.query(
      "SELECT id, user_id FROM auth_sessions WHERE id = $1 AND user_id = $2 AND refresh_token_hash = $3 AND revoked_at IS NULL AND expires_at > now() FOR UPDATE",
      [found.session_id, found.user_id, digest(refreshToken)]
    );
    const session = locked.rows[0];
    if (!session) throw Object.assign(new Error("Refresh token has already been rotated"), { statusCode: 401 });
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
  await inTransaction(async (client) => {
    await client.query("SELECT set_config('app.user_id', $1, true)", [userId]);
    await client.query(
      "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
      [sessionId, userId]
    );
  });
}
