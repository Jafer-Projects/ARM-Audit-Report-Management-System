import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { buildApp } from "../src/app.js";
import type { FastifyInstance } from "fastify";
import pg from "pg";

let app: FastifyInstance;
const workerPool = new pg.Pool({ connectionString: process.env.WORKER_DATABASE_URL });
let first: { accessToken: string; refreshToken: string; user: { id: string }; tenant: { id: string } };
let second: { accessToken: string; user: { id: string }; tenant: { id: string } };

before(async () => {
  app = await buildApp();
  await app.ready();
  const suffix = randomUUID();
  const one = await app.inject({
    method: "POST", url: "/v1/auth/register",
    payload: { email: `owner-${suffix}@example.test`, displayName: "First Owner", password: "correct-horse-battery-staple-1", organizationName: "First Firm" }
  });
  assert.equal(one.statusCode, 201, one.body);
  first = one.json();
  const two = await app.inject({
    method: "POST", url: "/v1/auth/register",
    payload: { email: `other-${suffix}@example.test`, displayName: "Second Owner", password: "correct-horse-battery-staple-2", organizationName: "Second Firm" }
  });
  assert.equal(two.statusCode, 201, two.body);
  second = two.json();
});

after(async () => { await app.close(); await workerPool.end(); });

test("API database role is non-superuser and PostgreSQL RLS independently enforces membership", async () => {
  const db = await import("../src/db.js");
  const role = await db.pool.query("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user");
  assert.equal(role.rows[0].rolsuper, false);
  assert.equal(role.rows[0].rolbypassrls, false);
  const client = await db.pool.connect();
  try {
    await client.query("BEGIN");
    await client.query("SELECT set_config('app.user_id', $1, true)", [second.user.id]);
    await client.query("SELECT set_config('app.tenant_id', $1, true)", [first.tenant.id]);
    const hidden = await client.query("SELECT count(*)::int AS count FROM sync_aggregates");
    assert.equal(hidden.rows[0].count, 0);
    await client.query("ROLLBACK");
  } finally { client.release(); }
});

test("worker database role can access its queue but cannot read tenant data", async () => {
  const role = await workerPool.query("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user");
  assert.equal(role.rows[0].rolsuper, false);
  assert.equal(role.rows[0].rolbypassrls, false);
  await workerPool.query("BEGIN");
  try {
    await workerPool.query("SELECT set_config('app.worker', 'true', true)");
    await workerPool.query("SELECT id FROM jobs LIMIT 1");
    await assert.rejects(workerPool.query("SELECT id FROM tenants"), (error) => error.code === "42501");
    await workerPool.query("ROLLBACK");
  } catch (error) {
    await workerPool.query("ROLLBACK");
    throw error;
  }
});

test("registration creates an owner-scoped tenant and login returns only own memberships", async () => {
  assert.ok(first.tenant.id);
  const context = await app.inject({
    method: "GET", url: "/v1/tenant/context",
    headers: { authorization: `Bearer ${first.accessToken}`, "x-tenant-id": first.tenant.id }
  });
  assert.equal(context.statusCode, 200, context.body);
  assert.equal(context.json().role, "owner");
});

test("sync push is version-checked, idempotent, and pull returns accepted events", async () => {
  const aggregateId = randomUUID();
  const event = {
    eventId: randomUUID(), idempotencyKey: randomUUID(), aggregateType: "workspace.client",
    aggregateId, baseVersion: 0, eventType: "aggregate.upsert", schemaVersion: 1,
    occurredAt: new Date().toISOString(), payload: { displayName: "Synthetic Client" }
  };
  const headers = { authorization: `Bearer ${first.accessToken}`, "x-tenant-id": first.tenant.id };
  const accepted = await app.inject({ method: "POST", url: "/v1/sync/events", headers, payload: { events: [event] } });
  assert.equal(accepted.statusCode, 200, accepted.body);
  assert.equal(accepted.json().results[0].status, "accepted");
  assert.equal(accepted.json().results[0].version, 1);

  const duplicate = await app.inject({ method: "POST", url: "/v1/sync/events", headers, payload: { events: [event] } });
  assert.equal(duplicate.statusCode, 200, duplicate.body);
  assert.equal(duplicate.json().results[0].duplicate, true);

  const conflict = await app.inject({
    method: "POST", url: "/v1/sync/events", headers,
    payload: { events: [{ ...event, eventId: randomUUID(), idempotencyKey: randomUUID(), payload: { displayName: "Stale Client" } }] }
  });
  assert.equal(conflict.statusCode, 200, conflict.body);
  assert.equal(conflict.json().results[0].status, "conflict");

  const pulled = await app.inject({ method: "GET", url: "/v1/sync/events", headers });
  assert.equal(pulled.statusCode, 200, pulled.body);
  assert.equal(pulled.json().events.length, 1);
});

test("tenant identifiers supplied by another user do not grant access", async () => {
  const response = await app.inject({
    method: "GET", url: "/v1/tenant/context",
    headers: { authorization: `Bearer ${second.accessToken}`, "x-tenant-id": first.tenant.id }
  });
  assert.equal(response.statusCode, 403);
});

test("refresh tokens rotate and logout revokes the access session", async () => {
  const refreshed = await app.inject({ method: "POST", url: "/v1/auth/refresh", payload: { refreshToken: first.refreshToken } });
  assert.equal(refreshed.statusCode, 200, refreshed.body);
  const oldToken = await app.inject({ method: "POST", url: "/v1/auth/refresh", payload: { refreshToken: first.refreshToken } });
  assert.equal(oldToken.statusCode, 401);
  const newRefresh = refreshed.json().refreshToken as string;
  const logout = await app.inject({
    method: "POST", url: "/v1/auth/logout",
    headers: { authorization: `Bearer ${refreshed.json().accessToken}` }
  });
  assert.equal(logout.statusCode, 204);
  const revoked = await app.inject({
    method: "GET", url: "/v1/tenant/context",
    headers: { authorization: `Bearer ${refreshed.json().accessToken}`, "x-tenant-id": first.tenant.id }
  });
  assert.equal(revoked.statusCode, 401);
  const rotatedAgain = await app.inject({ method: "POST", url: "/v1/auth/refresh", payload: { refreshToken: newRefresh } });
  assert.equal(rotatedAgain.statusCode, 401);
});
