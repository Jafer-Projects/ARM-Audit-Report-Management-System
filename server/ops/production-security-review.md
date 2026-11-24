# ARM cloud production security review gate

This checklist is a release gate, not a claim that an independent security review has already happened.

## Identity and access
- [ ] Verify email challenge expiry, one-time use, resend throttling and non-enumerating recovery behavior.
- [ ] Verify MFA TOTP drift window, replay handling, recovery-code single use, MFA disable re-authentication and recovery policy.
- [ ] Require MFA for tenant owners and privileged approvers before production.
- [ ] Review session rotation, revocation, password-reset invalidation and concurrent refresh races.
- [ ] Test invitation email matching, expiry, revocation, duplicate acceptance and cross-tenant denial.
- [ ] Review every permission key and every route; confirm no client-supplied role or tenant ID bypasses authorization.

## Tenant and database isolation
- [ ] Run negative cross-tenant tests for every tenant-owned table and every read/write path.
- [ ] Verify runtime, worker and migrator roles are distinct, non-superuser and have no BYPASSRLS.
- [ ] Verify worker custom GUC access is usable only with the worker database credential.
- [ ] Review SECURITY DEFINER functions, ownership, fixed search_path, grants and caller identity checks.
- [ ] Test append-only audit records, posted accounting journals and immutable evidence.
- [ ] Confirm migration role is not used by application runtime.

## Audit and accounting correctness
- [ ] Obtain professional review of report templates, engagement lifecycle and approval independence.
- [ ] Validate account code, currency, precision, period lock, tax, ledger posting and reversal rules.
- [ ] Add database-level posting procedure and reconciliation controls before production accounting use.
- [ ] Test concurrency, idempotency, duplicate journals and immutable reversal behavior.
- [ ] Validate report issuance/signature, version history, retention and legal requirements.

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

## Release decision
Production authorization requires passing CI on the exact release SHA, a completed independent security review, verified storage/scanner deployment, successful full restore drill, and named operational approval. No box is presumed checked by source-code presence.
