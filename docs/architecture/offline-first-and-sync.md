# Offline-First Desktop and Synchronization Protocol

## 1. Goal and constraints

The Windows desktop must support substantive assigned work without an active internet connection. Local storage is a working replica, not a license to bypass firm permissions or professional approvals. PostgreSQL is authoritative for shared cloud state; SQLite is authoritative for the user's unacknowledged local work until accepted by the server.

Do not implement synchronization as generic database replication or last-write-wins merging.

## 2. Local persistence

- Use SQLite transactions for every business operation.
- Persist the business change and corresponding outbox event atomically.
- Use stable UUIDs generated on the device for locally created records.
- Use explicit schema and event versions.
- Enable appropriate SQLite durability settings and test crash recovery.
- Evaluate SQLCipher or another maintained encryption approach; protect keys using Windows DPAPI/CNG or an appropriately designed key hierarchy.
- Keep database, WAL, temporary files, exports and local document cache inside protected application data locations.
- Avoid storing secrets in plain text or relying solely on obscurity of file paths.
- Ensure backup/restore includes a consistent database and document manifest.

## 3. Outbox event envelope

Each event should include:
- event_id (globally unique UUID).
- tenant_id and workspace_id (context, not authorization proof).
- device_id and authenticated actor_id.
- aggregate_type and aggregate_id.
- aggregate_version/base_version.
- event_type and schema_version.
- occurred_at_device and received_at_server.
- idempotency_key.
- correlation_id and causation_id where applicable.
- payload with only necessary data.
- integrity metadata.

Do not use the device clock as the sole ordering authority. The server records receipt time and validates aggregate versions and domain rules.

## 4. Delivery semantics

Use at-least-once delivery with idempotent server processing. Exactly-once network delivery cannot be assumed. The server inbox/deduplication table must make repeated event submissions safe. A client marks an event acknowledged only after receiving a durable server acknowledgment.

The server should return one of:
- accepted with canonical version and acknowledgment;
- duplicate already accepted;
- rejected with stable validation reason;
- conflict requiring resolution;
- deferred because an online-only authorization or approval is required.

## 5. Conflict policy by data type

| Data | Policy |
|---|---|
| Draft narrative | Version-aware merge or user resolution |
| Posted journal | Immutable; correction by reversal/replacement |
| Unposted journal draft | Version conflict; compare and resolve |
| Audit workpaper | Preserve versions; reviewer-controlled resolution |
| Evidence document | Immutable versions; new upload creates a new version |
| Client response | Preserve each submission and revision |
| Task state | Enforce valid state transitions |
| Tax return | Versioned draft; approval/submission locks controlled |
| Government filing | Idempotent submission attempt and official receipt tracking |
| Notification | Deduplicate by event/reference |

Never silently discard a rejected or conflicting local change. Keep it visible and recoverable until an authorized resolution is recorded.

## 6. Ordering and dependencies

Use aggregate-level ordering and explicit dependency references. A journal line must not be synchronized before its journal/period/account references exist. A workpaper link must not be accepted before its referenced engagement and evidence metadata are known. Batch events where useful, but preserve dependency order and bounded batch sizes.

## 7. Document synchronization

- Create document ID and version metadata locally.
- Calculate a cryptographic hash before upload.
- Upload using resumable multipart/chunked transfer.
- Verify server-side checksum and content length.
- Scan/validate file on server intake.
- Commit metadata and mark document as received only after durable object storage and metadata confirmation.
- Preserve original filename as metadata, not as a trusted storage path.
- Retry interrupted transfers without creating duplicate logical versions.
- Distinguish local-only, queued, uploading, uploaded, received, rejected and superseded states.

A document stored locally is not “received by the firm cloud” until server acknowledgment.

## 8. Offline authorization and revocation

A disconnected device cannot learn about a newly revoked user or device immediately. Therefore:
- Issue signed, bounded offline authorization leases.
- Cache only assigned data and minimum required permissions.
- Enforce lease expiry locally.
- Require online checks for high-risk actions such as final report issuance, final tax filing, role changes, payment actions and sensitive exports where appropriate.
- Support remote device revocation and key rotation.
- Define maximum offline duration based on risk and user needs.
- On lease expiry, restrict access to protected workspace data while preserving safe export/recovery paths for locally created work.
- Clearly communicate pending/restricted states.

Do not promise immediate remote wipe or immediate revocation while a device is disconnected.

## 9. Sync conflict workspace

Show the user:
- Record and field affected.
- Local and server versions.
- Actor and timestamps.
- Reason for conflict.
- Available safe actions.
- Required reviewer/manager approval.

Resolution must produce a new auditable event; it must not rewrite history.

## 10. Resilience and tests

