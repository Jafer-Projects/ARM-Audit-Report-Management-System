# Increments 0–5 — implementation evidence log

**Governing PR:** [#2](https://github.com/Jafer-Projects/ARM-Audit-Report-Management-System/pull/1)  
**Branch:** `implementation/increments-0-to-5`  
**Acceptance model:** one collective gate after Increment 5; no increment-level acceptance/pass declarations.

This log records implementation progress, design decisions and prepared test evidence while the integrated prototype is being built. Formal verification and test execution are deferred until Increment 5 implementation is complete.

## Current implementation baseline

- [x] Tauri 2 desktop shell and React/TypeScript application scaffold added.
- [x] Local SQLite database with nine forward-only migrations, checksum validation, WAL/full-sync settings and restart recovery added.
- [x] Immutable workspace-record version snapshots and client before/after profile history added.
- [x] Synthetic demo firm and four demo personas seeded.
- [x] Client register with draft save, field validation, duplicate comparison/acknowledgement, optimistic concurrency, activation, archive and restore commands added.
- [x] Client profile history window displays before/after snapshots, actor and timestamp.
- [x] Increment 0 local draft notes with client association and persistence migration added.
- [x] Client-scoped category, product, invoice, period and historical-record capture added; records cannot be moved between client workspaces.
- [x] Invoice source metadata, same-client invoice-number duplicate advisory, explicit separate-invoice reason and read-only manager review queue added.
- [x] Historical CSV mapping preview and atomic import added (5 MB / 1,000-row limits, retained source content, SHA-256 checksum, row-level provenance, duplicate-reference flags and count reconciliation).
- [x] Draft VAT workpaper grouping, VAT exception capture and print/PDF preview added without calculations or filing output.
- [x] Staff directory and role-aware task overview, due-state filters, workload counts and assigned preparer/reviewer workflow added.
- [x] Demo client visibility is scoped to non-draft task assignment; auditor and read-only reviewer views are limited to assigned client workspaces; client lifecycle and staff administration are manager/admin-only; local notes are author-scoped and activity/version history is assignment-filtered.
- [x] Task review-note lifecycle added (Open → Responded → Cleared/Reopened), with no-delete/content-immutable guards, preparer response and reviewer disposition.
- [x] Submitted task content is protected from reviewer edits; task reassignment/reviewer change and cancellation require recorded reasons; task snapshots preserve revisions.
- [x] Import-cost discovery captures all 12 practitioner prompts and Gate 5-A supporting artifact references; cost inputs remain locked.
- [x] Append-only local activity event table and activity view added.
- [x] Offline/local-only status presented honestly.
- [x] Windows x64 CI workflow added for typecheck, domain tests, frontend build and Rust build.
- [x] Windows 10/11 and WebView2 compatibility requirements documented; future bundles are configured for the offline WebView2 installer mode, but bundling remains disabled and no installer has been built or runtime-tested.
- [x] Three Node domain contract tests executed locally: 3 passed.
- [x] SQLite in-memory smoke checks exercised immutable-history triggers and allowed task review-note transitions; foreign-key check returned no violations.
- [ ] Rust tests (including new migration, import, version-history, review-note and assigned-client authorization tests), TypeScript typecheck, frontend build and Tauri Windows build remain unverified because no application workflow jobs have run; the available repository Actions runs report startup failure with zero jobs.

These checkmarks mean source code has been added, not that the feature has passed acceptance or been verified by CI.

## Known limitations retained deliberately

- Prototype database is not yet encrypted. Synthetic data only; no production security claim.
- No cloud synchronization, server acknowledgment, device authorization lease or production identity/RBAC.
- No tax calculation, VAT treatment, import landed-cost calculation, customs valuation or selling-price recommendation.
- No MRC interpretation.
- No real client documents, OCR, government connector, report signing or tax filing.
- Broad Windows 10/11 application compatibility is a target; actual edition/build coverage must be measured. Windows Server CI is a build environment, not proof of Windows 10/11 compatibility.
- Dependency lockfiles are generated during CI and uploaded for review; they are not yet committed to the branch.

## Evidence entries

| Date | Commit | Evidence | Result / limitation |
|---|---|---|---|
| 2026-11-23 | Initial PR #1 implementation series | Historical pre-deferral checks: 3 Node tests passed; SQLite DDL/trigger smoke checks passed (not acceptance and not a current integrated verification result) | Rust/TS/build CI unverified; no increment accepted; no collective acceptance decision |


## Binding override — prototype build and deferred collective verification

The product owner has issued a later, explicit decision recorded in [PR #1](https://github.com/Jafer-Projects/ARM-Audit-Report-Management-System/pull/1) and [Prototype Build Binding Contract](prototype-build-binding-contract.md). ARM is a controlled prototype using synthetic data. All implementation remains on `implementation/increments-0-to-5` in PR #1. Close the outstanding Increments 0–5 implementation gaps to the best practical professional standard, preserving existing behavior and avoiding scope expansion.

**This later decision supersedes any earlier wording in this file that requires continuous test execution or increment-level verification.** Formal verification and testing of the integrated Increments 0–5 delivery occur only after Increment 5 implementation is complete. Do not declare any increment accepted, passed, or independently complete. Test plans and fixtures may be prepared during implementation, but partial checks must not be presented as acceptance. This timing decision does not permit test deletion, assertion weakening, or unsupported claims. The one collective gate and separate production/legal/security/professional gates remain mandatory.


## Product-owner-directed prototype completion update

**Binding rule:** See [Prototype Build Binding Contract](prototype-build-binding-contract.md). Implementation continues on this branch and PR #1. Formal verification and testing are deferred until Increment 5 implementation is complete. No increment is individually accepted.

### Increment 5 prototype implementation added

- Source-linked goods-line and cost-component input records can be captured under a unique client import-case reference.
- Import cases require discovery, source/component policy, FX policy, allocation policy and synthetic worked-example references before calculation.
- Cost inputs require source references and dates, currency codes, explicit FX numerator/denominator, FX source/date/authorizer, and an explicit cost treatment.
- The Rust calculation module uses integer minor units and checked integer arithmetic; no floating-point money arithmetic.
- Goods values are converted using the explicit FX ratio supplied for the source record.
- Shared cost allocation supports Direct, Quantity, Weight, Volume and Value bases. Shared allocations use deterministic largest-remainder distribution so allocated minor units reconcile exactly to the component amount.
- Cost treatment keeps Capitalize, Exclude, Recoverable tax, Customs-value-only and Needs review separate. Recoverable tax, customs-value-only and unresolved items are not silently capitalized.
- Generated calculation snapshots are saved as immutable draft records with source and FX provenance. A new calculation creates a new snapshot rather than overwriting a prior result.
- UI shows line-level and component-level allocations and clearly labels the output as a draft management-cost worksheet, not customs value, tax computation or approved accounting valuation.
- Rust unit fixtures have been added for exact allocation reconciliation, direct allocation, recoverable tax exclusion, invalid FX and separation of customs-value-only/pending items. They have **not been executed** under the product owner's deferred-testing instruction.

### Remaining prototype limitations

- No live bank/official FX rate provider; FX source, date, authorization and ratio are explicit user-entered evidence.
- No Ethiopian customs valuation or tariff engine; customs-value-only inputs are segregated and not calculated.
- No approved accounting policy engine for capitalization, inventory valuation or recoverability.
- No external OCR or document ingestion pipeline.
- Local SQLite remains unencrypted and the app remains synthetic-data-only.
- The calculation is a transparent prototype worksheet, not a practitioner-approved or production financial output.

These implementation notes describe code added, not test results, increment acceptance, or production readiness.


### Increment 2 source-evidence implementation added

- Added forward-only SQLite migration 10 for source-document BLOB storage linked to a client invoice or historical record.
- Added local PDF/JPEG/PNG attachment and retrieval commands. The backend validates file signatures against declared media type, extension, size, path/control characters, record/client ownership and duplicate hashes.
- Source documents retain SHA-256, filename, media type, size, actor, timestamp, record link and correlation ID. Update/delete triggers protect evidence rows; demo reset explicitly removes synthetic attachments and restores the guards.
- Invoice, historical-record and import-cost input UI exposes attachment listing and download from the record workspace.
- This is local prototype evidence storage only. It is not encrypted, malware scanning is not implemented, and real client documents remain prohibited.


### Increment 2/3 workflow hardening added

- Invoice capture now supports multiple structured source line items (item/category reference, description, quantity/unit, unit price, line amount, tax label and tax amount) while retaining source notation rather than silently recalculating tax.
- Invoice review requires at least one structured line with description and source line amount, or a preserved source line-description field, plus a source reference or attached original invoice.
- Invoice review requires an explicit reviewed financial-period reference and invoice date inside that period's start/end window.
- Period records require unique client-scoped references, valid ISO date windows, source/calendar policy references before review, and manager/administrator review before use.
- Historical records submitted for review require an explicit reviewed period and source evidence; dated transactions must fall inside the selected period. Tax-history rows preserve the tax period exactly as supplied.
- These checks prevent automatic tax-period inference and stop unassigned/out-of-window records from being treated as review-ready.


### Increment 4 workspace notification implementation added

- Added migration 11 for persistent firm/client/record-scoped notifications addressed to the current demo actor.
- Task assignment, reviewer assignment, submission for review, blocked, returned and completed events create idempotent notifications inside the same local database transaction as the task update.
- Task assignee and reviewer must map to active demo identities; this keeps the notification recipient and task workspace context explicit in the controlled prototype.
- The inbox is recipient-scoped, filters notifications through the active persona's client access, shows unread state, and routes to the linked task and client workspace.
- Read state is one-way; notification content is immutable and history cannot be deleted except through the explicit synthetic-demo reset.
- Notifications are in-app/local only. No email, SMS, push, background due-date scheduler or production identity delivery is implemented.


## Collective verification entry point

Implementation code for the current prototype increments 0–5 is now present on the bound branch. This is the point at which the product-owner-directed collective verification and test program may begin. No individual increment is accepted. The overall result remains pending until typecheck, domain tests, Rust tests, frontend build, Windows Tauri build, and the integrated acceptance workflow have completed and any defects have been corrected.


### Verification availability note

The collective verification gate is open after the Increment 5 prototype implementation. The connected GitHub workflow-run lookup returned no PR-triggered run for the current head at the time of inspection. No local repository archive/runtime was available through the connected GitHub interface. Therefore no test, typecheck, frontend build, Rust build or Windows Tauri build result is claimed here. The gate remains pending; the configured workflow remains the intended execution path.


## Cloud-first implementation record — 2026-11-24

**Branch:** `implementation/cloud-first-platform` (based on the then-current `main` head).

Implemented in this branch:
- New isolated Node.js 22+/TypeScript cloud API package under `server/`; the existing root React/Vite and Tauri application remains unchanged.
- PostgreSQL migration for users, tenants, memberships, revocable sessions, sync aggregates/inbox, append-only audit events and worker jobs.
- Transaction-local tenant/user context, active membership verification and PostgreSQL RLS policies on tenant-owned tables.
- Password hashing with scrypt, HMAC-SHA-256 short-lived access tokens, rotating refresh tokens, session revocation and route rate limits.
- Versioned idempotent sync push, base-version conflict results, monotonic pull cursor and durable event inbox.
- Separate worker process with row-lock claiming, bounded retry/backoff and explicit handler registration boundary.
- OpenAPI contract, API README, container image and initial Compose topology.
- PostgreSQL-backed integration tests and a separate CI job to compile and execute them.

Not yet evidenced as passing at the time of this record: CI build/integration results on this branch, full production hardening, complete role/permission matrix, email verification/MFA, invitations, device leases, object storage/document scanning, business-domain server handlers, desktop outbox/pull integration, conflict UI, TLS ingress, independent backups/restore drills and security assessment. The generic JSON aggregate is not authorized for high-risk financial or professional records. This branch is an implementation increment, not a production release.
