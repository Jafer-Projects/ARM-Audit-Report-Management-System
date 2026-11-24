# ARM Data and Security Architecture

## 1. Data stores

### SQLite on Windows
Contains only data assigned to the signed-in user's permitted workspaces, local drafts, sync metadata, document cache and offline policy snapshot. Encrypt sensitive data using a maintained solution and protect encryption keys with Windows platform key protection. Database encryption does not replace OS account controls, endpoint protection or secure backups.

### PostgreSQL in cloud
Authoritative shared relational store for users, firms, memberships, clients, engagements, accounting, audit, tax, approvals, sync inbox/outbox and audit events. Use transactions, constraints, foreign keys and carefully designed tenant isolation.

### Object storage
Stores original documents, evidence, generated reports and exports. Use private buckets, short-lived scoped upload/download URLs, encryption, versioning/retention and independent backups. Store hashes and metadata in the relational database.

## 2. Suggested logical entities

Identity and tenancy:
- users, credentials, sessions, devices, device_keys
- firms, firm_memberships, roles, permissions, invitations
- subscriptions, plans, entitlements, seat_assignments

Client and engagement:
- clients, client_branches, client_contacts, client_access_grants
- engagements, engagement_team, engagement_terms, engagement_status_history
- independence_checks, acceptance_assessments

Accounting:
- fiscal_periods, currencies, exchange_rates
- chart_of_accounts, accounts, journals, journal_lines
- reconciliation_sessions, reconciliation_items
- receivables, payables, payroll_runs, inventory_items, fixed_assets
- budgets, financial_statement_runs

Audit:
- audit_plans, materiality_assessments, risks, controls
- audit_programmes, procedures, samples, test_results
- findings, proposed_adjustments, management_responses
- workpapers, review_notes, signoffs, report_versions

Documents:
- documents, document_versions, document_blobs
- evidence_links, document_requests, document_submissions
- access_grants, retention_holds

Tax:
- tax_profiles, tax_periods, tax_rule_sets, tax_rule_versions
- tax_calculations, tax_adjustments, tax_returns
- filing_attempts, filing_receipts, payment_references

Synchronization and governance:
- sync_events, sync_inbox, sync_outbox, device_cursors, sync_conflicts
- audit_events, approval_requests, workflow_transitions
- regulatory_sources, rule_change_proposals, release_records

This is a logical inventory, not a finalized physical schema. Normalize transactional data, define aggregate boundaries and review indexing/partitioning against real workloads.

## 3. Financial invariants

- Store money as fixed-precision decimal/numeric values or integer minor units with explicit currency precision; never use binary floating point for financial arithmetic.
- Every posted journal balances per currency and defined accounting policy.
- Posting is atomic.
- Posted entries cannot be destructively edited or deleted.
- Corrections use reversal and replacement with references.
- Closed periods are protected.
- Exchange rates record source, effective date and approval where required.
- Tax and financial calculations record rule/policy version, inputs and rounding method.
- Reports retain a reproducible data snapshot/reference and template version.

## 4. Multi-tenant isolation

Every tenant-owned record carries an organization/tenant scope. Use PostgreSQL row-level security as defense in depth, with transaction-scoped tenant context set only after server authentication and membership validation. The application must also enforce authorization. Use composite foreign keys or equivalent constraints where needed to prevent cross-tenant references.

Test:
- direct object ID substitution;
- cross-firm search;
- export and download URL reuse;
- background job tenant context;
- cache key collision;
- sync event tenant spoofing;
- support/admin access;
- database role and RLS bypass boundaries.

Never rely on UI hiding as authorization.

## 5. Authentication and device security

- MFA for privileged and professional accounts.
- Secure password hashing using a maintained modern algorithm.
- Short-lived access tokens and rotating refresh credentials.
- Device registration, key rotation and revocation.
- Signed desktop update packages.
- Protected local key storage.
- Least-privilege native bridge.
- Secure recovery and session invalidation.
- Rate limiting and security event monitoring.

## 6. Audit trail