Test network loss at every stage, duplicate events, process termination, power loss, partial uploads, expired tokens, revoked devices, schema upgrades, clock skew, dependency ordering, server failover and restore. Run long-duration offline tests and randomized fault injection. Measure queue age, retry counts, conflict rates and synchronization lag.

## 11. Protocol evolution

Every event and payload is versioned. New desktop releases must handle supported older server responses and migration paths. Define minimum supported desktop version and a safe upgrade gate. Never deploy a server schema change that makes supported offline clients unable to synchronize without a documented compatibility strategy.


## Cloud synchronization API implementation status

The first server contract is exposed at `POST /v1/sync/events` and `GET /v1/sync/events`, documented in `server/openapi.yaml`. Push accepts bounded batches of version-1 `prototype.workspace_record` `aggregate.upsert` envelopes, verifies the authenticated user's active membership in the selected tenant, uses tenant-scoped transactions and advisory locking, checks aggregate base versions, and persists idempotency outcomes. Inbox outcomes receive a server-side monotonic sequence used as the pull cursor. Pull exposes accepted, conflict and rejected outcomes so clients can advance without silently skipping conflict records. Repeated idempotency keys return the recorded result; stale base versions return explicit conflicts. Pushes are serialized per tenant to keep cursor commit order consistent.

This is an initial transport and persistence contract, not the complete desktop sync coordinator. The current client does not yet write business changes and outbox rows atomically, manage device identities/offline leases, upload resumable documents, negotiate protocol capabilities, apply pull events locally, or present a conflict-resolution workspace. Domain-specific event validators and authorization must replace generic JSON payload acceptance before authoritative financial or professional records use this path. Never mark a desktop event synchronized until the server acknowledgment is durably received and applied.


## 12. Implemented prototype boundary and current client integration

A SQLite migration now creates a durable `sync_outbox`, pull `cloud_sync_inbox`, per-aggregate server-version state and tenant cursor. A trigger atomically captures `workspace_record` events and existing records are queued for initial synchronization. Tauri commands expose configuration, outbox batching, acknowledgements, pull application and sync status. The React/TypeScript `src/cloudSync.ts` service connects these commands to the authenticated cloud sync API over HTTPS.

The desktop now exposes a cloud connection panel with account registration, email verification, sign-in, MFA/recovery, session refresh/revocation and local sync-status visibility. Access/refresh tokens remain in memory only and are not persisted in an OS credential vault. The visible sync action is disabled and the native sync configuration command rejects the seeded ARM Demonstration Firm. This hard block prevents the current synthetic workspace and its initial outbox backfill from being paired with a production tenant.

This implementation is intentionally limited to the explicitly named `prototype.workspace_record` aggregate. The local client directory is not yet synchronized; a pull that encounters a workspace record referencing a missing local client stops without advancing the cursor rather than silently dropping the dependency. Conflict counts are recorded, but conflict inspection/resolution UI and retry/merge policy are not complete. It does **not** synchronize authoritative audit engagements, workpapers, reports, journals, accounting periods or evidence metadata through the generic JSON event path. Those domains use dedicated cloud operations; offline synchronization for those domain aggregates, client dependency synchronization, device identity/leases, durable conflict-resolution UI, local encryption/key protection and resumable evidence upload remain separate required work. Do not infer production-grade offline authorization from the prototype outbox.


## 13. Desktop-to-cloud workflow parity audit (source inspection: 2026-11-24)

This section records the source-level relationship between the existing React/Tauri desktop workflows and the Node.js/PostgreSQL cloud API. It is an implementation inventory, not a declaration that the migration is complete.

### Architecture binding

The desktop must use the Node.js API as the authoritative business-operation boundary whenever online. Tauri/Rust remains responsible for native capabilities and the local SQLite repository. SQLite is a durable offline workspace/cache/outbox, not an independent production business authority. The React UI must call a typed application service that selects the cloud command when connected and persists a permitted offline command plus outbox event when disconnected. Both paths must enforce the same domain contract; the server must revalidate every command, membership, permission, record relationship and state transition before committing to PostgreSQL.

Do not make a screen "cloud migrated" merely by adding a sign-in button or by syncing a generic JSON snapshot. A workflow is migrated only when it has: (1) a server-owned domain model and migration, (2) an authenticated tenant-scoped API command/query, (3) desktop typed client integration, (4) local offline policy and durable outbox/inbox where allowed, (5) conflict/rejection and retry UX, (6) audit history, (7) tests for both online and offline paths, and (8) an explicit migration/import strategy for existing SQLite data.

### Existing desktop workflow inventory

