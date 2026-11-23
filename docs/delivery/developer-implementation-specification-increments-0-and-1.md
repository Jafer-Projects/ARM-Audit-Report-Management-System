# Developer Implementation Specification — Increments 0 and 1

**Status:** Approved for controlled implementation with synthetic data only  
**Applies to:** Increment 0 (working shell and local workspace) and Increment 1 (client/company registration)  
**Authority:** This document operationalizes the Phase 0 approval and the Phase 1 delivery plan. If a broad baseline document is less specific, this specification governs these two increments. It does not authorize production, real client data, live posting, tax filing or regulated report issuance.

## 1. Purpose and implementation rule

The developer must be able to implement these increments without inventing business rules. Build the smallest complete vertical slice, not a collection of disconnected screens. Every user-visible state must correspond to persisted application state. All unknown or professionally governed behavior must be visibly marked as not verified, deferred or requiring review.

Use synthetic records and fictional identities. Do not load actual client, taxpayer, staff or engagement data.

## 2. Scope boundary

### Included
- Windows desktop application shell using the repository's Tauri 2 / Rust / React / TypeScript direction.
- Local-only synthetic demo workspace.
- Demo persona selection for usability demonstrations; this is not production authentication.
- Client/company list, guided create/edit flow, profile detail and completeness display.
- Local SQLite persistence, schema versioning, safe migrations and restart recovery.
- Local audit-event history for material profile changes.
- Visible local/sync status model, with sync disabled and clearly labelled as not connected in these increments.
- Automated unit, integration, UI and persistence tests.

### Explicitly excluded
- Cloud account registration, production login, MFA and password recovery.
- Real firm tenancy, production RBAC enforcement, invitations or subscriptions.
- Cloud sync, multi-device merge, offline authorization leases or server acknowledgments.
- Real client data, document uploads, OCR or external integrations.
- TIN validation against government systems.
- Accounting journals, VAT calculations, tax returns, audit opinions, report signing or filing.
- Staff/task management. A draft note in Increment 0 must not be represented as the full task module planned for Increment 4.
- Installer signing or public desktop distribution pending security and OS support gates.

## 3. Technical baseline and implementation decisions

### 3.1 Application structure
Use a modular monolith with clear boundaries:
- `app-shell`: navigation, window lifecycle, global status and error presentation.
- `demo-session`: synthetic persona selection and reset.
- `client-workspaces`: client list, profile, create/edit and completeness.
- `local-persistence`: SQLite access, migrations, transaction boundaries.
- `audit-history`: append-only local event recording and history view.
- `sync-status`: status presentation only; no network sync implementation in these increments.

UI components must not issue raw SQL. Use typed application services/repositories. Tauri commands expose narrow, validated operations; do not expose arbitrary filesystem access or generic SQL execution to the webview.

### 3.2 Dependency and runtime policy
Pin exact versions in lockfiles. Use maintained, supported releases at implementation time; document selected versions and licenses in the repository. Do not upgrade major dependencies implicitly. CI must run formatting, lint, typecheck, unit tests, integration tests and a production build. Do not claim CI passes unless the workflow actually completes successfully.

### 3.3 Local database
SQLite is the local source of truth for this prototype. Use transactions for each user operation. Enable foreign keys and appropriate durability settings. Apply numbered, forward-only migrations; each migration must be tested from an empty database and from the immediately previous schema. Never silently drop user data during migration.

Encryption is a security gate, not a checkbox. Before any real data or distributable build, select and test a maintained SQLite encryption implementation, document its license and threat model, and implement key creation, protection, rotation, backup, restore and loss/recovery behavior. The prototype may use synthetic data while the encryption spike is evaluated; the UI and documentation must not claim the prototype is production-encrypted. Never store database keys in source code, plain configuration or logs.

### 3.4 Local data location
Store the database and future document cache in the per-user application-data directory using the platform's supported application path APIs. Do not write business data beside the executable, into the repository, or to arbitrary user-selected paths by default. Restrict filesystem access to the minimum required directory. Exports are out of scope for Increments 0–1.

### 3.5 Date, number and identity conventions
- Persist timestamps as UTC ISO-8601 values; display in the user's local timezone.
- Store ETB amounts only when a later approved workflow defines amount semantics; no financial amount fields are part of client registration.
- Generate internal IDs as UUIDs. Do not use TIN, company name or sequential database IDs as primary keys.
- Preserve entered identity text. Do not normalize away meaningful punctuation or transliteration.
- English is the initial application UI. Amharic documents/OCR are later capabilities and are not implied by this increment.

