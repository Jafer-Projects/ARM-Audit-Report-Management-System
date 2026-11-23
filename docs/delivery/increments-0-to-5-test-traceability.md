# Increments 0–5 — Collective Test Traceability Matrix

**Binding scope:** `implementation/increments-0-to-5`, PR #1  
**Purpose:** Map every minimum screen-level and increment-level scenario in the Developer Implementation Specification to executable tests or an explicitly identified collective UI/platform verification.  
**Acceptance rule:** This matrix is a preparation and coverage-control artifact. It does not pass increments individually. Formal execution and the sole acceptance decision remain deferred until Increment 5 is implemented.  
**Status vocabulary:** `AUTOMATED` means a test directly exercises the stated behavior or its production helper; `PARTIAL` means only a related lower-level assertion exists; `PENDING` means an executable test or evidence is still required. A test being committed is not evidence that it has run or passed.

## 1. Increment-level acceptance scenarios

### Increment 0 — Desktop shell and local workspace (7)

| ID | Normative scenario | Test / evidence | Status |
|---|---|---|---|
| I0-01 | First launch shows synthetic-data warning and opens demo workspace without network access | Windows first-launch/offline UI acceptance; no automated UI harness yet | PENDING |
| I0-02 | Persona selector exposes exactly four demo personas and labels active persona | `commands.rs::set_demo_persona`; four-persona UI enumeration test still required | PARTIAL |
| I0-03 | Read-only reviewer cannot invoke create/update/archive commands | `commands.rs::read_only_reviewer_is_rejected_by_the_shared_mutation_guard`; extend to every mutating command | PARTIAL |
| I0-04 | Local note survives application restart with identical content and association | `db.rs::local_notes_survive_database_reopen`; full UI restart/association evidence pending | PARTIAL |
| I0-05 | Activity view shows corresponding event and actor | `db.rs::local_event_actor_and_scope_are_retained_for_material_mutations`; activity UI assertion pending | PARTIAL |
| I0-06 | Reset requires confirmation and restores only synthetic fixtures | Reset dialog + destructive-boundary integration test required | PENDING |
| I0-07 | No production login, live integrations or future-module dead ends | UI navigation/first-launch acceptance and static regression required | PENDING |

### Increment 1 — Client/company registration (10)

| ID | Normative scenario | Test / evidence | Status |
|---|---|---|---|
| I1-01 | Minimal Company profile saves as Draft and completeness is correct | Field validation unit test exists; command persistence and completeness/UI integration tests still required | PARTIAL |
| I1-02 | Optional TIN/contact/sector updates profile summary | Command update + summary UI integration test required | PENDING |
| I1-03 | Invalid email is flagged without losing other entered values | `commands.rs::client_field_validation_accepts_minimal_profile_and_rejects_invalid_fields`; dirty-form retention and command integration tests pending | PARTIAL |
| I1-04 | Duplicate name/TIN warns and never silently merges/overwrites | `commands.rs::duplicate_acknowledgement_is_required_only_when_candidates_exist`; database candidate matching/no-merge command test pending | PARTIAL |
| I1-05 | Search is firm-scoped; archived hidden by default and shown on request | Client query command integration tests required | PENDING |
| I1-06 | Update history contains changed fields and before/after values | `db.rs::client_change_history_stores_before_and_after_snapshots`, `client_change_history_is_immutable`; command-generated update snapshot test pending | PARTIAL |
| I1-07 | Reviewer can view but cannot mutate | `commands.rs::read_only_reviewer_is_rejected_by_the_shared_mutation_guard`; full command read/write permission matrix pending | PARTIAL |
| I1-08 | Stale row_version conflicts without losing user's values | `commands.rs::optimistic_row_version_guard_rejects_missing_or_stale_versions`; save-command conflict/input-retention integration test pending | PARTIAL |
| I1-09 | Mutation and duplicate acknowledgment each have one audit event in the same transaction | Existing client-history/event database tests cover stored records only; command transaction atomicity test pending | PARTIAL |
| I1-10 | Restart and offline operation preserve committed records | Note restart test exists; client/profile full restart test required | PARTIAL |

### Increment 2 — Product/category register and invoice capture (11)