| Existing desktop workflow/source | Current source of truth | Cloud source implementation | Desktop-to-cloud status | Required work |
|---|---|---|---|---|
| Workspace overview and demo persona (initialize, setPersona) | SQLite demo_firm, demo_actor, app_setting; persona-based demo authorization | Registration, login, tenant context and membership exist | **Not migrated** | Replace demo persona as production identity/authorization. Load tenant profile, current user, memberships, permissions and server health from Node API. Retain demo mode only as isolated synthetic-data mode. |
| Client/company register (clients, saveClient, duplicate search, archive, activate, restore, change history) | SQLite client, client_change | No client CRUD, duplicate-search, lifecycle or client-history API/table in cloud migrations | **Not implemented in cloud** | Add tenant-owned client model, migrations, server validation, duplicate rules, lifecycle commands, immutable history, typed desktop adapter and offline client-directory sync. |
| Categories, products and invoice capture (records with category, product, invoice; invoice duplicate search and line drafts) | SQLite workspace_record and JSON payload | No category/product/invoice domain tables or routes | **Not implemented in cloud** | Model category hierarchy, product references, invoice headers/lines, source-text amounts, currency, periods and duplicate detection. Preserve the prototype's explicit rule that captured source text is not automatically a tax/accounting calculation. |
| Historical periods, records, CSV import and VAT workpapers | SQLite workspace_record, historical_import_batch; import rows retained as source-derived records | No period, historical import, VAT workpaper, tax rule version or import-batch API | **Not implemented in cloud** | Add checksum/idempotent import batches, row-level validation and provenance, period ownership, draft/review transitions and versioned tax-rule boundary. VAT calculation remains unavailable until approved rules and independently verified fixtures exist. |
| Staff, task assignment and review notes (staff, task, add/respond/disposition review note) | SQLite workspace_record, task_review_note; local demo persona and assignee strings | Tenant memberships and RBAC exist, but no staff directory, task, assignment or review-note domain APIs | **Not implemented in cloud** | Bind staff to real tenant memberships; add task ownership, assignment, workflow transitions, reviewer/preparer separation, review-note lifecycle, audit history and offline authorization checks. Do not map demo persona strings to cloud permissions. |
| Import-costing worksheet and preview | SQLite workspace_record; calculation in Rust import_cost.rs | No cloud import-cost domain, calculation service, versioned FX source or calculation fixtures | **Not implemented in cloud** | Port the approved calculation contract to a shared/server-owned implementation, add source-backed FX and allocation evidence, independent expected-result fixtures and API; preserve draft/management-only limitations. |
| Local notes | SQLite local_note | No cloud notes model or API | **Not implemented in cloud** | Decide whether notes are private device drafts or shared tenant notes. If shared, add tenant/client/record scope, permissions, retention, audit and sync; never upload private notes implicitly. |
| Source documents (attachSourceDocument, list/read) | SQLite source_document and Tauri filesystem operations | S3-compatible upload/list/read and malware scan APIs exist for engagement/workpaper/report documents | **Cloud backend exists; desktop not integrated** | Add desktop upload/download client, streaming/resumable transfer, scan-state UI, permission checks, checksum verification and local encrypted cache. Resolve attachment relationship mismatch: desktop accepts a generic recordId; cloud currently requires engagement, workpaper or report. |
| Notifications | SQLite workspace_notification | No tenant notification query/read API | **Not implemented in cloud** | Add server-generated notification model, recipient/tenant authorization, unread/read operations and desktop pull/push integration. |
| Activity history and record versions | SQLite local_event, workspace_record_version, client history | Cloud audit_events and audit_domain_history are written by selected server operations, but there is no complete desktop-facing audit/history query contract | **Partial backend only; desktop not integrated** | Expose authorized, paginated audit/history queries with actor, before/after, correlation and provenance; align local version display to server canonical versions. |
| Audit engagements, workpapers and reports | Desktop initial screens currently use generic records; no dedicated engagement/workpaper/report UI | Node APIs and PostgreSQL tables exist for create/list and selected state transitions | **Cloud backend only; no desktop workflow integration** | Build dedicated desktop screens and typed API operations. Complete required update/review/approval/sign-off/issuance/lock rules and traceability before replacing prototype records. |
| Accounting accounts and journals | No authoritative accounting ledger UI in the current desktop sections | Node APIs exist for accounts, journal creation, posting, reversal and list/detail | **Cloud backend only; no desktop workflow integration** | Add account and journal screens, period controls, balanced-entry validation, immutable posting/reversal, decimal precision, concurrency and independent reconciliation fixtures. Ledger, trial balance, period close, bank reconciliation and financial statements are not established by these endpoints. |
| Identity, MFA, recovery and tenant membership | Desktop cloud panel only; normal workspace still initializes local demo data | Node auth, email verification, recovery, MFA, membership, invitations and RBAC endpoints exist | **Partial integration** | Make session/tenant/permission context part of the application shell and every business service. Add invitation acceptance UI, tenant/member/role administration UI, device registration/revocation, OS credential-vault storage and token refresh/revocation lifecycle. The existing panel alone does not authorize or route the desktop's business commands through Node. |
| Synchronization | SQLite outbox/inbox currently captures only prototype.workspace_record | Node sync schema accepts only prototype.workspace_record | **Prototype only** | Implement domain-specific commands/events and read models for every approved offline workflow. Add device identity/lease, protocol negotiation, tombstones, dependency ordering, rejected-event correction, conflict inspection/resolution, retry/backoff, partial-batch handling, cursor safety, retention and old-client compatibility. Never sync accounting/audit approvals as arbitrary JSON snapshots. |
| Demo reset | SQLite reset_demo_data | No cloud reset endpoint | **Intentionally local-only** | Keep this unavailable in cloud/production mode. Ensure demo data cannot be paired or uploaded to a real tenant. |