Use append-only events for security-sensitive and professional actions. Include actor, tenant, device, action, target, timestamp, correlation ID and references to before/after versions. Protect logs from ordinary application administrators; use hash chaining or another verifiable tamper-evidence design where appropriate. A hash chain alone does not prevent privileged deletion unless anchored and independently protected.

## 7. Document security

- Allowlist required file types and validate content, not only extensions.
- Enforce size and decompression limits.
- Malware scan before making files available to other users.
- Isolate OCR and conversion workers.
- Prevent path traversal and unsafe macro execution.
- Use private object storage and scoped URLs.
- Preserve original bytes and cryptographic hashes.
- Record all access, release and export events.

## 8. Privacy and retention

Before production, document lawful purpose, data minimization, notice/consent where applicable, controller/processor responsibilities, retention, legal holds, client export, deletion, incident response and cross-border hosting. Avoid collecting identity or payroll fields that are not necessary. Obtain qualified local legal advice; this document does not determine legal compliance.

## 9. Backup and recovery

- Encrypted PostgreSQL backups and point-in-time recovery where supported.
- Independent object storage replication/versioning.
- Encrypted backup copies outside the primary VPS/provider.
- Consistent document metadata/blob reconciliation.
- Regular restore drills.
- Defined RPO/RTO approved by product owners.
- Device-local recovery and export procedures.
- Alert on failed or stale backups.

## 10. Threat modelling

Maintain a threat model covering lost/stolen laptops, malicious insiders, compromised firm accounts, cross-tenant access, malware in uploads, malicious documents, sync replay, credential theft, compromised update channel, ransomware, VPS compromise, backup compromise and government connector credential exposure. Reassess after material architecture changes.


## Cloud data/security implementation status

The initial cloud migration creates identity, tenant membership, session, synchronization, append-only audit and worker-job tables. RLS is enabled with FORCE on tenant-owned tables. The API sets `app.user_id` and `app.tenant_id` using transaction-local settings and performs an active-membership check before tenant operations. Tenant identifiers in headers are selectors only, never authorization evidence.

The first integration suite exercises registration, tenant context, cross-tenant denial, idempotent synchronization, stale-version conflict, pull visibility, refresh rotation and session revocation. It does not yet constitute a complete tenant-isolation assessment. Before production, add direct SQL/RLS bypass-boundary tests using separate least-privilege roles, organization invitation and role-administration tests, device/revocation tests, privileged support controls, MFA/email verification, secrets rotation and an independent security review. Audit and sync inbox rows have append-only triggers; production grants must additionally deny mutation privileges to application roles.

The cloud bootstrap separates three non-superuser credentials: `arm_runtime` (only explicitly granted API tables), `arm_worker` (queue table only, with column-level update rights), and `arm_migrator` (schema ownership and forward migrations). The database bootstrap administrator is not used by API or worker processes. Runtime grants are explicit in the migration rather than broad default table grants, so a new table is not automatically exposed to the API. CI asserts the role boundaries and the API/worker cannot read the migration ledger or tenant identity tables outside their grants.


The cloud database roles and security-definer functions are explicitly scoped. Security-definer functions use a fixed search path and are executable by the runtime role only. Identity-sensitive verification, MFA and invitation functions validate the transaction-local caller identity; pre-authentication account lookup is deliberately limited to the recovery/login flow and must remain behind non-enumerating rate-limited API routes. The worker-only RLS policies additionally require the actual `arm_worker` database identity; setting a custom worker GUC from the API role is insufficient. The API runtime has no access to the migration ledger or job queue, and the worker cannot read tenant business data.

Production object storage uses an S3-compatible adapter with SigV4 request signing and server-side encryption. The bucket must be private, enforce TLS and encryption, block public access and use a least-privilege key. TLS is terminated by the Nginx gateway; only port 443 is published. Mounted Compose secret files are a deployment mechanism, not a substitute for an approved host/cloud secret manager, rotation and access auditing.

These controls still require independent review and adversarial tests. In particular, RLS context is established by the trusted API transaction layer; a database credential compromise or arbitrary SQL execution under the shared runtime role is outside the protection boundary of a plain custom GUC. Use parameterized queries, least-privilege grants, injection testing and independent penetration review.