## 4. Increment 0 — Screen and behavior specification

### 4.1 First launch
1. Show product name, prototype status and a concise notice: "Demonstration workspace — synthetic data only. Not connected to production."
2. Offer "Open demo workspace" and "Reset demo data".
3. Opening the demo creates or opens a local synthetic firm workspace and a small set of clearly fictional sample clients.
4. Reset requires confirmation, records that the demo dataset was reset, and recreates only the built-in synthetic dataset. It must not erase data outside the application's demo database.
5. No email/password prompt. Demo persona selection is clearly labelled as a simulation.

### 4.2 Demo personas
Provide these fixed personas:
- **Demo Firm Administrator** — can view and maintain synthetic client profiles.
- **Demo Engagement Manager** — can view synthetic client profiles and create/edit profiles for demonstration.
- **Demo Auditor** — can view and create/edit synthetic client profiles for demonstration.
- **Demo Read-only Reviewer** — can view profiles and history but cannot create, edit or archive.

These personas exist only to demonstrate screen affordances and basic command guards. They are not a substitute for production identity, server authorization, engagement assignment or a complete permission matrix. Every screen must show "Demo role" while this mode is active. The persona must not be selectable in a production build.

### 4.3 Navigation
Provide a simple left navigation:
- Home
- Clients
- Activity
- Settings / Demo

Home shows one primary next action ("Continue with clients"), local storage state and a clear prototype/synthetic-data indicator. Do not show disabled future modules as if they are available. A compact "Not in this prototype" list may identify Accounting, Audit engagements, Tax and Sync without clickable dead-end actions.

### 4.4 Local workspace and draft
The demo user can create a short local draft note associated with a synthetic client, save it, close/reopen the application and verify persistence. The note is a demonstration artifact only—not a task, workpaper, audit finding or approval. Required fields: title (1–120 chars), body (0–2,000 chars), optional client association. Draft status is always "Local draft". No assignment, due date, priority or completion workflow.

### 4.5 Activity
Show a chronological local activity list with actor persona, action, target, local event time and result. It must distinguish business events from application diagnostics. No edit/delete controls for audit events. The prototype may offer a controlled demo reset, but reset history must be recorded before the synthetic database is recreated or retained in a separate non-sensitive diagnostic log.

## 5. Increment 1 — Client/company registration specification

### 5.1 User journey
1. Open Clients.
2. See search, status filter, result count and "Add client".
3. Select Add client and complete a short guided form.
4. Save as Draft; required-field validation is shown inline and at the field.
5. Open the profile summary; completeness is shown with the exact missing fields.
6. Edit an existing draft; compare before/after values in activity history.
7. Archive a duplicate/test record only after confirmation. Archive is reversible in the prototype; it is not legal deletion.
8. Search by name or TIN text and reopen the record.

### 5.2 Client profile field dictionary

| Field | Type / limits | Required to save draft? | Rules |
|---|---|---:|---|
| Record kind | Enum: Company / Individual / Other organization | Yes | Drives labels only in this increment; does not assert legal classification |
| Legal or full name | Unicode text, 2–200 chars | Yes | Trim outer whitespace; reject blank; preserve internal punctuation and script |
| Trading/display name | Unicode text, 0–200 chars | No | Optional; if blank, show legal/full name |
| TIN | Text, 0–40 chars | No | Store exactly as entered after outer trim; no format or government validity claim; allow "Not provided" |
| Sector/activity | Controlled choice plus Other | No | Choices are editable reference data; Other requires a short description |
| Primary contact name | Unicode text, 0–160 chars | No | No identity verification implied |
| Contact email | Text, max 254 chars | No | Basic syntax check only; do not send verification email |
| Contact phone | Text, max 40 chars | No | Preserve entered international/local notation; no carrier validation |
| Address | Unicode text, max 500 chars | No | Free text; no geocoding |
| Internal reference | Text, max 60 chars | No | Firm-local label; uniqueness warning within demo firm only |
| Notes | Text, max 2,000 chars | No | No sensitive personal details in demo data |

System-managed fields: UUID, demo firm ID, record status (Draft / Active / Archived), completeness status, created/updated timestamps, created/updated demo actor IDs, row version and schema version.

