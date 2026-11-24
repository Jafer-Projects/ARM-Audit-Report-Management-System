# ARM cloud production security review gate

This checklist is a release gate, not a claim that an independent security review has already happened. It applies to the exact release candidate SHA and its deployed configuration.

## Current source-level release blockers (2026-11-24)

The cloud-first branch is a foundation, not a production-ready desktop/cloud product. The following are known blockers from the workflow-parity audit in `docs/architecture/offline-first-and-sync.md`:

- The normal desktop shell still initializes the seeded `ARM Demonstration Firm`; the cloud session is isolated in a panel and does not authorize existing business commands.
- Cloud access/refresh tokens remain in component memory; there is no OS credential-vault integration or complete session lifecycle at the application-shell level.
- All existing desktop business methods still call Tauri/SQLite. The server has no client-directory domain model/API, and several other existing desktop workflows have no cloud domain model or routes.
- Audit/accounting cloud endpoints are not integrated into dedicated desktop workflows. The server domain is not a complete audit, accounting, tax or reporting product.
- Generic sync supports only `prototype.workspace_record`; the visible sync action is disabled for the seeded demo workspace. There is no complete per-domain sync, device identity/lease, conflict-resolution workspace or safe rejected-event recovery UX.
- The document APIs do not yet cover the desktop's generic record attachment contract or provide complete resumable desktop upload/download integration.
- No complete, resumable, auditable SQLite-to-PostgreSQL production migration and reconciliation report exists. Never upload the seeded demo data as customer data.
- The cloud API dependency lockfile and successful exact-head cloud CI evidence must be verified before a release. A workflow that may generate a lockfile is not evidence that a lockfile or successful run exists.
- Production infrastructure, external security review and isolated full database/object restore evidence remain unverified.

These blockers prohibit production tenant onboarding, real audit evidence, statutory filing, accounting posting for customer books, and production synchronization until the relevant gates below are completed. Do not bypass the disabled sync action or demo pairing guard to make a demonstration appear operational.

## Identity and access
- [ ] Make Node session, selected tenant, membership, permission and device context the application-wide production identity boundary; demo mode must be isolated and visibly distinct.
- [ ] Store refresh credentials using a reviewed OS credential vault integration; define rotation, expiry, logout, revocation and secure failure behavior.
- [ ] Verify email challenge expiry, one-time use, resend throttling and non-enumerating recovery behavior.
- [ ] Verify MFA TOTP drift window, replay handling, recovery-code single use, MFA disable re-authentication and recovery policy.
- [ ] Require MFA for tenant owners and privileged approvers before production.
- [ ] Review session rotation, revocation, password-reset invalidation and concurrent refresh races.
- [ ] Test invitation email matching, expiry, revocation, duplicate acceptance and cross-tenant denial.
- [ ] Review every permission key and every route; confirm no client-supplied role or tenant ID bypasses authorization.
- [ ] Integrate member, role, permission, invitation and device administration into the authenticated desktop experience.

## Tenant and database isolation
- [ ] Run negative cross-tenant tests for every tenant-owned table and every read/write path.
- [ ] Verify runtime, worker and migrator roles are distinct, non-superuser and have no BYPASSRLS.
- [ ] Verify worker custom GUC access is usable only with the worker database credential.
- [ ] Review SECURITY DEFINER functions, ownership, fixed search_path, grants and caller identity checks.
- [ ] Test append-only audit records, posted accounting journals and immutable evidence.
- [ ] Confirm migration role is not used by application runtime.
- [ ] Add production client, category, product, invoice, period, historical-import, staff/task/review, costing, notification and history models with tenant-scoped policies before enabling those workflows.

## Workflow parity and data migration
- [ ] Classify every desktop action as cloud-online authoritative, offline-capable under a domain-specific sync contract, or intentionally local-only.
- [ ] For every migrated workflow, require a server-owned domain model, authenticated tenant-scoped API, typed desktop integration, domain validation, audit history and parity tests.
- [ ] Preserve domain semantics; do not replace accounting, audit, VAT, historical import, task review or evidence workflows with arbitrary JSON snapshots.
- [ ] Define offline write policy, dependency ordering, idempotency, canonical versions, conflicts, retry/backoff and rejected-event correction for each offline-capable domain.
- [ ] Bootstrap tenant directory/reference data before dependent records; never advance a pull cursor past an unresolved dependency.
- [ ] Build a dry-run, resumable, idempotent SQLite-to-PostgreSQL migration with explicit source workspace selection, field mapping, duplicate report, source checksums, relationship validation, audit-history preservation and reconciliation totals.
- [ ] Keep seeded demonstration records out of customer migration; require explicit operator selection and reviewed mapping for each real source workspace.
- [ ] Prove rollback/forward-repair behavior and preserve the original SQLite source until reconciliation is approved.

## Audit and accounting correctness
- [ ] Obtain professional review of report templates, engagement lifecycle and approval independence.
- [ ] Validate account code, currency, precision, period lock, tax, ledger posting and reversal rules.
- [ ] Add database-level posting procedure and reconciliation controls before production accounting use.
- [ ] Test concurrency, idempotency, duplicate journals and immutable reversal behavior.
- [ ] Validate report issuance/signature, version history, retention and legal requirements.
- [ ] Do not enable VAT/statutory calculations until approved rule packs and independently checked fixtures exist.
- [ ] Keep final report issuance, journal posting, role changes and statutory filing online-only unless a separately approved offline authorization model exists.

## Evidence and infrastructure
- [ ] Verify S3-compatible private object storage, encryption at rest, key rotation and tenant-isolated access.
- [ ] Verify ClamAV signatures update, scanner failure quarantine, size/time limits and no clean download before scan.
- [ ] Verify TLS certificate chain, renewal, HSTS, ingress limits, network isolation and firewall.
- [ ] Store all credentials in an approved secret manager; rotate and test emergency revocation.
- [ ] Run dependency/SBOM/license/vulnerability scans and pin container images by digest.
- [ ] Enable centralized security logs, alerting, rate limits and incident response.
- [ ] Perform external penetration test and remediate all critical/high findings.
- [ ] Complete data-protection, retention, deletion, backup and disaster-recovery review.

## Recovery evidence
- [ ] Schedule encrypted database and document-object backups to a separate failure domain.
- [ ] Verify backup checksums, encryption-key escrow and offsite replication.
- [ ] Run `ops/restore-drill.sh` after every material schema/storage change.
- [ ] Restore both database and document objects into isolated resources and verify referenced object hashes.
- [ ] Record RPO/RTO results, approver and drill evidence.

## Automated release verification
- [ ] Run frontend typecheck, tests and production build.
- [ ] Run Rust format, all-target build, tests, Clippy and Windows desktop build.
- [ ] Generate and commit `server/package-lock.json`; verify clean `npm ci`, TypeScript build, repeated migrations and PostgreSQL integration tests.
- [ ] Run migration upgrade tests from every supported schema version and verify downgrade/forward-repair policy.
- [ ] Run workflow parity tests online, offline, after reconnect, across two devices, after permission revocation and after partial failure.
- [ ] Confirm every required check passed on the exact release SHA; do not rely on a previous commit's green checks.

## Release decision

Production authorization requires all applicable checkboxes to be supported by retained evidence, passing CI on the exact release SHA, a completed independent security review, verified storage/scanner deployment, successful full restore drill, workflow/data migration sign-off and named operational approval. No box is presumed checked by source-code presence. PR #2 is a draft foundation and is not a production deployment authorization.
