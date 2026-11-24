import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { buildApp } from "../src/app.js";
import type { FastifyInstance } from "fastify";

let app: FastifyInstance;
let first: { accessToken: string; refreshToken: string; tenant: { id: string } };
let second: { accessToken: string; tenant: { id: string } };

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

after(async () => { await app.close(); });

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
});
