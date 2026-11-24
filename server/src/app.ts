import Fastify, { type FastifyInstance } from "fastify";
import helmet from "@fastify/helmet";
import rateLimit from "@fastify/rate-limit";
import { z } from "zod";
import { config } from "./config.js";
import { pool, withTenant } from "./db.js";
import { authenticate, login, register, revokeSession, rotateRefreshToken, type AuthenticatedRequest } from "./auth.js";
import { pushEvents, pullEvents, pushSchema } from "./sync.js";

const registerSchema = z.object({
  email: z.string().email().max(254),
  displayName: z.string().trim().min(1).max(160),
  password: z.string().min(12).max(128),
  organizationName: z.string().trim().min(2).max(200)
});
const loginSchema = z.object({ email: z.string().email().max(254), password: z.string().min(1).max(128) });
const refreshSchema = z.object({ refreshToken: z.string().min(40).max(256) });

function tenantId(request: AuthenticatedRequest): string {
  const value = request.headers["x-tenant-id"];
  if (typeof value !== "string" || !z.string().uuid().safeParse(value).success) {
    throw Object.assign(new Error("A valid x-tenant-id header is required"), { statusCode: 400 });
  }
  return value;
}

export async function buildApp(): Promise<FastifyInstance> {
  const app = Fastify({
    logger: { level: config.LOG_LEVEL, redact: ["req.headers.authorization", "req.headers.cookie"] },
    requestIdHeader: "x-correlation-id",
    bodyLimit: 1_048_576
  });
  await app.register(helmet);
  await app.register(rateLimit, { max: 120, timeWindow: "1 minute" });

  app.setErrorHandler((error, request, reply) => {
    const status = (error as { statusCode?: number }).statusCode ?? 500;
    if (status >= 500) request.log.error({ err: error }, "request failed");
    reply.code(status).send({
      error: status >= 500 ? "internal_error" : "request_error",
      message: status >= 500 ? "The request could not be completed" : error.message,
      requestId: request.id
    });
  });

  app.get("/health/live", async () => ({ status: "ok" }));
  app.get("/health/ready", async (_request, reply) => {
    try {
      const result = await pool.query("SELECT to_regclass('public.users') IS NOT NULL AS identity_ready, to_regclass('public.sync_inbox') IS NOT NULL AS sync_ready");
      if (!result.rows[0]?.identity_ready || !result.rows[0]?.sync_ready) {
        return reply.code(503).send({ status: "not_ready", database: "available", schema: "incomplete" });
      }
      return { status: "ready", database: "available", schema: "ready" };
    } catch {
      return reply.code(503).send({ status: "not_ready", database: "unavailable" });
    }
  });

  app.post("/v1/auth/register", { config: { rateLimit: { max: 8, timeWindow: "1 minute" } } }, async (request, reply) => {
    const parsed = registerSchema.safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error", issues: parsed.error.issues });
    try {
      return reply.code(201).send(await register(parsed.data));
    } catch (error) {
      if ((error as { code?: string }).code === "23505") return reply.code(409).send({ error: "account_already_exists" });
      throw error;
    }
  });

  app.post("/v1/auth/login", { config: { rateLimit: { max: 8, timeWindow: "1 minute" } } }, async (request, reply) => {
    const parsed = loginSchema.safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error", issues: parsed.error.issues });
    return login(parsed.data.email, parsed.data.password);
  });

  app.post("/v1/auth/refresh", { config: { rateLimit: { max: 20, timeWindow: "1 minute" } } }, async (request, reply) => {
    const parsed = refreshSchema.safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error", issues: parsed.error.issues });
    return rotateRefreshToken(parsed.data.refreshToken);
  });

  app.post("/v1/auth/logout", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request, reply) => {
    const auth = (request as AuthenticatedRequest).auth!;
    await revokeSession(auth.sid, auth.sub);
    return reply.code(204).send();
  });

  app.get("/v1/tenant/context", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request) => {
    const auth = (request as AuthenticatedRequest).auth!;
    const id = tenantId(request as AuthenticatedRequest);
    return withTenant(auth.sub, id, async (client) => {
      const result = await client.query(
        "SELECT t.id, t.name, t.slug, m.role FROM tenants t JOIN tenant_memberships m ON m.tenant_id = t.id WHERE t.id = $1 AND m.user_id = $2 AND m.status = 'active'",
        [id, auth.sub]
      );
      return result.rows[0];
    });
  });

  app.post("/v1/sync/events", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request, reply) => {
    const parsed = pushSchema.safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error", issues: parsed.error.issues });
    const auth = (request as AuthenticatedRequest).auth!;
    const results = await pushEvents(auth.sub, tenantId(request as AuthenticatedRequest), parsed.data.events);
    return { results };
  });

  app.get("/v1/sync/events", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request) => {
    const auth = (request as AuthenticatedRequest).auth!;
    const parsed = z.object({
      after: z.string().regex(/^\\d+$/).optional(),
      limit: z.coerce.number().int().min(1).max(100).default(50)
    }).safeParse(request.query);
    if (!parsed.success) throw Object.assign(new Error("Invalid sync cursor or limit"), { statusCode: 400 });
    const events = await pullEvents(auth.sub, tenantId(request as AuthenticatedRequest), parsed.data.after, parsed.data.limit);
    return { events, nextCursor: events.length ? String(events[events.length - 1].received_seq) : parsed.data.after ?? null };
  });

  app.addHook("onClose", async () => { await pool.end(); });
  return app;
}
