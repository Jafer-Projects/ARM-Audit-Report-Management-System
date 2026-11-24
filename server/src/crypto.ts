import { createHmac, randomBytes, scrypt as scryptCallback, timingSafeEqual } from "node:crypto";
import { promisify } from "node:util";
import { config } from "./config.js";

const scrypt = promisify(scryptCallback);
const KEY_LENGTH = 64;

export async function hashPassword(password: string): Promise<string> {
  const salt = randomBytes(16);
  const derived = (await scrypt(password, salt, KEY_LENGTH, { N: 1 << 15, r: 8, p: 1, maxmem: 64 * 1024 * 1024 })) as Buffer;
  return `scrypt$${salt.toString("base64url")}$${derived.toString("base64url")}`;
}

export async function verifyPassword(password: string, encoded: string): Promise<boolean> {
  const [algorithm, saltText, hashText] = encoded.split("$");
  if (algorithm !== "scrypt" || !saltText || !hashText) return false;
  const expected = Buffer.from(hashText, "base64url");
  const actual = (await scrypt(password, Buffer.from(saltText, "base64url"), expected.length, {
    N: 1 << 15, r: 8, p: 1, maxmem: 64 * 1024 * 1024
  })) as Buffer;
  return actual.length === expected.length && timingSafeEqual(actual, expected);
}

export type AccessClaims = { sub: string; sid: string; iat: number; exp: number; iss: "arm-cloud" };

export function signAccessToken(userId: string, sessionId: string): string {
  const now = Math.floor(Date.now() / 1000);
  const header = Buffer.from(JSON.stringify({ alg: "HS256", typ: "JWT" })).toString("base64url");
  const payload = Buffer.from(JSON.stringify({
    sub: userId, sid: sessionId, iat: now,
    exp: now + config.ACCESS_TOKEN_TTL_SECONDS, iss: "arm-cloud"
  })).toString("base64url");
  const unsigned = `${header}.${payload}`;
  const signature = createHmac("sha256", config.JWT_SECRET).update(unsigned).digest("base64url");
  return `${unsigned}.${signature}`;
}

export function verifyAccessToken(token: string): AccessClaims {
  const parts = token.split(".");
  if (parts.length !== 3 || !parts[0] || !parts[1] || !parts[2]) throw new Error("Invalid access token");
  const unsigned = `${parts[0]}.${parts[1]}`;
  const expected = createHmac("sha256", config.JWT_SECRET).update(unsigned).digest();
  const actual = Buffer.from(parts[2], "base64url");
  if (actual.length !== expected.length || !timingSafeEqual(actual, expected)) throw new Error("Invalid access token");
  const header = JSON.parse(Buffer.from(parts[0], "base64url").toString("utf8")) as { alg?: string };
  const claims = JSON.parse(Buffer.from(parts[1], "base64url").toString("utf8")) as AccessClaims;
  if (header.alg !== "HS256" || claims.iss !== "arm-cloud" || typeof claims.sub !== "string" || typeof claims.sid !== "string" || typeof claims.iat !== "number" || typeof claims.exp !== "number" || claims.exp <= Math.floor(Date.now() / 1000) || claims.iat > Math.floor(Date.now() / 1000) + 60) {
    throw new Error("Expired or invalid access token");
  }
  return claims;
}
