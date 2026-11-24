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

The desktop now exposes a cloud connection panel with account registration, email verification, sign-in, MFA/recovery, session refresh/revocation and a manually initiated sync action. Access/refresh tokens remain in memory only and are not persisted in an OS credential vault. Sync requires an explicit user confirmation because the initial outbox backfill includes existing workspace records, including synthetic demonstration records. Do not connect a demonstration workspace to a production tenant.

This implementation is intentionally limited to the explicitly named `prototype.workspace_record` aggregate. The local client directory is not yet synchronized; a pull that encounters a workspace record referencing a missing local client stops without advancing the cursor rather than silently dropping the dependency. Conflict counts are recorded, but conflict inspection/resolution UI and retry/merge policy are not complete. It does **not** synchronize authoritative audit engagements, workpapers, reports, journals, accounting periods or evidence metadata through the generic JSON event path. Those domains use dedicated cloud operations; offline synchronization for those domain aggregates, client dependency synchronization, device identity/leases, durable conflict-resolution UI, local encryption/key protection and resumable evidence upload remain separate required work. Do not infer production-grade offline authorization from the prototype outbox.
