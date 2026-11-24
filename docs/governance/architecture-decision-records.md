# ARM Architecture Decision Records

This document records the current approved baseline. Decisions may be superseded only through a reviewed change with rationale, impact and migration plan.

## ADR-001 — Windows-first offline desktop
**Status:** Accepted

**Decision:** Build a Windows desktop application using Tauri 2, Rust native services and React/TypeScript UI.

**Rationale:** The product requires substantive offline work, local document access and efficient desktop distribution. A browser-only application is insufficient as the primary professional work surface.

**Consequences:** Maintain a secure native bridge, signed installers, local migrations, update rollback and Windows endpoint security requirements. Other desktop OS support is not an initial commitment.

## ADR-002 — Shared TypeScript application stack
**Status:** Accepted

**Decision:** Use TypeScript/React for UI and Node.js/TypeScript for backend; use Rust for native desktop services.

**Rationale:** Limits language diversity while retaining native desktop capabilities.

**Consequences:** Enforce shared API contracts and domain boundaries. Specialized processing languages may be added only when justified by a measured requirement.

## ADR-003 — SQLite local and PostgreSQL cloud
**Status:** Accepted

**Decision:** Use SQLite for local offline state and PostgreSQL for authoritative cloud state.

**Rationale:** Embedded transactional storage supports offline operation; PostgreSQL provides robust relational and multi-user cloud data management.

**Consequences:** Implement a purpose-built, versioned synchronization protocol. Do not use naive database replication or last-write-wins conflict resolution. Select and validate local encryption/key management before storing real client data.

## ADR-004 — Modular monolith initially
**Status:** Accepted

**Decision:** Start with a modular monolith and independently deployable background workers.

**Rationale:** Appropriate for moderate budget and early scale while preserving domain boundaries.

**Consequences:** Define module ownership and contracts. Extract services only after measured scaling, isolation or availability needs.

## ADR-005 — Firm-administered client workspaces
**Status:** Accepted

**Decision:** The firm creates and administers client workspaces and controls staff access. Client users receive explicit scoped permissions.

**Rationale:** Matches the selected firm-led professional service model.

**Qualification:** Application administration does not decide legal ownership of all underlying client records or remove client rights. Contracts, applicable law, professional obligations, retention and export requirements govern.

## ADR-006 — Dual individual and firm subscriptions
**Status:** Accepted

**Decision:** Support individual professional plans and firm plans with seat management.

**Rationale:** Supports independent practitioners and established firms.

**Consequences:** Separate subscription entitlements from organization membership and data permissions. A personal subscription cannot bypass firm confidentiality boundaries.

## ADR-007 — Professional-maintained client accounting
**Status:** Accepted

**Decision:** Accountants/bookkeepers maintain client books. Clients provide source records and may review/confirm selected information. Auditors receive scoped access.

**Rationale:** Matches the selected operating model and provides a controlled accounting record.

**Consequences:** Implement preparer/reviewer controls, source-document traceability and separation between accounting books and audit workpapers.

## ADR-008 — Government integrations behind adapters
**Status:** Accepted

**Decision:** Implement separate adapters for e-invoicing, tax return filing and other authorized services.

**Rationale:** External interfaces, approvals and regulations can change independently.

**Consequences:** Manual preparation/export remains available. Direct submission is not enabled until official specifications, access authorization, certification and sandbox validation are established.

## ADR-009 — Human approval for professional conclusions
**Status:** Accepted

**Decision:** Automation and AI may assist but cannot independently issue audit opinions, approve material accounting postings or submit tax returns.

**Rationale:** Preserves professional accountability and traceability.

**Consequences:** All automated outputs are labeled, reviewable and linked to source data and rule/model versions.

## ADR-010 — VPS-first deployment with independent backups
**Status:** Accepted

**Decision:** Start on Ubuntu LTS VPS with containerized services, private database access and independent object storage/backup.

**Rationale:** Fits the selected hosting direction and moderate budget.

**Consequences:** Capacity must be benchmarked; single-VPS failure must not destroy the only copy of data. Define and test RPO/RTO before production.


## ADR-011 — Cloud API is authoritative for shared state

**Status:** Accepted; initial implementation in progress.

**Decision:** Implement the approved Node.js/TypeScript modular-monolith API and PostgreSQL as the authoritative cloud boundary for shared multi-tenant state. Retain Tauri/Rust and SQLite for native desktop capabilities, local offline transactions, drafts and synchronization outbox. Run background work in a separately deployable worker process. Use explicit versioned APIs and synchronization envelopes; do not use database replication or last-write-wins merging.

**Rationale:** The approved architecture already requires a cloud-first source of truth while preserving substantive offline desktop use. A server boundary is required for centralized authorization, tenant isolation, collaboration, shared records and controlled integrations.

**Consequences:** New shared business operations must be implemented and authorized server-side before production use. Existing local-only commands remain prototype/local behavior until migrated through explicit domain contracts. The generic JSON sync aggregate is not a canonical financial data model. The first cloud slice does not authorize real client data, live filing or production deployment. Full email verification, MFA, membership administration, device leases, object storage, domain handlers, conflict UI, deployment hardening and independent security review remain release blockers.