| ID | Normative scenario | Test / evidence | Status |
|---|---|---|---|
| I2-01 | Product/category records are client-scoped; cross-client ID substitution is rejected | Persona assignment helper is tested; actual two-client record-ID substitution matrix pending | PENDING |
| I2-02 | Archived product remains visible on historical invoice | Product archive + historical invoice rendering integration test required | PENDING |
| I2-03 | Draft invoice saves with permitted missing fields and shows exact missing list | Draft-save and UI missing-field-list test required | PENDING |
| I2-04 | Invalid values do not erase other entered values | Form state retention/UI validation test required | PENDING |
| I2-05 | Duplicate invoice candidate is advisory; no automatic overwrite/merge | Invoice duplicate command + no-merge integration test required | PENDING |
| I2-06 | Source attachment metadata and association survive restart | Attachment command/reopen/read-back test required | PENDING |
| I2-07 | Unsupported or unsafe file type is rejected clearly | `commands.rs::source_document_validation_checks_content_name_mime_extension_and_size` and `source_document_type_uses_file_signature_not_declared_extension` | AUTOMATED |
| I2-08 | Preparer cannot mark own work reviewed | Assigned preparer/reviewer command integration test required | PENDING |
| I2-09 | Reviewer decision and preparer correction are distinct events | Review/correction event-separation integration test required | PENDING |
| I2-10 | Offline capture and restart preserve records | Invoice/product/attachment restart integration test required | PENDING |
| I2-11 | No VAT calculation or tax-compliance claim appears | Existing Node vocabulary test is a local constant, not UI implementation; UI copy regression required | PARTIAL |

### Increment 3 — Historical data and draft VAT workpaper (10)

| ID | Normative scenario | Test / evidence | Status |
|---|---|---|---|
| I3-01 | Historical data remains distinct from posted accounting data | Record-type/schema/command separation test required | PENDING |
| I3-02 | Import preview identifies unmapped columns and bad rows before commit | Import-preview parser/UI fixture test required | PENDING |
| I3-03 | Import provenance identifies file, sheet/row and mapping version | `db.rs::historical_import_batch_retains_source_content_and_mapping`; command row-provenance end-to-end test pending | PARTIAL |
| I3-04 | No row is silently dropped | Import reconciliation totals and rejection-reason test required | PENDING |
| I3-05 | Source-to-workpaper drill-down reconciles to included rows | Workpaper-to-source reconciliation integration test required | PENDING |
| I3-06 | No approved tax rule pack means no authoritative VAT calculation | Existing Node status vocabulary test only; production command/UI gate test required | PARTIAL |
| I3-07 | Tax rule version/effective date is stored with each calculated draft | Rule-pack versioned draft test required; authoritative calculation remains gated | PENDING |
| I3-08 | Input changes do not rewrite previously reviewed snapshots | Workspace version immutability exists; reviewed-input revision command test required | PARTIAL |
| I3-09 | Review notes and preparer corrections are distinct events | `db.rs::task_review_notes_cannot_be_deleted`; same-actor separation and command-level attribution test pending | PARTIAL |
| I3-10 | Offline restart preserves source references and draft workpapers | Historical import/workpaper restart integration test required | PENDING |

### Increment 4 — Staff tasks and review workflow (10)

| ID | Normative scenario | Test / evidence | Status |
|---|---|---|---|
| I4-01 | Inactive staff cannot receive new assignment | Staff/task command integration test required | PENDING |
| I4-02 | Reassignment retains prior assignee and reason | Immutable reassignment history test required | PENDING |
| I4-03 | Preparer cannot clear own review note | Review-note actor-separation command test required | PENDING |
| I4-04 | Task completion and professional approval are separate states | `commands.rs::task_lifecycle_requires_submission_before_completion`; UI/approval separation test pending | PARTIAL |
| I4-05 | Reviewer action is attributable and cannot be deleted | `db.rs::task_review_notes_cannot_be_deleted`; reviewer identity and timestamp assertions required | PARTIAL |
| I4-06 | Blocked task requires reason and appears in manager queue | Block reason + queue command/UI integration test required | PENDING |
| I4-07 | Overdue calculation uses stored due date and local display timezone | Deterministic clock/timezone boundary test required | PENDING |
| I4-08 | Cancelled task retains history and work product | Cancellation preservation integration test required | PENDING |
| I4-09 | Cross-client task access is rejected | Assigned-persona access helper is tested; actual two-client task command boundary test pending | PARTIAL |
| I4-10 | Offline task updates persist after restart; no sync status fabricated | Task restart + UI sync-claim regression required | PENDING |

### Increment 5 — Import-cost worksheet (9)