### Source-level defects and mismatches found

1. src/services/desktop.ts invokes respond_task_review_note, and src-tauri/src/commands.rs defines the command, but src-tauri/src/lib.rs did not register it in Tauri's invoke handler. The command is now registered on this branch. This is a concrete existing desktop action that otherwise failed at runtime.
2. All business methods in src/services/desktop.ts still call Tauri commands. No existing client, product, invoice, historical import, task, review-note, costing, notes, notification, client-history or record-history method routes to the Node API.
3. The prototype sync service can send only prototype.workspace_record, but the panel's sync action is now disabled for the seeded demonstration workspace and the native sync configuration command rejects that firm. The sync code does not synchronize the client, local_note, task_review_note, historical_import_batch, source_document or workspace_notification tables.
4. The generic pull implementation depends on a client row already existing locally. There is no client-directory sync/bootstrap path; the code stops on a missing client rather than advancing the cursor, which is safer than silently losing the relationship but leaves sync incomplete.
5. The desktop's recordId document attachment contract does not match the cloud document API's engagement/workpaper/report-only attachment contract.
6. The desktop uses local demo personas for task and reviewer actions. Those are not a substitute for server-enforced tenant membership, permission checks, assigned-user identity or separation of duties.
7. The desktop UI does not currently expose cloud audit/accounting screens despite the server endpoints. Conversely, several existing desktop screens have no corresponding cloud API or PostgreSQL model.
8. Cloud identity calls are made from a separate panel. Closing it attempts to revoke the current session, and tokens are held in component memory; the desktop application shell and its business operations do not consume that session as their active authorization context.
9. The server worker has handlers for email delivery and document scanning plus a no-op health-check job. OCR, report generation/export, historical imports, reconciliation, tax jobs, notifications and other planned handlers are not implemented merely because job infrastructure exists.
10. The current source does not demonstrate a production data migration from existing SQLite firms/clients/records/documents into tenant-owned PostgreSQL data, nor a reconciliation report proving row counts, references, checksums and audit-history preservation.

### Required migration order and completion gate

1. **Identity and shell binding:** make Node session, tenant, membership, permission and device context the only production identity authority; separate demo workspace mode from cloud workspace mode.
2. **Core tenant directory:** migrate clients and client history first, including bootstrap/pull ordering and referential integrity.
3. **Initial increments 0–5:** migrate category/product/invoice, historical/period/import/VAT workpaper, staff/task/review-note, import-costing, notes, notifications, activity and versions. Keep each workflow's actual business semantics and source evidence; do not flatten all of them into generic JSON.
4. **Evidence:** integrate desktop document upload/download with the cloud scanner and local encrypted cache.
5. **Professional domains:** connect dedicated audit engagement/workpaper/report and accounting account/journal screens to the Node APIs; complete domain rules before allowing sensitive offline mutations.
6. **Synchronization:** provide a versioned per-domain sync contract with server validation, idempotency, dependency order, durable acknowledgements, conflicts and recovery UX. Sensitive approvals, journal posting and report issuance should be server-online operations unless a separately approved offline authorization model exists.
7. **Migration and rollout:** implement a one-time, resumable, auditable SQLite-to-PostgreSQL import with dry-run, mapping, duplicate report, source checksums, relationship checks, reconciliation totals and rollback/forward-repair plan. Do not auto-upload seeded demo data.
8. **Verification:** create workflow-level parity tests that exercise the same use cases online, offline, after reconnect, across two devices, after permission revocation, and after partial failure. The desktop and server tests must assert the same state transitions and authorization outcomes.

**Cloud-first acceptance rule:** every desktop business action must be classified as (a) cloud-online authoritative, (b) offline-capable with an explicit domain sync contract, or (c) local-only by design (for example, private draft or demo reset). No unclassified local business write may be treated as production data. A workflow is not migrated until the UI uses the Node API/application service, PostgreSQL owns the canonical state, and the offline contract and migration evidence are tested. This parity gate is separate from the deferred collective acceptance decision for Increments 0–5 and does not pass or accept any increment.
