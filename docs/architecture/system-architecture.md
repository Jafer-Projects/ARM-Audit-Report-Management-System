# ARM System Architecture

## 1. Architecture choice

Use a **modular monolith with independently deployable background workers** for the initial product. Do not start with microservices. Domain boundaries, contracts and data ownership must be explicit so high-load or high-risk modules can be extracted later if justified.

## 2. Technology baseline

| Layer | Selection | Responsibility |
|---|---|---|
| Windows desktop | Tauri 2 | Desktop shell, secure native bridge and installer |
| Native services | Rust | Filesystem operations, protected local services and performance-sensitive tasks |
| Shared UI | React + TypeScript | Desktop and web user interfaces |
| API/backend | Node.js + TypeScript | Business rules, authorization, workflows and sync APIs |
| Local relational store | SQLite | Offline transactional workspace and outbox |
| Cloud relational store | PostgreSQL | Authoritative shared records and transactions |
| Object storage | S3-compatible | Documents, evidence, exports and backups |
| Background jobs | Worker process | OCR, exports, notifications, scheduled work and integration tasks |
| Queue/cache | Redis when justified | Job coordination, rate limits and cache; never financial source of truth |
| Hosting | Ubuntu LTS VPS initially | Containerized API, worker and database services |

Verify supported versions, security advisories, licenses, Tauri plugin trust, SQLite encryption options and operational support before pinning dependencies. Keep a software bill of materials and dependency update policy.

## 3. Logical components

### Desktop client
- Presentation layer.
- Local application service layer.
- Domain modules shared where safe with server.
- SQLite repository and migrations.
- Local document vault.
- Synchronization coordinator.
- Device identity and offline authorization verifier.
- Report rendering/export.

### Cloud application
- Identity and organization module.
- Firm/client/engagement module.
- Accounting module.
- Audit module.
- Tax module.
- Document metadata and access module.
- Workflow and approval module.
- Licensing/billing module.
- Reporting module.
- Synchronization API.
- Government integration adapters.
- Audit event service.

### Background workers
- OCR and extraction.
- Malware scanning coordination.
- Document conversion and previews.
- Report generation.
- Notifications.
- Reconciliation/import jobs.
- Tax/export jobs.
- Government submission polling where authorized.
- Retention and backup verification tasks.

## 4. Domain boundaries

Modules own their business rules and tables. Cross-module interactions use explicit application services or domain events, not direct arbitrary table mutation. Accounting owns posted journal invariants. Audit owns procedures, workpapers and conclusions. Tax owns tax periods, rule versions, calculations and returns. Documents own binary metadata, versions and access checks.

## 5. Data flow

1. Desktop user performs an authorized local operation.
2. Local domain validation runs and persists the change plus outbox event in one SQLite transaction.
3. Sync coordinator sends ordered, idempotent events when connected.
4. API authenticates the device and user, validates tenant/engagement permissions and business invariants.
5. PostgreSQL transaction applies accepted changes and records deduplication/inbox state.
6. Server returns durable acknowledgment, canonical version and any conflicts.
7. Desktop marks only acknowledged events as synchronized and applies server changes.
8. Notifications and other devices receive authorized updates.

## 6. API design

- Version APIs and sync protocol independently.
- Use HTTPS only.
- Use short-lived access tokens and rotating refresh/device credentials.
- Scope requests to explicit tenant and workspace context.
- Use idempotency keys for commands that can be retried.
- Return structured validation errors and conflict payloads.
- Use resumable, pre-authorized document upload flows.
- Never accept a client-supplied tenant ID as proof of access.
- Provide correlation IDs and structured audit events.
- Document API contracts and generate/validate clients where practical.

## 7. Desktop/web UI sharing

Share design system, typed contracts and appropriate business logic. Do not share privileged native capabilities with the browser portal. Tauri commands must validate input, constrain filesystem paths and expose only the minimum required capabilities. The web portal must not depend on desktop-only APIs.

## 8. Deployment topology

Initial VPS may host reverse proxy, API and worker, with PostgreSQL on a protected service or separate database host depending on provider and budget. Object storage should be independent of the VPS where feasible. No public PostgreSQL port. Separate production, staging and development environments. Secrets are injected at runtime and never committed.

## 9. Scalability path

First scale vertically and optimize database queries, indexes, connection pooling and worker concurrency. Then separate PostgreSQL and workers, add multiple stateless API instances, use managed/object storage and introduce read replicas or specialized services only when measured demand justifies them.

## 10. Architecture quality attributes

- Offline resilience and data durability.
- Strong tenant and engagement isolation.
- Financial correctness and reproducibility.
- Traceable professional review.
- Secure document lifecycle.
- Recoverability and operational observability.
- Maintainable modular boundaries.
- Backward-compatible synchronization.


## Cloud-first implementation alignment — initial slice

The repository now contains an initial Node.js/TypeScript cloud API in `server/`, alongside—not in place of—the existing Rust/Tauri desktop backend. The cloud package implements health endpoints, initial user/tenant registration, password login, short-lived signed access tokens, rotating refresh tokens, session revocation, tenant context validation and a versioned synchronization inbox. PostgreSQL migrations establish users, tenants, memberships, sessions, sync aggregates/inbox, audit events and a worker queue. PostgreSQL RLS is enabled and forced on tenant-owned tables; API tenant operations set transaction-local user and tenant context and verify active membership.

The separate worker executable claims jobs with `FOR UPDATE SKIP LOCKED`, bounded retry/backoff and worker-only RLS policy. The queue runtime is infrastructure only: domain handlers (OCR, malware scanning, notifications, exports and government adapters) are not yet implemented.

The generic `sync_aggregates` JSONB table is a protocol/bootstrap mechanism, not a replacement for domain-owned accounting, audit, tax, document or approval tables. High-risk operations require dedicated server-side commands and invariants. See `server/README.md` and `server/openapi.yaml` for the implemented surface and explicit limitations.
