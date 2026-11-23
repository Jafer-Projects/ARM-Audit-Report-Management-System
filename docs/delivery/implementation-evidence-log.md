# Increments 0–5 — implementation evidence log

**Governing PR:** [#2](https://github.com/Jafermenza/ARM-Audit-Report-Management-System/pull/2)  
**Branch:** `implementation/increments-0-to-5`  
**Acceptance model:** one collective gate after Increment 5; no increment-level acceptance/pass declarations.

This log records engineering evidence while the integrated system is being built. Individual checks are continuous verification only.

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
- [x] Task review-note lifecycle added (Open → Responded → Cleared/Reopened), with no-delete/content-immutable guards, preparer response and reviewer disposition.
- [x] Submitted task content is protected from reviewer edits; task reassignment/reviewer change and cancellation require recorded reasons; task snapshots preserve revisions.
- [x] Import-cost discovery captures all 12 practitioner prompts and Gate 5-A supporting artifact references; cost inputs remain locked.
- [x] Append-only local activity event table and activity view added.
- [x] Offline/local-only status presented honestly.
- [x] Windows x64 CI workflow added for typecheck, domain tests, frontend build and Rust build.
- [x] Three Node domain contract tests executed locally: 3 passed.
- [x] SQLite in-memory smoke checks exercised immutable-history triggers and allowed task review-note transitions; foreign-key check returned no violations.
- [ ] Rust tests (including new migration, import, version-history and review-note tests), TypeScript typecheck, frontend build and Tauri Windows build remain unverified because no application workflow jobs have run; the available repository Actions runs report startup failure with zero jobs.

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
| 2026-11-23 | Initial PR #2 implementation series | 3 Node tests passed; SQLite DDL/trigger smoke checks passed | Rust/TS/build CI unverified; no increment accepted; no collective acceptance decision |
