import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { buildApp } from "../src/app.js";
import type { FastifyInstance } from "fastify";
import pg from "pg";
import { pool } from "../src/db.js";

let app: FastifyInstance;
const workerPool = new pg.Pool({ connectionString: process.env.WORKER_DATABASE_URL });
const firstPassword = "correct-horse-battery-staple-1";
let firstEmail = "";
let first: { accessToken: string; refreshToken: string; user: { id: string }; tenant: { id: string } };
let second: { accessToken: string; user: { id: string }; tenant: { id: string } };

before(async () => {
  app = await buildApp();
  await app.ready();
  const suffix = randomUUID();
  firstEmail = `owner-${suffix}@example.test`;
  const one = await app.inject({
    method: "POST", url: "/v1/auth/register",
    payload: { email: firstEmail, displayName: "First Owner", password: firstPassword, organizationName: "First Firm" }
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

test("API role is non-superuser and tenant-scoped tables are protected by RLS", async () => {
  const role = await pool.query("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user");
  assert.equal(role.rows[0].rolsuper, false);
  assert.equal(role.rows[0].rolbypassrls, false);
  await assert.rejects(pool.query("SELECT version FROM schema_migrations"), (error) => error.code === "42501");
  await assert.rejects(pool.query("SELECT id FROM jobs"), (error) => error.code === "42501");
});

test("worker role is non-superuser and cannot read tenant identity tables", async () => {
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

test("registration creates an owner-scoped tenant and tenant context resolves the role", async () => {
  assert.ok(first.tenant.id);
  const context = await app.inject({
    method: "GET", url: "/v1/tenant/context",
    headers: { authorization: `Bearer ${first.accessToken}`, "x-tenant-id": first.tenant.id }
  });
  assert.equal(context.statusCode, 200, context.body);
  assert.equal(context.json().role, "owner");
});

test("login verifies credentials and returns only the caller's active memberships", async () => {
  const response = await app.inject({
    method: "POST", url: "/v1/auth/login",
    payload: { email: firstEmail, password: firstPassword }
  });
  assert.equal(response.statusCode, 200, response.body);
  assert.equal(response.json().memberships.length, 1);
  assert.equal(response.json().memberships[0].tenant_id, first.tenant.id);
  const rejected = await app.inject({
    method: "POST", url: "/v1/auth/login",
    payload: { email: firstEmail, password: "incorrect-password" }
  });
  assert.equal(rejected.statusCode, 401);
});

test("sync push is version-checked, idempotent, and pull returns accepted events", async () => {
  const aggregateId = randomUUID();
  const event = {
    eventId: randomUUID(), idempotencyKey: randomUUID(), aggregateType: "prototype.workspace_record",
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
  assert.equal(pulled.json().events.length, 2);
  assert.deepEqual(pulled.json().events.map((item) => item.outcome), ["accepted", "conflict"]);
  assert.ok(pulled.json().nextCursor);
  const drained = await app.inject({ method: "GET", url: `/v1/sync/events?after=${pulled.json().nextCursor}`, headers });
  assert.equal(drained.statusCode, 200);
  assert.equal(drained.json().events.length, 0);

  const unsupportedDomain = await app.inject({
    method: "POST", url: "/v1/sync/events", headers,
    payload: { events: [{ ...event, eventId: randomUUID(), idempotencyKey: randomUUID(), aggregateType: "accounting.posted_journal" }] }
  });
  assert.equal(unsupportedDomain.statusCode, 400);
});

test("PostgreSQL RLS hides another tenant even when the tenant context is supplied directly", async () => {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");
    await client.query("SELECT set_config('app.user_id', $1, true)", [second.user.id]);
    await client.query("SELECT set_config('app.tenant_id', $1, true)", [first.tenant.id]);
    const hidden = await client.query("SELECT count(*)::int AS count FROM sync_aggregates");
    assert.equal(hidden.rows[0].count, 0);
    await client.query("ROLLBACK");
  } finally { client.release(); }
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
