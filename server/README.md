# ARM Cloud API (foundational increment)

This package is the first Node.js/TypeScript cloud service for ARM. It is a modular-monolith foundation, not a completed migration of desktop business workflows.

## Local development

Requirements: Node.js 22.16+, PostgreSQL 16+, and npm.

```sh
cp .env.example .env
# Set separate random POSTGRES_PASSWORD, ARM_RUNTIME_PASSWORD,
# ARM_WORKER_PASSWORD and JWT_SECRET values before starting Compose.
docker compose --env-file .env up --build
```

Run the API integration suite against a disposable PostgreSQL database:

```sh
# Use a disposable PostgreSQL database. Bootstrap roles as its owner/admin first.
ADMIN_DATABASE_URL=postgres://owner:...@localhost:5432/arm_test \
MIGRATION_DATABASE_URL=postgres://owner:...@localhost:5432/arm_test \
DATABASE_URL=postgres://arm_runtime:...@localhost:5432/arm_test \
WORKER_DATABASE_URL=postgres://arm_worker:...@localhost:5432/arm_test \
ARM_RUNTIME_PASSWORD=... ARM_WORKER_PASSWORD=... JWT_SECRET=... \
npm run db:bootstrap-runtime
npm run build
npm test
```

The test command applies pending migrations before running the PostgreSQL-backed tests. Never point it at a production database.

## Current API surface

- `POST /v1/auth/register` — create a user and initial tenant owner.
- `POST /v1/auth/login` — authenticate and issue a short-lived access token plus a rotating refresh token.
- `POST /v1/auth/refresh` — rotate a refresh token.
- `POST /v1/auth/logout` — revoke the current session.
- `GET /v1/tenant/context` — resolve the caller's active role in the selected tenant; requires `x-tenant-id`.
- `POST /v1/sync/events` — submit versioned, idempotent aggregate upsert events.
- `GET /v1/sync/events` — pull accepted, conflict and rejected inbox outcomes using the monotonic `after` cursor.
- `GET /health/live` and `GET /health/ready` — process and database health.

The OpenAPI contract is in `openapi.yaml`.

## Security and current limitations

- Passwords are stored using Node's scrypt KDF; refresh tokens are stored only as SHA-256 digests.
- Access tokens are HMAC-SHA-256 signed, short-lived, and checked against a revocable server-side session on every authenticated request.
- Tenant context comes from a request header but is never trusted by itself: active membership is checked inside a transaction.
- Tenant-owned tables enforce PostgreSQL row-level security using transaction-local user and tenant context.
- Sync delivery is at-least-once. Inbox uniqueness, advisory locking and aggregate base versions make retries safe and stale updates explicit.
- Audit and sync inbox rows are append-only at the database trigger layer.

This first slice does **not** yet implement email verification, MFA, organization invitations/member administration, full permission matrices, device registration/leases, S3 document upload/scanning, domain-specific accounting/audit/tax command handlers, a complete conflict-resolution UI, or production secrets/key management. The worker queue runtime is present, but domain job handlers must be added with their owning modules. These are explicit blockers for real client data and production.

Do not use the generic JSON sync aggregate table as a substitute for domain-specific financial invariants. Posted journals, tax filings, report issuance and other high-risk operations must receive dedicated server-side commands, permissions, immutable history and independently tested invariants before they are synchronized as authoritative data.
