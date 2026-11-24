import { z } from "zod";
import type { PoolClient } from "pg";
import { withTenant } from "./db.js";

const syncEventSchema = z.object({
  eventId: z.string().uuid(),
  idempotencyKey: z.string().min(8).max(160),
  aggregateType: z.string().min(1).max(80).regex(/^[a-z][a-z0-9_.-]*$/),
  aggregateId: z.string().uuid(),
  baseVersion: z.number().int().min(0),
  eventType: z.literal("aggregate.upsert"),
  schemaVersion: z.number().int().positive(),
  occurredAt: z.string().datetime({ offset: true }),
  correlationId: z.string().max(160).optional(),
  payload: z.record(z.string(), z.unknown())
});
export const pushSchema = z.object({ events: z.array(syncEventSchema).min(1).max(100) });

type SyncEvent = z.infer<typeof syncEventSchema>;

async function applyEvent(client: PoolClient, userId: string, tenantId: string, event: SyncEvent) {
  const duplicate = await client.query(
    "SELECT event_id, idempotency_key, outcome, resulting_version, outcome_detail FROM sync_inbox WHERE tenant_id = $1 AND (idempotency_key = $2 OR event_id = $3)",
    [tenantId, event.idempotencyKey, event.eventId]
  );
  if (duplicate.rowCount) {
    const row = duplicate.rows[0];
    if (row.idempotency_key !== event.idempotencyKey || row.event_id !== event.eventId) {
      return { eventId: event.eventId, status: "rejected", version: null, detail: { reason: "event_or_idempotency_key_reused" }, duplicate: false };
    }
    return { eventId: event.eventId, status: row.outcome, version: row.resulting_version, detail: row.outcome_detail, duplicate: true };
  }

  await client.query(
    "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
    [`${tenantId}:${event.aggregateType}:${event.aggregateId}`]
  );
  const current = await client.query(
    "SELECT version, payload FROM sync_aggregates WHERE tenant_id = $1 AND aggregate_type = $2 AND aggregate_id = $3 FOR UPDATE",
    [tenantId, event.aggregateType, event.aggregateId]
  );
  const currentVersion = Number(current.rows[0]?.version ?? 0);
  let outcome: "accepted" | "conflict" = "accepted";
  let resultingVersion: number | null = null;
  let detail: Record<string, unknown> = {};

  if (currentVersion !== event.baseVersion) {
    outcome = "conflict";
    detail = { reason: "base_version_mismatch", expectedVersion: currentVersion, suppliedBaseVersion: event.baseVersion };
  } else {
    resultingVersion = currentVersion + 1;
    await client.query(
      `INSERT INTO sync_aggregates(tenant_id, aggregate_type, aggregate_id, version, payload, updated_by)
       VALUES ($1, $2, $3, $4, $5::jsonb, $6)
       ON CONFLICT (tenant_id, aggregate_type, aggregate_id)
       DO UPDATE SET version = EXCLUDED.version, payload = EXCLUDED.payload, updated_by = EXCLUDED.updated_by, updated_at = now()`,
      [tenantId, event.aggregateType, event.aggregateId, resultingVersion, JSON.stringify(event.payload), userId]
    );
    detail = { acceptedAtServer: new Date().toISOString() };
    await client.query(
      "INSERT INTO audit_events(tenant_id, actor_id, action, target_type, target_id, correlation_id, metadata) VALUES ($1, $2, 'sync.aggregate.upsert', $3, $4, $5, $6::jsonb)",
      [tenantId, userId, event.aggregateType, event.aggregateId, event.correlationId ?? null, JSON.stringify({ eventId: event.eventId, version: resultingVersion })]
    );
  }

  await client.query(
    `INSERT INTO sync_inbox(tenant_id, actor_id, event_id, idempotency_key, aggregate_type, aggregate_id, base_version, resulting_version, event_type, schema_version, payload, outcome, outcome_detail, occurred_at_device)
     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::jsonb,$12,$13::jsonb,$14)`,
    [tenantId, userId, event.eventId, event.idempotencyKey, event.aggregateType, event.aggregateId, event.baseVersion,
      resultingVersion, event.eventType, event.schemaVersion, JSON.stringify(event.payload), outcome, JSON.stringify(detail), event.occurredAt]
  );
  return { eventId: event.eventId, status: outcome, version: resultingVersion, detail, duplicate: false };
}

export async function pushEvents(userId: string, tenantId: string, events: SyncEvent[]) {
  return withTenant(userId, tenantId, async (client) => {
    await client.query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))", ["sync-tenant:" + tenantId]);
    const results = [];
    for (const event of events) results.push(await applyEvent(client, userId, tenantId, event));
    return results;
  });
}

export async function pullEvents(userId: string, tenantId: string, after: string | undefined, limit: number) {
  return withTenant(userId, tenantId, async (client) => {
    const result = await client.query(
      `SELECT event_id, actor_id, aggregate_type, aggregate_id, base_version, resulting_version, received_seq,
              event_type, schema_version, payload, outcome, outcome_detail, received_at
       FROM sync_inbox
       WHERE tenant_id = $1 AND outcome = 'accepted' AND ($2::bigint IS NULL OR received_seq > $2::bigint)
       ORDER BY received_seq LIMIT $3`,
      [tenantId, after ?? null, limit]
    );
    return result.rows;
  });
}