Do not add registration number, legal form, branch, multiple TIN, tax regime, VAT registration, beneficial owner, bank details or identity-document fields until a practitioner demonstrates the need and the product owner approves definitions. Do not assume one TIN per legal entity or validate TIN length without authoritative rules.

### 5.3 Validation and duplicate handling
- Legal/full name is required for draft save.
- Email is optional; when present, apply conservative syntax validation and show a correction message without rejecting valid international domains.
- TIN is optional and opaque text. Never display "verified".
- Duplicate matching is advisory: normalized name plus exact trimmed TIN when present. Show possible matches; do not auto-merge or block save.
- A duplicate warning must link to the possible existing profile. User may continue after acknowledging the warning; the acknowledgment is audited.
- Search is limited to the active demo firm. Search legal/display name and TIN using literal text; no cross-workspace search.
- Validation errors must identify the field and corrective action. Never clear other valid fields after an error.

### 5.4 Completeness model
Completeness is not legal or tax readiness. In this increment:
- **Minimum profile complete:** record kind and legal/full name present.
- **Additional details missing:** TIN, sector or contact details absent; display as an informational checklist only.
- **Not verified:** always displayed for TIN and contact identity unless a future authoritative verification flow exists.
Do not label a profile "compliant", "approved client", "KYC complete" or "tax registered".

### 5.5 Status transitions
- Draft → Active: available only after required minimum profile fields exist and user confirms "Activate profile".
- Active → Archived: confirmation required; profile remains searchable when "Include archived" is enabled.
- Archived → Active: permitted to demo administrator/manager/auditor; event records restoration.
- No hard delete in the UI.
- Profile editing never erases prior values from history.
- A profile status is not an engagement acceptance, independence clearance or authorization to provide regulated services.

### 5.6 Multi-firm and branch boundary
The prototype has exactly one synthetic demo firm context at a time. The schema must still include firm/workspace ownership on each client row so later multi-tenant architecture is not blocked. Do not implement multi-firm switching, cross-firm sharing, branch hierarchies or client transfer in these increments. These require a separate design and authorization rules.

## 6. Minimal logical data model

Use UUID primary keys and foreign keys. Names below are logical contracts; physical naming may follow repository conventions if mappings are documented.

- `demo_firm(id, name, created_at)`
- `demo_actor(id, display_name, persona, is_demo, created_at)`
- `client(id, firm_id, record_kind, legal_name, display_name, tin_raw, sector_code, sector_other, contact_name, contact_email, contact_phone, address_text, internal_reference, notes, status, row_version, created_at, updated_at, created_by, updated_by, archived_at)`
- `client_change(id, client_id, firm_id, actor_id, action, changed_at, before_json, after_json, correlation_id)`
- `local_note(id, firm_id, client_id nullable, actor_id, title, body, status, created_at, updated_at)`
- `local_event(id, firm_id, actor_id, event_type, target_type, target_id, occurred_at_utc, correlation_id, payload_json, schema_version)`
- `schema_migration(version, applied_at, checksum)`

Constraints:
- Foreign keys enabled.
- Every client and note belongs to the single demo firm.
- Actor references must resolve to a demo actor.
- TIN is stored as text, never numeric.
- Audit/event records are append-only from application commands.
- JSON payloads are schema-versioned and must not contain passwords, secrets or unnecessary personal data.
- All create/update/archive operations and their event writes occur in one database transaction.
- Optimistic concurrency uses row_version; stale updates return a conflict and preserve both the current record and user's unsaved input.

## 7. Local audit event contract

Minimum event types:
- `demo_workspace_opened`
- `demo_data_reset`
- `client_created`
- `client_updated`
- `client_activated`
- `client_archived`
- `client_restored`
- `possible_duplicate_acknowledged`
- `local_note_created`
- `local_note_updated`
- `demo_persona_changed`

Each event includes event UUID, firm/workspace UUID, actor UUID, event type, target type/UUID, UTC timestamp, correlation UUID, schema version and minimal structured payload. For update events, store changed field names and before/after values for fields in this synthetic dataset. Do not log full documents, secrets or unnecessary personal data. If an operation fails, no success event is written; a diagnostic error may be logged separately without sensitive payloads.

## 8. Offline and synchronization behavior

