import Fastify, { type FastifyInstance } from "fastify";
import helmet from "@fastify/helmet";
import cors from "@fastify/cors";
import rateLimit from "@fastify/rate-limit";
import { z } from "zod";
import { config } from "./config.js";
import { pool, withTenant } from "./db.js";
import { authenticate, login, register, revokeSession, rotateRefreshToken, completeMfaLogin, type AuthenticatedRequest } from "./auth.js";
import { verifyEmailAddress, requestPasswordRecovery, resetPassword, beginMfaEnrollment, confirmMfaEnrollment, disableMfa } from "./security.js";
import { pushEvents, pullEvents, pushSchema } from "./sync.js";
import { listRoles, createRole, replaceRolePermissions, assignMemberRole, inviteOrganizationMember } from "./authorization.js";
import { acceptOrganizationInvitation } from "./organizations.js";
import { createEngagement, listEngagements, createWorkpaper, transitionWorkpaper, createAuditReport, transitionAuditReport, createJournal, postJournal, createAccount, listAccounts } from "./domain.js";
import { uploadDocument, readCleanDocument } from "./documents.js";

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
    bodyLimit: 72_000_000
  });
  await app.register(helmet);
  await app.register(cors,{origin:config.CORS_ORIGINS.split(",").map((value)=>value.trim()).filter(Boolean),methods:["GET","POST","PUT","OPTIONS"],allowedHeaders:["authorization","content-type","x-tenant-id","x-correlation-id"],credentials:false,maxAge:600});
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

  app.post("/v1/auth/verify-email", async (request, reply) => {
    const parsed = z.object({ token: z.string().min(32).max(128) }).safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error" });
    return verifyEmailAddress(parsed.data.token);
  });

  app.post("/v1/auth/recovery/request", { config: { rateLimit: { max: 5, timeWindow: "15 minutes" } } }, async (request, reply) => {
    const parsed = z.object({ email: z.string().email().max(254) }).safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error" });
    return requestPasswordRecovery(parsed.data.email);
  });

  app.post("/v1/auth/recovery/complete", { config: { rateLimit: { max: 5, timeWindow: "15 minutes" } } }, async (request, reply) => {
    const parsed = z.object({ token: z.string().min(32).max(128), password: z.string().min(12).max(128) }).safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error" });
    return resetPassword(parsed.data.token,parsed.data.password);
  });

  app.post("/v1/auth/mfa/complete", { config: { rateLimit: { max: 8, timeWindow: "1 minute" } } }, async (request, reply) => {
    const parsed = z.object({ challengeToken: z.string().min(32).max(128), code: z.string().regex(/^(?:\\d{6}|[a-f0-9]{16})$/i) }).safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: "validation_error" });
    return completeMfaLogin(parsed.data.challengeToken,parsed.data.code);
  });

  app.post("/v1/auth/mfa/enroll", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request,reply) => {
    const parsed=z.object({password:z.string().min(1).max(128)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return beginMfaEnrollment(auth.sub,auth.sub,parsed.data.password);
  });

  app.post("/v1/auth/mfa/confirm", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request,reply) => {
    const parsed=z.object({code:z.string().regex(/^\\d{6}$/)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    return confirmMfaEnrollment((request as AuthenticatedRequest).auth!.sub,parsed.data.code);
  });

  app.post("/v1/auth/mfa/disable", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request,reply) => {
    const parsed=z.object({password:z.string().min(1).max(128),code:z.string().regex(/^\\d{6}$/)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    return disableMfa((request as AuthenticatedRequest).auth!.sub,parsed.data.password,parsed.data.code);
  });

  app.post("/v1/auth/logout", { preHandler: async (request) => {
    (request as AuthenticatedRequest).auth = await authenticate(request);
  } }, async (request, reply) => {
    const auth = (request as AuthenticatedRequest).auth!;
    await revokeSession(auth.sid, auth.sub);
    return reply.code(204).send();
  });

  app.get("/v1/tenant/roles", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request) => {
    const auth=(request as AuthenticatedRequest).auth!;
    return {roles:await listRoles(auth.sub,tenantId(request as AuthenticatedRequest))};
  });

  app.post("/v1/tenant/roles", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({key:z.string().regex(/^[a-z][a-z0-9_.-]{1,63}$/),name:z.string().trim().min(2).max(100),permissions:z.array(z.string()).max(100)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await createRole(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });

  app.put("/v1/tenant/roles/:roleId/permissions", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({permissions:z.array(z.string()).max(100)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!,params=request.params as {roleId:string};
    return replaceRolePermissions(auth.sub,tenantId(request as AuthenticatedRequest),params.roleId,parsed.data.permissions);
  });

  app.post("/v1/tenant/memberships/:membershipId/roles", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({roleId:z.string().uuid()}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!,params=request.params as {membershipId:string};
    return assignMemberRole(auth.sub,tenantId(request as AuthenticatedRequest),params.membershipId,parsed.data.roleId);
  });

  app.post("/v1/tenant/invitations", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({email:z.string().email().max(254),roleId:z.string().uuid()}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await inviteOrganizationMember(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data.email,parsed.data.roleId,config.PUBLIC_APP_URL));
  });

  app.post("/v1/tenant/invitations/accept", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({token:z.string().min(32).max(128)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    return acceptOrganizationInvitation((request as AuthenticatedRequest).auth!.sub,parsed.data.token);
  });

  app.get("/v1/audit/engagements", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request) => {
    const auth=(request as AuthenticatedRequest).auth!;
    return {engagements:await listEngagements(auth.sub,tenantId(request as AuthenticatedRequest))};
  });
  app.post("/v1/audit/engagements", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({clientReference:z.string().min(1).max(100),title:z.string().min(3).max(200),periodStart:z.string().date(),periodEnd:z.string().date(),framework:z.string().min(1).max(80)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await createEngagement(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });
  app.post("/v1/audit/workpapers", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({engagementId:z.string().uuid(),reference:z.string().min(1).max(80),title:z.string().min(1).max(200),procedure:z.string().min(1).max(10000)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await createWorkpaper(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });
  app.post("/v1/audit/workpapers/:id/transition", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({expectedVersion:z.number().int().positive(),action:z.enum(["prepare","review","approve","reopen"]),conclusion:z.string().max(10000).optional()}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!,params=request.params as {id:string};
    return transitionWorkpaper(auth.sub,tenantId(request as AuthenticatedRequest),params.id,parsed.data.expectedVersion,parsed.data.action,parsed.data.conclusion);
  });
  app.post("/v1/audit/reports", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({engagementId:z.string().uuid(),reportType:z.enum(["independent_auditor","management_letter","internal_audit","special_purpose"]),title:z.string().min(1).max(200),body:z.record(z.string(),z.unknown())}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await createAuditReport(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });
  app.post("/v1/audit/reports/:id/transition", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({expectedVersion:z.number().int().positive(),action:z.enum(["prepare","review","approve","issue","withdraw"])}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!,params=request.params as {id:string};
    return transitionAuditReport(auth.sub,tenantId(request as AuthenticatedRequest),params.id,parsed.data.expectedVersion,parsed.data.action);
  });
  app.get("/v1/accounting/accounts", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request) => {
    const auth=(request as AuthenticatedRequest).auth!;
    return {accounts:await listAccounts(auth.sub,tenantId(request as AuthenticatedRequest))};
  });
  app.post("/v1/accounting/accounts", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({code:z.string().min(1).max(40),name:z.string().min(1).max(160),type:z.enum(["asset","liability","equity","income","expense"]),currency:z.string().regex(/^[A-Z]{3}$/)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await createAccount(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });

  app.post("/v1/accounting/journals", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({journalNumber:z.string().min(1).max(80),date:z.string().date(),description:z.string().min(1).max(500),currency:z.string().regex(/^[A-Z]{3}$/),lines:z.array(z.object({accountId:z.string().uuid(),description:z.string().max(500).optional(),debit:z.number().nonnegative(),credit:z.number().nonnegative()})).min(2).max(500)}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(201).send(await createJournal(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });
  app.post("/v1/accounting/journals/:id/post", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request) => {
    const auth=(request as AuthenticatedRequest).auth!,params=request.params as {id:string};
    return postJournal(auth.sub,tenantId(request as AuthenticatedRequest),params.id);
  });

  app.post("/v1/documents", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request,reply) => {
    const parsed=z.object({fileName:z.string().min(1).max(255),mediaType:z.enum(["application/pdf","image/jpeg","image/png"]),contentBase64:z.string().min(8).max(70_000_000),engagementId:z.string().uuid().optional(),workpaperId:z.string().uuid().optional(),reportId:z.string().uuid().optional()}).safeParse(request.body);
    if(!parsed.success) return reply.code(400).send({error:"validation_error"});
    const auth=(request as AuthenticatedRequest).auth!;
    return reply.code(202).send(await uploadDocument(auth.sub,tenantId(request as AuthenticatedRequest),parsed.data));
  });
  app.get("/v1/documents/:id/content", { preHandler: async (request) => { (request as AuthenticatedRequest).auth = await authenticate(request); } }, async (request) => {
    const auth=(request as AuthenticatedRequest).auth!,params=request.params as {id:string};
    return readCleanDocument(auth.sub,tenantId(request as AuthenticatedRequest),params.id);
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
      after: z.string().regex(/^\d{1,19}$/).refine((value) => BigInt(value) <= 9223372036854775807n).optional(),
      limit: z.coerce.number().int().min(1).max(100).default(50)
    }).safeParse(request.query);
    if (!parsed.success) throw Object.assign(new Error("Invalid sync cursor or limit"), { statusCode: 400 });
    const events = await pullEvents(auth.sub, tenantId(request as AuthenticatedRequest), parsed.data.after, parsed.data.limit);
    return { events, nextCursor: events.length ? String(events[events.length - 1].received_seq) : parsed.data.after ?? null };
  });

  app.addHook("onClose", async () => { await pool.end(); });
  return app;
}