| ID | Normative scenario | Test / evidence | Status |
|---|---|---|---|
| I5-01 | No approved discovery brief means calculation cannot be activated | `commands.rs::import_cost_calculation_input_cannot_bypass_discovery_gate`; discovery decision/approval evidence test pending; preview command Gate 5-A integration test pending | PARTIAL |
| I5-02 | Each cost input traces to source and preserves original currency | Existing costing provenance validation; command persistence/read-back test required | PARTIAL |
| I5-03 | FX rate has source, date and authorization | New costing provenance negative tests in `import_cost.rs`; saved-input command test pending | PARTIAL |
| I5-04 | Allocation is reproducible and versioned | New quantity/weight/volume/value and deterministic remainder tests in `import_cost.rs`; persisted revision version test pending | PARTIAL |
| I5-05 | Missing source/rate blocks readiness instead of using guessed default | New source/FX negative tests in `import_cost.rs`; save-record readiness command test pending | PARTIAL |
| I5-06 | Input change creates a new revision and invalidates prior review | Calculation revision and reviewed-snapshot invalidation test required | PENDING |
| I5-07 | Reviewer sees exact submitted snapshot | Submitted calculation snapshot read-back/reviewer test required | PENDING |
| I5-08 | Calculation reproduces independently verified expected results | Synthetic arithmetic fixtures exist; practitioner-approved independent expected-result fixture remains required | PARTIAL |
| I5-09 | Optional selling-price analysis remains separate from statutory/audit result | Feature boundary/UI copy regression test required | PENDING |

## 2. Shared foundation and cross-increment regression

| ID | Requirement | Existing/new test location | Status |
|---|---|---|---|
| X-01 | Empty database migration and repeat migration | `db.rs::migration_is_idempotent_and_demo_is_seeded_once` | AUTOMATED |
| X-02 | Migration checksum mismatch is rejected | `db.rs::migration_checksum_mismatch_is_rejected` | AUTOMATED |
| X-03 | Failed/uncommitted transaction does not persist partial data | `db.rs::uncommitted_workspace_write_rolls_back_when_transaction_is_dropped` | AUTOMATED |
| X-04 | Foreign-key integrity rejects orphaned history | `db.rs::foreign_key_constraints_reject_orphaned_workspace_history` | AUTOMATED |
| X-05 | Append-only event/history/version protections | Existing event, client history, record version and review-note immutability tests | AUTOMATED |
| X-06 | Optimistic concurrency preserves current values | `commands.rs::optimistic_row_version_guard_rejects_missing_or_stale_versions`; workspace-record stale update and command-level preservation test pending | PARTIAL |
| X-07 | Role/permission matrix across every command | Current reviewer/client and assignment tests; full command matrix pending | PARTIAL |
| X-08 | Multi-client boundary across every read/write/attachment/history command | One invalid-client write test; comprehensive two-client substitution suite pending | PARTIAL |
| X-09 | UI keyboard, labels, help, loading/empty/error/conflict states | Browser-level test harness and Windows runtime evidence pending | PENDING |
| X-10 | Offline launch, crash/restart and write-failure recovery | Local note restart and transaction rollback tests; UI/fault-injection suite pending | PARTIAL |
| X-11 | Windows 10/11 runtime compatibility | CI Windows build is compile evidence only; representative device runtime evidence pending | PENDING |

## 3. Newly added executable test groups

The current commit adds:
- Command-level tests for client creation, draft state, audit/history creation, invalid email rejection, duplicate acknowledgment, stale-version conflict, reviewer mutation rejection, reviewed-period date boundaries, invoice review prerequisites, historical-import duplicate-row rejection, record allowlists/transitions and invalid client references.
- Import-cost tests for quantity, weight, volume and value allocation; deterministic largest-remainder rounding; fractional quantity/FX rounding; duplicate references; invalid quantities/prices/weights/currencies/provenance; unknown direct targets; unsupported allocation/treatment; negative components; and arithmetic overflow.
- Database tests for dropped-transaction rollback, foreign-key enforcement and actor/firm/target event scope.

These tests are included in the Rust test target and execute through `cargo test --manifest-path src-tauri/Cargo.toml`. They have not been reported as passing until a workflow run executes against the exact current branch head.

## 4. Collective acceptance evidence checklist

The acceptance report must include:
- Exact branch head SHA and PR #1 head SHA.
- Actual test counts, pass/fail/ignored counts and full logs for Node and Rust.
- Typecheck, lint/format, frontend build and Windows Tauri build outcomes.
- UI and manual Windows 10/11 runtime evidence.
- Migration/restart/offline/fault-injection evidence.
- Two-client authorization and role-command matrix results.
- All 57 scenario outcomes, with source/test/evidence links.
- Practitioner evidence for Gate 5-A and independently validated import-cost expected results.
- Explicit unresolved legal, security, tax, accounting and production gates.

**No individual increment is accepted.** One integrated result is recorded only after Increment 5 and the full collective test program. Any pending or failed required scenario blocks a ready/accepted outcome.
