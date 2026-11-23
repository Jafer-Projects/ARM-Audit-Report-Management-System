# Phase 0 — Technical spike and validation matrix

All spikes use synthetic data, disposable environments and pinned candidate dependencies. A successful demonstration supports a decision; it is not production security certification.

| ID | Spike | Minimum scenarios | Required evidence | Pass condition |
|---|---|---|---|---|
| TS-01 | SQLite encryption and Windows key protection | Open/lock; wrong key; Windows user change; backup/restore; lost device; rotation; corruption. | Prototype, threat model, benchmark, license/advisory review, recovery runbook. | No plaintext sensitive data or reusable key in config/logs; recovery limits understood; independent security approval. |
| TS-02 | Tauri native boundary | Denied path, traversal, symlink, oversized input, untrusted web content, disabled command, plugin permissions. | Capability inventory, command allowlist, negative tests, build provenance. | UI cannot access arbitrary files/secrets/native capabilities; commands validate inputs and paths. |
| TS-03 | Local transaction/outbox | Crash before/after commit, retry, duplicate, out-of-order event, schema upgrade. | State log, invariant tests, recovery behavior. | Domain mutation and outbox persist atomically; no event sent before commit or lost after commit. |
| TS-04 | Server inbox/idempotency | Duplicate requests, same key/different payload, concurrent delivery, mid-transaction crash, replay. | Integration tests, transaction trace, deduplication design. | One durable business effect per idempotency key; mismatched payload rejected and audited. |
| TS-05 | Conflict handling | Concurrent metadata, journal, workpaper, approval and client submission edits. | Entity conflict policy and UX prototype. | No last-write-wins for financial/professional records; both versions preserved; sensitive conflicts require authorized resolution. |
| TS-06 | Resumable document transfer | Interrupted upload, duplicate chunk, checksum mismatch, expired token, malware scan failure. | Integrity report, limits, retry and quarantine flow. | Final hash matches source; incomplete/malicious files never marked accepted. |
| TS-07 | Offline lease | Long offline period, expiry, clock rollback, device revocation, user removal, restart. | Threat model and user-visible state design. | Privileged offline access is bounded and fail-safe; expiry/revocation cannot silently be bypassed. |
| TS-08 | Accounting numeric model | Decimal precision, rounding, negative values, FX, balanced journal, reversal, closed period. | Independent fixtures and invariant/property tests. | Results match approved expected values; posted journals cannot be unbalanced or destructively edited. |
| TS-09 | Tenant/engagement isolation | Cross-tenant IDs, filters, exports, search, jobs, object URLs and support access. | Automated abuse tests and access matrix. | Unauthorized paths deny access without disallowed existence/content leakage. |
| TS-10 | Backup and restore | Database, object store, keys/config, isolated restore, corruption and partial restore. | Timed restore, checksums, RPO/RTO evidence and runbook. | Complete consistent restore within approved targets; backup access separately controlled. |
| TS-11 | Windows packaging/update | Clean install, standard user, offline install, signature, interrupted update, rollback, uninstall. | Signed test installer, SBOM, compatibility and recovery notes. | Install/update integrity verified; interrupted update does not corrupt local vault. |
| TS-12 | Capacity and poor network | Sync burst, large documents, latency, packet loss, server restart and retry storm. | Load profile, latency/error metrics, resource use and cost estimate. | Pilot workload meets product-defined targets without data loss or retry amplification. |

## Evidence package

Record spike ID, date, commit, exact dependency versions, owner/reviewer, OS/hardware, environment, synthetic data, reproducible steps, expected/actual results, sanitized logs, measurements, limitations, findings and pass/fail/blocked status. Link the related issue and ADR.

## Immediate no-go findings

Plaintext sensitive data, key leakage, arbitrary native filesystem access, silent sync loss, duplicate financial effects, unbounded privileged offline access, cross-tenant disclosure or un-restorable backups are blockers. Do not downgrade/suppress tests to claim a pass.