For Increments 0–1 the application is intentionally local-only:
- All create/edit/archive and note actions work with no network connection.
- Show status "Saved on this device" after the local transaction commits.
- Show a persistent banner "Sync is not enabled in this prototype".
- Do not show "Synced", "Uploaded", "Cloud backup complete" or a fake pending queue.
- Reopening the app must recover committed data.
- A failed transaction must leave the previous committed state intact and show a retryable error.
- Simulate process termination after save in automated tests.
- Do not create an outbox until the actual sync contract is implemented; do not imply sync exists.

## 9. Screen-level acceptance criteria

### Increment 0
- First launch shows synthetic-data warning and opens the demo workspace without network access.
- Persona selector shows exactly the four demo personas and labels the active persona.
- Read-only reviewer cannot invoke create/update/archive commands; direct command tests must reject these operations.
- User creates a local note, restarts app and finds identical content and association.
- Activity view shows the corresponding event and actor.
- Reset requires confirmation and restores only synthetic fixtures.
- No production login, live integrations or future-module dead ends are present.

### Increment 1
- User creates a Company profile with only record kind and legal name; it saves as Draft and completeness is correctly shown.
- User adds optional TIN/contact/sector and sees the updated summary.
- Invalid email is flagged without losing other entered values.
- Duplicate name/TIN match warns but never silently merges or overwrites.
- Search returns only records in the demo firm; archived records are hidden by default and visible when requested.
- Update history shows changed fields and before/after values.
- Read-only reviewer can view but cannot mutate.
- Stale row_version update returns a conflict and preserves the user's input for resolution.
- Every successful mutation and duplicate-warning acknowledgment has one corresponding audit event in the same transaction.
- App restart and offline operation preserve all committed records.

## 10. Required automated test suites

- Unit: field validation, email syntax, completeness, duplicate candidate matching, status transitions, role-command guards, event payload schema.
- Persistence: migration from empty DB, migration checksum, rollback on failed transaction, foreign keys, restart durability, optimistic concurrency.
- UI: keyboard navigation, labels/help text, inline errors, empty/loading/error states, archived filter, duplicate warning and confirmation dialogs.
- Security: path traversal attempts against native commands, arbitrary SQL is not exposed, read-only command rejection, firm_id scoping, unsafe log payload checks.
- Resilience: offline launch, forced close after commit, disk/write failure simulation, corrupt/incompatible schema handling without silent deletion.
- Build: Windows x64 debug build and production build in CI. Installer packaging remains blocked pending OS/security decision.

## 11. Required developer deliverables

Before declaring either increment complete, provide:
1. Running Windows desktop prototype and reproducible developer setup instructions.
2. Schema diagram or migration definitions and migration test report.
3. Screen-flow evidence (screenshots or short recording) using synthetic data.
4. Test results for all acceptance criteria, including negative tests.
5. Dependency/version/license inventory.
6. Open defects and deferred decisions with owner and impact.
7. Practitioner demonstration script and feedback capture template.
8. Updated PRD traceability: each implemented requirement ID linked to screen, service, schema and test.

## 12. Decisions still requiring owner or qualified review

These do not block the local synthetic prototype unless stated:
- **Windows 10 support policy:** see the explicit product-owner decision request in the PR discussion. Do not claim general Windows 10 support or package for unsupported Windows 10 until lifecycle, edition and security-update eligibility are confirmed. This blocks the supported OS matrix and installer acceptance, not Windows 11/local prototype work.
- **Encryption library and key lifecycle:** blocks real data and distribution; synthetic spike may proceed.
- **Production identity/RBAC and full permission matrix:** blocks real multi-tenant data; demo persona guards are not production security.
- **Legal entity types, branch structure and multiple TINs:** collect through practitioner sessions before expanding the profile schema.
- **MRC definition, revolving VAT meaning, invoice classes and import-cost components:** block those respective accounting/tax workflows, not client registration.
- **Tax matrix:** not received or independently reconciled; no authoritative tax calculation may be implemented.
- **Controller/processor roles, cross-border hosting, retention, client exit/export and contracts:** block real client data.
- **RPO/RTO, backup/restore, support and incident response:** block production.
- **Amharic OCR and document language behavior:** separate discovery and benchmark; not part of the English UI prototype.

## 13. Definition of ready for the next increment

Increment 2 may begin detailed design only after Increment 1 is demonstrated and accepted by at least one practitioner, field definitions and exception cases are recorded, all critical acceptance tests pass, and the product owner confirms the next workflow. This does not require closing unrelated tax, licensing or production gates.
