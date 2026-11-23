# Window-by-window and user-by-user workflow specification — Increments 1–5

**Status:** Detailed delivery specification for practitioner validation; Increment 1 inherits the approved normative contract. Increments 2–5 are implementation-ready workflow proposals only where the source requirements are explicit. Items marked **Practitioner decision required** must be demonstrated and approved before the affected behavior is coded as a business rule.

**Applies to:** ARM Windows desktop application, local-first workflow, synthetic data during controlled development.

**Companion documents:**
- [Developer implementation specification — Increments 0 and 1](developer-implementation-specification-increments-0-and-1.md)
- [Phase 1 incremental development and feedback plan](phase-1-incremental-development-and-feedback.md)
- [Auditor pain points and initial workflow requirements](../product/auditor-pain-points-and-initial-workflow-requirements.md)
- [Roles, permissions and workflows](../product/user-roles-and-permissions.md)

---

## 1. How to use this specification

This document translates each increment into visible application windows and role-specific journeys. Developers should be able to identify:
1. which window opens, from where and for whom;
2. what information the user sees and enters;
3. which actions are available, unavailable or require confirmation;
4. what is saved locally and what history is recorded;
5. how validation, empty, error, conflict and offline states behave;
6. what must be tested before the increment is demonstrated.

A window is a meaningful work surface, not necessarily a separate operating-system window. Use a consistent application shell with a content area, contextual side panel or modal only when it reduces navigation. Avoid a screen made of disconnected cards and unexplained action buttons. Each work area must answer: **Where am I? What am I working on? What is the next safe action? What has been saved?**

### 1.1 User categories and authority

The following are product roles for the intended system. During Increments 1–5, only the existing four demo personas are available in the prototype. Do not represent the demo selector as production authentication or complete RBAC.

| Intended user | Main responsibility | Typical access boundary |
|---|---|---|
| Firm administrator | Firm setup, user administration, client workspace administration, configuration and operational oversight | Firm-wide administration subject to ownership, contracts and privileged-action controls |
| Engagement manager / accounting manager | Organize client work, assign staff, monitor progress, review completeness and coordinate work | Assigned firm clients/engagements; no unrestricted platform administration |
| Auditor / accountant / preparer | Maintain client records, capture source transactions, prepare schedules and complete assigned work | Assigned clients and work items; may prepare but cannot self-approve restricted work |
| Reviewer / audit manager / partner | Review evidence, calculations, workpapers and conclusions; clear or return review notes | Explicitly assigned review scope; approval authority depends on engagement and professional authorization |
| Client contact | Supply records, answer requests, review permitted information and communicate with the firm | Only their own organization and explicitly shared requests/documents; not part of the desktop prototype increments |
| Platform support / operator | Operate the service and provide controlled technical support | No default access to client business content; exceptional access must be authorized, time-limited and audited |

**Demo personas currently defined by the Increment 0/1 contract:**
- Demo Firm Administrator
- Demo Engagement Manager
- Demo Auditor
- Demo Read-only Reviewer

In the current prototype all three non-read-only personas can demonstrate client profile maintenance. This deliberately simplified guard is not the final production permission matrix. Later increments may demonstrate additional role differences, but the UI must show a persistent **Demo role — simulated permissions** label. Never imply that a locally selected persona is secure authentication.

### 1.2 Common window layout and behavior

Every work window must consistently provide:
- **Top application bar:** ARM name, current demo firm/workspace, active demo persona, local save indicator, and prototype/synthetic-data banner.
- **Primary navigation:** Home, Clients, and Activity in the early increments. Later modules appear only when that increment is actually enabled; no dead-end future-module buttons.
- **Page heading:** plain-language title, one-sentence purpose and primary next action.
- **Context trail:** e.g. Clients → Client name → Invoices → Invoice draft.
- **Main work area:** readable list, form, worksheet or review queue; avoid presenting every possible field at once.
- **Contextual help:** explain the business meaning of fields with a short example; do not use unexplained accounting software terminology.
- **Save state:** distinguish unsaved changes, saving, saved on this device, save failed and (only in a future sync increment) server acknowledged.
- **Footer/status region:** current local-only status and actionable errors. Do not imply cloud backup or synchronization exists.
- **Keyboard and accessibility:** visible labels, logical tab order, focus management, keyboard-operable controls and screen-reader names.

### 1.3 Common interaction contract

- Save operations are explicit and transactional. A success message appears only after local commit.
- Closing a dirty form asks whether to save, discard or continue editing. Never silently discard.
- Validation errors appear beside the affected field and in a concise summary; other valid inputs remain intact.
- Destructive or consequential actions require a confirmation that names the target and consequence.
- Archive is not deletion. No hard-delete action is provided for business records.
- Every material create, update, archive, restore, review, approval or export action records actor, target, timestamp and outcome in an append-only event history.
- The application works offline for the approved local workflows. During Increments 1–5, show **Saved on this device — sync not enabled**. Do not invent an outbox, pending upload state or cloud acknowledgment.
- Errors must explain what the user can do next. Technical diagnostics must not expose secrets or unnecessary personal data.
- Never use a disabled button as the only explanation that a workflow is unavailable. Explain the missing prerequisite or that the capability is not in this increment.

---

# 2. Increment 1 — Client and company registration

**Outcome:** A practitioner can create, find, review and maintain a basic client profile in the synthetic firm workspace.

**Normative implementation source:** The complete field dictionary, validation, completeness, duplicate, archive/restore, persistence, event and test rules are in [Developer implementation specification — Increments 0 and 1](developer-implementation-specification-increments-0-and-1.md). This section explains the user-visible window sequence and role experience; it does not replace that contract.

## 2.1 User journey

Clients → Add client → Enter identity → Save draft → Review completeness → Activate profile (if ready) → Return to client list.

### Window I1-W01 — Client register

**Purpose:** Let a practitioner find a client and understand the current profile state before opening it.

**Visible content**
- Page title: **Clients**
- Search input: name, display name or TIN text.
- Status filter: All active / Draft / Archived; archived excluded by default.
- Result count and a clear empty-state message.
- Table/list columns: client name, record kind, TIN (masked only if later policy requires; for synthetic demo show entered text), sector, profile status, completeness, last updated.
- Primary action: **Add client**.
- Row action: **Open profile**. Archive/restore actions appear only where permitted and should not clutter every row.

**User behavior**
- Firm administrator: browse all synthetic demo-firm records, create, edit, activate, archive and restore.
- Engagement manager: same demonstration access in this prototype; production scope will be limited to authorized firm clients.
- Auditor/accountant: create and maintain client profile in the demo; later production scope will depend on assignment and firm policy.
- Read-only reviewer: search, filter and open; no create/edit/archive controls. Direct command calls must also reject mutation.

**States**
- No clients: explain why and show Add client if persona permits.
- No search results: preserve search text and offer clear search.
- Archived results: visually distinguish archived and prevent accidental edits without explicit restore.
- Local database unavailable/write failure: explain that changes cannot be saved; do not show a false success.

### Window I1-W02 — Add client: identity

**Purpose:** Capture the minimum identity information without overwhelming the user.

**Fields**
- Record kind: Company / Individual / Other organization (required).
- Legal or full name (required; 2–200 characters).
- Trading/display name (optional).
- TIN (optional opaque text; 0–40 characters; never claim verified).
- Sector/activity (optional controlled choice; Other requires description).

**Behavior**
- Short helper text explains legal/full name versus display name.
- The user may save a draft with only record kind and legal/full name.
- TIN is stored as text, not a number; no invented length, checksum or government validation.
- Duplicate candidates are advisory only. Show possible matches and let the user open them in a separate view. If user continues, record acknowledgment.
- No registration certificate, legal form, tax regime, branch, bank or beneficial-owner field is added until approved.

**Role behavior**
- Demo administrator, manager and auditor: may save a draft.
- Read-only reviewer: cannot enter this flow; if opened by a deep link, show access-denied explanation and no mutation path.

### Window I1-W03 — Add client: contact and address

**Fields**
- Primary contact name (optional).
- Contact email (optional; conservative syntax check only; no verification email).
- Contact phone (optional; preserve local/international notation).
- Address (optional free text).
- Internal reference (optional firm-local label; advisory uniqueness warning).
- Notes (optional, 0–2,000 characters; demo-only and no sensitive personal data).

**Behavior**
- All fields remain optional for draft save.
- Field help states that entering a contact does not verify their identity or authority.
- Invalid email shows a correction message without clearing other fields.
- Back/next navigation preserves entered data.
- On final save, the profile and its audit event commit in one transaction.

### Window I1-W04 — Client profile summary

**Purpose:** Give the practitioner a single authoritative view of what is known and missing.

**Sections**
1. Identity: record kind, legal/full name, display name, TIN and verification state.
2. Sector and activity.
3. Contact and address.
4. Firm reference and notes.
5. Profile status and completeness checklist.
6. Recent changes/history.

**Actions**
- Edit profile (mutable personas only).
- Activate profile when minimum required fields are present; show confirmation.
- Archive profile (confirmation required).
- Restore archived profile (confirmation required).
- View activity/history.

**Completeness labels**
- Minimum profile complete: record kind and legal/full name exist.
- Additional details missing: informational list only.
- Not verified: TIN and contact identity remain unverified.
- Never use compliant, KYC complete, approved client or tax registered.

**Status rules**
- Draft → Active after minimum fields and user confirmation.
- Active → Archived after confirmation.
- Archived → Active through restore.
- No hard delete.

### Window I1-W05 — Duplicate candidate comparison

**Purpose:** Help a user recognize a possible existing client without merging records automatically.

**Shows**
- Current draft identity.
- Candidate name, record kind, TIN text (if present), status and last updated.
- Why it matched: normalized name and/or exact trimmed TIN.
- Actions: open existing profile, return to draft, or acknowledge and continue.

**Rules**
- No automatic merge, overwrite or block.
- Acknowledgment is recorded as an event.
- If a user believes records are the same legal entity, record a follow-up issue; merging is not in this increment.

### Window I1-W06 — Client activity history

**Purpose:** Make changes explainable.

**Shows**
- Actor demo persona, action, timestamp, target and result.
- For profile updates: changed field names and before/after values from the synthetic dataset.
- Duplicate acknowledgment, activation, archive and restore events.
- Clear distinction between business activity and technical diagnostics.

**Rules**
- No edit/delete controls for history.
- History must survive application restart.
- Reset of synthetic demo data must be explicitly identified as a demo reset, not routine deletion.

## 2.2 Increment 1 acceptance walkthrough by user

| User | Scenario | Expected result |
|---|---|---|
| Demo Firm Administrator | Creates a company using only record kind and legal name | Draft saved; minimum completeness correct; event recorded |
| Demo Engagement Manager | Adds contact and TIN details to the draft | Profile updates; old/new values visible in history |
| Demo Auditor | Searches for a client by TIN text and acknowledges a possible duplicate | Search is firm-scoped; no automatic merge; acknowledgment event recorded |
| Demo Read-only Reviewer | Opens profile and activity | Can inspect; all mutation routes are unavailable and command-guarded |

**Increment 1 exit:** all existing normative acceptance criteria pass, including offline restart durability, validation preservation, duplicate handling, event atomicity and stale-row conflict behavior.

---

# 3. Increment 2 — Product/category register and invoice capture

**Outcome:** An auditor/accountant can maintain a client-scoped product catalogue and capture a controlled batch of local sales or purchase invoices with source references and review states.

**Important scope boundary:** Local sales and purchase invoice capture is the initial target. Import/export invoice classes are not enabled until practitioners validate actual document examples and required fields. The source proposal lists “MRC” but does not define it. It must remain an unresolved field, not be expanded or assigned a guessed meaning.

## 3.1 Main journey

Open client → Products & invoices → Maintain categories/products → Open invoice register → Add invoice → Attach/select synthetic source evidence → Validate → Save draft → Review queue → Mark reviewed or return for correction.

### Window I2-W01 — Client work hub

**Purpose:** Establish the client context before creating a catalogue item or invoice.

**Shows**
- Client identity and status from Increment 1.
- Client profile completeness and unverified identity notice.
- Increment 2 work areas: Product & categories; Invoice register.
- Recent activity for this client.
- Clear message if client is archived or still a draft.

**Actions**
- Open product/category register.
- Open invoice register.
- Return to client list.
- No accounting ledger or tax filing actions.

**Role behavior**
- Demo administrator/manager/auditor: access synthetic client workspace.
- Read-only reviewer: view only.
- Production access will later require firm, client and engagement assignment checks; this prototype does not provide them.

### Window I2-W02 — Product/category register

**Purpose:** Maintain the reference list used during invoice capture.

**List columns**
- Category or product name.
- Item type (category/product).
- Internal code if the practitioner confirms it is needed.
- Active/archived status.
- Last updated.
- Usage count only if reliably derived.

**Primary actions**
- Add category.
- Add product.
- Search/filter.
- Open item.
- Archive item (only if not silently breaking existing invoice references).

**Behavior**
- Every item belongs to exactly one synthetic client.
- Do not show items belonging to another client.
- Category hierarchy depth, item codes, unit of measure, variants, inventory tracking and duplicate rules are **Practitioner decision required**. Start with the minimum confirmed fields; do not assume a full inventory module.
- Existing invoice lines retain their historical product/category reference if a catalogue item is later archived.
- Archive must warn if item is referenced; it must not rewrite historical invoices.

### Window I2-W03 — Add/edit category

**Purpose:** Create a usable classification without inventing an accounting taxonomy.

**Initial proposed fields**
- Category name (required).
- Optional description.
- Optional parent category only after the practitioner confirms hierarchy is needed.
- Status managed by the system.

**Validation**
- Trim outer whitespace; reject blank name.
- Show a possible duplicate within the same client; do not merge automatically.
- Preserve category references on existing invoice lines.
- Changes are recorded in activity history.

**Practitioner decision required:** naming conventions, allowed nesting, whether categories are shared across clients or strictly client-specific, and who maintains them.

### Window I2-W04 — Add/edit product

**Purpose:** Create a catalogue item that can be selected on invoice entry.

**Proposed minimum fields pending practitioner confirmation**
- Product/service name (required).
- Category (optional until category policy is confirmed).
- Description (optional).
- Item code/SKU (optional and not assumed unique).
- Unit/quantity description (optional until actual units are demonstrated).
- Active/archived status.

**Do not add** stock on hand, reorder thresholds, purchase cost, selling price, barcode, serial number, batch/lot, warehouse or valuation method in this increment without an approved inventory brief.

**Behavior**
- Product belongs to the current client.
- A product can be selected on invoice lines only if active.
- If archived after use, historic invoice lines remain readable.
- Read-only reviewer can view but not mutate.

### Window I2-W05 — Invoice register

**Purpose:** Find invoices and understand which require attention.

**Filters**
- Transaction class: Sales / Purchase (initially).
- Date or period range.
- Invoice number.
- Customer/supplier text.
- Draft / Needs correction / Ready for review / Reviewed.
- Duplicate warning / missing information.

**Columns**
- Invoice date.
- Invoice number.
- Sales or purchase.
- Customer/supplier name.
- TIN text (if provided).
- Amount before VAT.
- VAT amount as entered.
- Currency (only after the currency handling rule is confirmed; ETB default may be proposed but must not silently reinterpret source documents).
- Review state.
- Source attachment indicator.

**Actions**
- Add invoice.
- Open invoice.
- Filter/search.
- Import spreadsheet only after template and mapping validation; if included, import preview is mandatory.
- Export is deferred until output fields, privacy and export requirements are agreed.

**Empty state:** explain that no invoices have been captured for this client and point to Add invoice.

### Window I2-W06 — New invoice: classification and header

**Fields**
- Transaction class: Sales / Purchase.
- Invoice number (required only if practitioner confirms; otherwise allow draft with missing-field warning).
- Invoice date (required for a review-ready invoice; draft behavior explicitly indicated).
- Financial period/year (derived only under an approved calendar rule; do not infer tax period).
- Customer/supplier name.
- Customer/supplier TIN (opaque optional text).
- Remark/description (optional).
- Source type (paper, PDF, spreadsheet, other) and source reference.

**Behavior**
- Distinguish supplier from customer according to sales/purchase direction.
- Preserve source values as entered.
- No government TIN validation.
- Do not infer invoice authenticity.
- The original proposal's field “MRC” remains parked in the unresolved-field register until its meaning, type, requiredness and applicability are confirmed.

### Window I2-W07 — New invoice: line items and amounts

**Proposed fields**
- Product/service reference (optional until catalogue policy confirmed).
- Description as shown on source invoice.
- Quantity (decimal precision and unit policy require practitioner decision).
- Amount before VAT.
- VAT amount as shown on source document.
- Currency code and source amount where multi-currency is needed; exact FX workflow is not in this increment unless approved separately.
- Line note/remark.

**Rules**
- This is source-document capture, not a tax engine.
- Do not calculate legal VAT rates, exemptions, inclusive/exclusive treatment or tax liability from an unverified rule.
- If the user enters a total, show it as source-entered, not system-verified.
- Arithmetic cross-check may be shown only as a non-authoritative discrepancy warning after its formula and rounding behavior are approved.
- Preserve original captured values; corrections create a version/history record.

**Practitioner decision required:** invoice line vs header-only model, decimal precision, currency and FX capture, credit notes/cancellations, rounding, required invoice fields and whether quantity is meaningful for services.

### Window I2-W08 — Source evidence panel

**Purpose:** Associate an invoice with its original support.

**Prototype behavior**
- Use synthetic fixture documents or controlled sample files only.
- Display filename, type, size, capture date and association to invoice.
- Allow opening a safe preview where the file type is supported.
- If file preview fails, preserve the attachment reference and show an actionable message.

**Not yet authorized**
- Real client document ingestion, antivirus pipeline, OCR, arbitrary filesystem paths, cloud upload or public sharing.
- Before real files: security review of file type validation, malware scanning, size limits, storage encryption, path handling, retention and backup.
- A source file is evidence, not proof that invoice contents are correct.

### Window I2-W09 — Validation and duplicate review

**Purpose:** Surface incomplete or repeated records before review.

**Shows**
- Missing required information.
- Duplicate candidate reason (same client + invoice number, or other practitioner-approved match).
- Amount/date inconsistencies, only where defined.
- Links to candidate invoices and source documents.

**Actions**
- Open current invoice.
- Open candidate.
- Mark as not duplicate with reason (if policy approved).
- Return to draft.
- Do not delete or auto-merge.

**State model proposed**
- Draft → Ready for review → Reviewed.
- Ready for review → Needs correction.
- Needs correction → Draft/Ready for review after correction.
- Reviewed records are corrected through a new revision or controlled amendment; never silently overwritten.

**Practitioner decision required:** exact duplicate keys, who can mark not duplicate, whether reviewer can return/clear, and how corrected/cancelled invoices are represented.

### Window I2-W10 — Invoice review queue and detail

**Preparer view**
- Shows own drafts and items returned for correction.
- Explains what is missing and who returned it (demo actor).
- Resubmit after corrections.

**Reviewer view**
- Shows only synthetic invoices in review state.
- Opens source evidence, captured values, validation and change history.
- Can mark Reviewed or Needs correction only if the demonstration role is assigned review capability by the increment's explicit demo matrix.
- Cannot alter preparer's captured values while recording a review decision. A correction is made as a separate revision/action.

**Read-only reviewer persona**
- In the existing Increment 0/1 persona contract, this persona remains read-only. Do not silently give it review authority. If a review demonstration is needed, use the Demo Engagement Manager persona and clearly label the simulated permission.

## 3.2 Increment 2 role-by-role walkthrough

| User | What they do | What they must not be able to do |
|---|---|---|
| Demo Firm Administrator | Inspect client catalogue/invoices; maintain synthetic reference data for demo | Cross-client access outside demo firm; claim production admin powers |
| Demo Engagement Manager | Review register completeness, coordinate correction and inspect reviewed queue | Change source values while pretending to be reviewer; approve tax treatment |
| Demo Auditor/accountant | Add categories/products and capture sales/purchase invoices with source references | Mark an unreviewed record as professionally approved; claim TIN or invoice authenticity verified |
| Demo Read-only Reviewer | Inspect product and invoice records/history | Create, edit, archive, import or change review status |

## 3.3 Increment 2 minimum test scenarios

- Product/category records are client-scoped; cross-client ID substitution is rejected.
- Archived product remains visible on historical invoice.
- Draft invoice can be saved with permitted missing fields and shows exact missing list.
- Invalid values do not erase other fields.
- Duplicate invoice candidate is advisory; no automatic overwrite/merge.
- Source attachment metadata and file association survive restart.
- Unsupported or unsafe file type is rejected with a clear reason.
- Preparer cannot mark own work reviewed unless a future approved policy explicitly allows it.
- Reviewer decision and preparer correction are distinct events.
- Offline capture and restart preserve records.
- No VAT calculation or tax-compliance claim appears.

---

# 4. Increment 3 — Historical data and draft VAT workpaper

**Outcome:** A practitioner can enter/import historical purchases, expenses and tax-related records with provenance, assign them to periods, and prepare a draft schedule that can be traced to source records.

**Critical rule:** The tax specialist's asserted tax matrix has not yet been supplied and independently inspected. Therefore this increment may build data capture, source traceability, draft grouping and review UI, but no authoritative VAT computation, legal tax treatment, filing readiness or statutory output may be claimed until effective-dated rules and independently verified test cases are approved.

## 4.1 Main journey

Open client → Historical data → Select period → Capture/import records → Attach source → Resolve duplicates/missing data → Reconcile opening/historical values → Build draft VAT workpaper from eligible records → Review exceptions → Record reviewer outcome → Export only if approved export specification exists.

### Window I3-W01 — Historical data workspace

**Purpose:** Give the accountant a controlled home for prior-period information.

**Shows**
- Client and selected reporting period.
- Tabs/areas: Purchases, Expenses, Tax history, VAT carry-forward/revolving VAT (terminology to be confirmed), Imports and reconciliation.
- Record counts by draft/review state.
- Source completeness and unresolved exceptions.
- Clear distinction between historical/imported information and live accounting ledger entries.

**Actions**
- Add historical record.
- Import approved template.
- Open source register.
- Open reconciliation.
- Start draft VAT workpaper only when minimum data exists and rule status is clearly shown.

### Window I3-W02 — Period selector and period details

**Fields**
- Period label.
- Start date and end date.
- Period type (month/quarter/year/custom only after practitioner and tax-period policy approval).
- Source/authority for period definition.
- Status: Open for preparation / Locked for preparation (if lock behavior approved).

**Rules**
- Do not assume fiscal year equals calendar year.
- Do not derive statutory tax periods from a date without approved Ethiopian tax rules for the taxpayer type.
- Period overlap and gaps are shown; do not auto-correct silently.
- Historical record can retain its source period label even if mapping is unresolved.

### Window I3-W03 — Historical purchases and expenses register

**Purpose:** Capture prior records without presenting them as posted accounting entries.

**Proposed fields pending workflow validation**
- Record type: Purchase / Expense / Other (controlled set).
- Transaction date and source period.
- Supplier/payee name.
- TIN text (optional opaque).
- Description/category.
- Amount and currency as stated in source.
- VAT amount as stated in source, if present.
- Invoice/reference number, if present.
- Source document reference.
- Entry method: manual / imported.
- Preparer and captured timestamp.
- Review state.

**Rules**
- Historical entries are not automatically journalized.
- Original source amount and currency are preserved.
- No inferred VAT recoverability.
- Imported records carry import batch ID, original row number and source filename.
- Correcting a historical row creates a traceable revision.

**Practitioner decision required:** which historical classes are required, exact fields, tax treatment, opening balances, date coverage, category mapping and whether purchase/expense are separate records or a common transaction model.

### Window I3-W04 — Historical tax and VAT carry-forward register

**Purpose:** Preserve prior tax-period context and values supplied by the client/accountant.

**Proposed fields**
- Tax type / label exactly as supplied.
- Tax period as stated on source.
- Amount or balance as stated.
- Currency.
- Source document/return reference.
- Opening/closing/carry-forward label only where confirmed.
- Notes and review state.

**Special handling**
- “Revolving VAT” is retained as the wording from the supplied auditor proposal; do not silently rename it to a specific legal balance or assume its treatment.
- Carry-forward calculations, expiry, adjustments, offsets and recoverability require tax specialist approval.
- Values entered from prior returns are source facts, not automatically validated tax positions.

### Window I3-W05 — Import batch and mapping preview

**Purpose:** Let a user inspect a spreadsheet before data is written.

**Shows**
- File name and source format.
- Selected template/version.
- Sheet and row count.
- Column mapping with sample values.
- Required mapping gaps.
- Duplicate candidates and invalid rows.
- Import summary: rows ready, rows requiring correction, rows skipped by explicit user choice.

**Actions**
- Map columns.
- Preview normalized values.
- Download/view error report if export policy allows.
- Confirm import.
- Cancel without changing stored records.

**Rules**
- No silent row dropping.
- No import until the user confirms the preview.
- A failed batch is atomic or has an explicitly designed partial-import contract; do not leave ambiguous half-imported data.
- Store import provenance and row-level mapping errors.
- Spreadsheet formulas/macros are not executed.

### Window I3-W06 — Source and provenance detail

**Shows**
- Source file/document reference.
- Import batch and original row number, or manual entry label.
- Captured by and captured time.
- Current review state.
- Revision history.
- Link to source evidence.

**Purpose:** The user can answer, “Where did this amount come from?” without searching unrelated folders.

**Rules**
- Never replace original source values with calculated values.
- Any transformation displays source value, transformed value, formula/mapping version and actor.
- Real document handling remains gated by security approval.

### Window I3-W07 — Historical reconciliation

**Purpose:** Compare entered/imported historical information with supplied source totals or approved opening records.

**Shows**
- Source total and captured total by defined grouping.
- Difference.
- Missing/duplicate/unmapped items.
- Reconciliation note and reviewer status.

**Actions**
- Mark reconciled only when an approved reconciliation rule exists and required evidence is linked.
- Record an unresolved difference with explanation.
- Return records for correction.

**Not allowed**
- Auto-plug, forced balancing entry or hidden adjustment.
- Marking a discrepancy as zero by rounding without an approved precision rule.
- Converting historical data into posted ledger balances.

### Window I3-W08 — Draft VAT workpaper builder

**Purpose:** Create a reviewable draft schedule with links back to captured source records.

**Before calculation**
- Show tax rule pack name/version/effective period and approval status.
- If no approved rule pack and independently validated test cases exist, disable calculation and explain: **Tax calculation rules have not been approved for this workflow. You can review source data, but ARM cannot calculate an authoritative VAT result.**
- If an approved rule pack is available in a later authorized environment, display scope, effective date, source authority, assumptions and known exclusions.

**Draft schedule sections**
- Included source records.
- Excluded records with reason.
- Missing or ambiguous records.
- Manual adjustments (only if adjustment type and approval workflow are defined).
- Calculated totals, if and only if approved rule pack is active.
- Drill-down from every total to its contributing invoice/history rows.

**Rules**
- A draft workpaper is not an official return or submission.
- No legal tax conclusion is generated by a UI status.
- Every calculation is reproducible from stored inputs, rule version and rounding settings.
- Changing source data invalidates/recomputes the draft under controlled versioning; prior draft remains in history.

### Window I3-W09 — VAT exception and review window

**Preparer sees**
- Missing documents, unknown tax category, unmatched period, duplicate source, invalid amount/currency, and rule exceptions.
- Each exception has a description and a clear next action.
- Preparer can correct source capture or explain why the issue remains unresolved.

**Reviewer sees**
- Workpaper totals and source drill-down.
- Rule pack/version and assumptions.
- Excluded rows and manual adjustments.
- Review notes and exception disposition.

**Review actions**
- Return for correction.
- Clear an exception with a reason, if the review policy allows.
- Mark workpaper reviewed (not filed/approved by authority).
- Record professional sign-off only after the real approval matrix is defined and implemented.

### Window I3-W10 — Workpaper export/print preview

**Purpose:** Preview an internally usable workpaper and its supporting register.

**May be implemented only after**
- Required report fields and layout are approved.
- Export access and data minimization are defined.
- The output visibly says Draft / Internal workpaper and includes period, client, preparer, review state, rule version (if applicable) and generated timestamp.

**Never label**
- Official tax return, filed, government accepted, compliant or final unless a separately approved and verified workflow justifies it.

## 4.2 Increment 3 role-by-role walkthrough

| User | What they do | What they must not be able to do |
|---|---|---|
| Demo Firm Administrator | Inspect historical workspace and audit history; manage synthetic demo configuration | Change tax rule authority or declare a tax rule approved |
| Demo Engagement Manager | Monitor completeness, assign follow-up in the future task module, inspect reconciliation and review state | Override unresolved tax-rule evidence |
| Demo Auditor/accountant | Capture/import historical records, link source and prepare draft workpaper | Convert historical records to posted journals or claim statutory correctness |
| Reviewer (simulated via manager until a distinct review persona is approved) | Inspect source traceability, exceptions and draft calculations where approved | Approve own preparation or bypass missing rule pack |
| Client contact | Not in this desktop increment | No client portal or direct access is implied |

## 4.3 Increment 3 minimum test scenarios

- Historical data remains distinct from posted accounting data.
- Import preview identifies unmapped columns and bad rows before commit.
- Import provenance identifies file, sheet/row and mapping version.
- No row is silently dropped.
- Source-to-workpaper drill-down reconciles to included rows.
- No approved tax rule pack means no authoritative VAT calculation.
- Tax rule version/effective date is stored with every calculated draft.
- Changing an input does not rewrite a previously reviewed snapshot without trace.
- Review notes and preparer corrections are distinct events.
- Offline restart preserves source references and draft workpapers.

---

# 5. Increment 4 — Staff tasks and review workflow

**Outcome:** A manager can coordinate staff work with explicit assignment, due dates, priorities and blockers; a preparer can complete work and submit it for review; a reviewer can return or clear review notes separately from task completion.

**Boundary:** Task completion is not professional approval. Staff profile management and role-based access in production require a separately approved identity, invitation, firm membership, assignment and authorization design. Demo staff records and demo personas are not production users.

## 5.1 Main journey

Manager opens Work queue → Creates task from client/work item → Assigns demo staff member → Sets due date and priority → Preparer opens My work → Performs work and records outcome → Submits for review → Reviewer checks output and evidence → Clears review note or returns with comments → Manager monitors overdue/blockers.

### Window I4-W01 — Staff directory

**Purpose:** Show who is available for work in the synthetic firm.

**Proposed fields**
- Display name.
- Demo staff reference.
- Job title / discipline (optional).
- Active/inactive status.
- Skills or work categories only if practitioners confirm they are useful.
- Demo persona association (for simulation only).

**Rules**
- Do not collect real employee identity documents, payroll data, bank details, salary or personal sensitive data.
- No invitation, email login, password, MFA or production account creation.
- A staff profile is not a credential.
- Deactivation must not erase historical task assignments.

### Window I4-W02 — Work overview / manager dashboard

**Purpose:** Give managers one clear place to see work progress and next action.

**Views**
- My tasks.
- Team tasks.
- Due soon.
- Overdue.
- Blocked.
- Waiting for review.
- Recently completed.

**Summary values**
- Counts derived from task records and current status.
- Filters by client, assignee, priority and due date.
- No arbitrary productivity ranking or staff surveillance metrics.

**Primary actions**
- Create task.
- Open task.
- Reassign task (if policy permits).
- Open workload view.

**Role behavior**
- Manager: sees synthetic team queue and can create/assign.
- Preparer/auditor: sees own assigned work.
- Reviewer: sees tasks explicitly assigned for review.
- Read-only reviewer demo persona remains view-only; do not confuse the label with professional review authority.

### Window I4-W03 — Create task

**Fields**
- Task title (required).
- Description / expected output (required enough to explain the work).
- Client (required for client work; select from current demo firm's active clients).
- Related record (optional: invoice batch, historical import, VAT workpaper or other approved target).
- Assignee (required).
- Due date (required only if firm chooses; default must not be invented).
- Priority (proposed Low / Normal / High / Urgent; confirm meanings with practitioners).
- Task type (proposed data capture / review / follow-up / other).
- Required evidence or completion note (optional until workflow is defined).

**Behavior**
- Show a concise summary before creation.
- Warn if client is archived or related record is inaccessible.
- Assignee must be active in the synthetic staff directory.
- Save and create an audit event in one transaction.
- Assignment notification is in-app/local demo only; no email or push notification in this increment.

**Practitioner decision required:** whether a task may have multiple assignees, watchers, dependencies, recurring schedule, estimated hours, confidentiality level and task templates.

### Window I4-W04 — Task detail

**Shows**
- Task title, description and expected output.
- Client and related record.
- Creator, assignee, due date, priority.
- Current status and status history.
- Comments/activity.
- Blocker and review state.
- Attachments/evidence references where permitted.

**Actions**
- Assignee: start, update progress, mark blocked, submit for review, respond to return.
- Manager: edit assignment/due date/priority, reassign with reason, cancel task with reason.
- Reviewer: open review item and record review note/decision.
- Other users: view only if task access is explicitly granted.

**Rules**
- Every status change records actor, time, previous state, next state and reason where required.
- Reassignment preserves previous assignee and history.
- Cancelling a task does not delete its work product or evidence.

### Window I4-W05 — My work

**Purpose:** Help an individual know what to do next without navigating the entire firm.

**Sections**
- Continue current task.
- Due next.
- Overdue.
- Blocked / waiting on someone.
- Returned for correction.
- Submitted and waiting for review.

**Each task row shows**
- Client.
- Plain-language task title.
- Due date.
- Priority.
- Current state.
- Next required action.

**Primary action:** Open task / Continue work.

**Empty state:** “No assigned work is waiting for you.” Do not fill it with fake tasks.

### Window I4-W06 — Task execution / completion form

**Fields**
- Work performed / completion summary.
- Result or deliverable reference.
- Evidence links (to existing synthetic records).
- Time spent only if practitioner confirms the need and data policy.
- Blocker explanation when status is Blocked.
- Submit-for-review confirmation.

**Rules**
- A task cannot be marked complete if required output is absent.
- Submitting for review changes task state to Waiting for review; it does not mark the underlying accounting/audit work approved.
- A user cannot submit another person's task unless explicitly assigned as delegate under a defined policy.
- Preserve draft progress on local save.

### Window I4-W07 — Review queue

**Purpose:** Separate review work from preparer work.

**Shows**
- Items awaiting review.
- Client and related work item.
- Preparer and submitted timestamp.
- Due date / priority.
- Required output and evidence completeness.
- Prior review notes.

**Reviewer actions**
- Open review.
- Clear review note / accept task output (task-level acceptance only).
- Return to preparer with specific findings and required correction.
- Escalate to manager where policy requires.
- Add review note with category, explanation and evidence reference.

**Rules**
- Reviewer must not silently edit preparer's submitted work.
- A reviewer action is attributable and append-only.
- The task-level “accepted” state must not be displayed as an audit opinion, tax approval or posted-journal authorization.
- Separation of preparer/reviewer must be enforced in the demo command layer where the simulated policy says so; production enforcement requires server-side RBAC.

### Window I4-W08 — Review note detail

**Fields**
- Review note category (proposed: missing evidence / incorrect data / unclear explanation / calculation query / other).
- Description (required).
- Affected record/evidence link.
- Required response.
- Reviewer and timestamp.
- Resolution response by preparer.
- Reviewer disposition and timestamp.

**Lifecycle**
- Open → Responded → Cleared, or Open → Responded → Reopened.
- If returned for correction, task returns to In progress / Needs correction according to a confirmed state model.
- A review note cannot be deleted to hide a disagreement.
- Closure requires reviewer action and a recorded reason/outcome.

**Practitioner decision required:** exact note categories, whether multiple review cycles are allowed, escalation and partner sign-off requirements.

### Window I4-W09 — Workload and overdue view

**Purpose:** Help managers remove bottlenecks rather than simply count tasks.

**Shows**
- Tasks by assignee and state.
- Overdue and due-soon list.
- Blocker age and explanation.
- Work waiting for review.
- Reassignment history.

**Rules**
- Avoid employee performance scores in the prototype.
- Counts must be traceable to task rows.
- Filters must respect client/workspace boundaries.
- No hidden reassignment or automatic reassignment.

## 5.2 Increment 4 role-by-role walkthrough

| User | What they do | What they must not be able to do |
|---|---|---|
| Demo Firm Administrator | Inspect synthetic staff directory and overall work queue | Create production identities or bypass firm authorization |
| Demo Engagement Manager | Create tasks, assign/reassign, set priority/due date, monitor blockers and review queue | Mark a preparer's professional work approved merely because a task is complete |
| Demo Auditor/accountant (preparer) | Open assigned task, work on related records, record completion, submit for review and respond to notes | Clear their own review note or approve their own restricted output |
| Reviewer (simulated only with explicit assignment) | Inspect submitted work, evidence and notes; return or clear task-level review | Rewrite submitted work invisibly or self-review their own work |
| Demo Read-only Reviewer | Inspect task/history only | Assign, change status, submit, review or approve |

## 5.3 Increment 4 minimum test scenarios

- Inactive staff member cannot receive new assignment.
- Reassignment retains prior assignee and reason.
- Preparer cannot clear own review note.
- Task completion and professional approval are separate states.
- Reviewer action is attributable and cannot be deleted.
- Blocked task requires a reason and appears in manager queue.
- Overdue calculations use the stored due date and local display timezone correctly.
- Cancelled task retains history and work product.
- Cross-client task access is rejected.
- Offline task updates persist after restart; no sync status is fabricated.

---

# 6. Increment 5 — Import-cost worksheet discovery and controlled prototype

**Outcome:** First obtain and approve the missing practitioner brief. Only then prototype an import-cost worksheet whose inputs, allocation method, FX assumptions and calculation are reproducible. Selling-price suggestion remains optional and separate.

**Hard dependency:** The original auditor proposal explicitly says an additional brief is needed for imported-goods cost inputs. The worksheet must not be implemented as a guessed list of customs, freight, insurance, duty or other charges. Even plausible components require confirmation of applicability and allocation method.

## 6.1 Gate 5-A — Practitioner discovery workshop (required before coding calculation logic)

### Window I5-W01 — Import-cost discovery workspace

**Purpose:** Collect real workflow knowledge in a structured way without asking auditors to design software.

**Workshop participants**
- Accountant/auditor who prepares import-cost calculations.
- Reviewer/partner who checks them.
- Client/import operations representative where appropriate.
- Tax/customs specialist for legal or tax treatment questions.
- Product owner / analyst recording decisions.

**Workshop prompts**
1. What event starts the calculation?
2. Is the unit of work a shipment, customs declaration, supplier invoice, item, container or another object?
3. Which source documents are normally available?
4. Which cost components are used in practice? Which are optional or case-specific?
5. Which amounts are in ETB and which are foreign currency?
6. Which FX rate, source and date are used, and who authorizes it?
7. How are shared costs allocated across products/items?
8. How are damaged, missing, returned or partially received goods handled?
9. Which taxes/duties are included, excluded or separately recoverable? Who confirms this?
10. What output is required and who reviews it?
11. What happens when a source document or FX rate is missing?
12. Is the result used for inventory valuation, management pricing, tax, audit evidence or more than one purpose?

**Workshop outputs**
- Approved workflow diagram.
- Field dictionary with definition, unit, currency, source and requiredness.
- Sample anonymized/synthetic import case with expected result independently calculated.
- Cost component inclusion/exclusion policy.
- FX source/date/authorization policy.
- Allocation method and worked example.
- Exception handling and review matrix.
- Versioned calculation specification.
- List of unresolved legal/accounting questions and their qualified owners.

**Exit rule:** no calculation window is implemented until the above artifacts are accepted by the relevant accounting/tax practitioners and product owner. If they cannot agree, record the disagreement and keep the feature in discovery.

### Window I5-W02 — Import case register (only after Gate 5-A)

**Purpose:** List approved import-cost cases.

**Proposed fields after workshop**
- Case reference.
- Client.
- Shipment/import event identifier as confirmed.
- Supplier/source party.
- Relevant dates.
- Currency and status.
- Prepared by / reviewer.
- Calculation version.
- Status: Draft / Ready for review / Returned / Reviewed / Archived.

Do not assume one case equals one customs declaration or one invoice; the workshop determines the relationship.

### Window I5-W03 — Source document and cost input capture

**Purpose:** Capture each approved cost input with source traceability.

**Each cost input must carry**
- Component code/name from approved configuration.
- Amount and currency.
- Source document/reference.
- Date and source party.
- FX rate, source, date and authorization where conversion is needed.
- Allocation basis if shared across items.
- Included/excluded status and reason.
- Preparer and capture timestamp.

**Rules**
- Never hide the source currency amount after conversion.
- Preserve original value and converted value with rate/source/date.
- No default FX rate may be silently applied.
- Missing evidence or rate creates an exception, not a guessed value.
- Component configuration is versioned; changes do not rewrite prior calculations.

### Window I5-W04 — Allocation and calculation preview

**Purpose:** Show the user how the total is allocated and how the result was derived.

**Shows**
- Approved inputs and source references.
- Allocation basis and formula.
- Per-item and total calculations, where applicable.
- Rounding precision and method.
- Excluded items and reasons.
- Missing inputs and warnings.
- Calculation specification version.

**Rules**
- The same saved inputs and version must reproduce the same result.
- The user can inspect the formula and allocation path.
- Changing a source input creates a new calculation revision.
- No silent balancing plug.
- The application must not label the result a statutory customs value, tax basis or audit conclusion unless qualified policy and authority support that claim.

### Window I5-W05 — Import-cost review

**Preparer view**
- See unresolved exceptions.
- Correct inputs or attach missing source.
- Submit a version for review.

**Reviewer view**
- Reperform/inspect calculation.
- Compare source documents and FX assumptions.
- Accept for the stated internal purpose or return with a specific issue.
- Record review note and version reviewed.

**Rules**
- Reviewer sees immutable snapshot of the version submitted.
- Subsequent edits create a new revision and invalidate the prior “current” review status.
- Review purpose must be explicit (e.g. internal costing); do not reuse the same approval for tax or audit purposes without a separate policy.

### Window I5-W06 — Optional selling-price analysis (separate feature gate)

This is not part of the required import-cost worksheet. Implement only after a separate product decision and practitioner validation.

**If approved, proposed inputs**
- Selected cost basis/version.
- Pricing method (markup or margin; do not treat them as interchangeable).
- Target rate and assumptions.
- Optional overhead/market assumptions only if defined.
- Scenario comparison.

**Shows**
- Cost basis.
- Formula and method.
- Assumptions and sensitivity.
- Suggested price range/result clearly labelled **management decision support — not an audit opinion, statutory valuation or guaranteed market price**.

**Rules**
- No automatic price publication or transaction posting.
- User must be able to edit assumptions and understand the result.
- Keep pricing output separate from tax calculation and audit conclusions.

## 6.2 Increment 5 role-by-role walkthrough

| User | What they do | What they must not be able to do |
|---|---|---|
| Demo Firm Administrator | Inspect case register and configuration/version history | Invent or silently change approved cost components |
| Demo Engagement Manager | Monitor preparation/review state and assign follow-up tasks once Increment 4 exists | Override accounting/tax policy or mark unresolved calculation approved |
| Demo Auditor/accountant (preparer) | Capture source-backed cost inputs, run approved calculation version and submit for review | Guess missing inputs, FX rates or allocation rules |
| Reviewer / accounting lead | Reperform calculation, inspect assumptions and approve for the stated purpose or return | Approve a different calculation version or use costing approval as tax/audit approval |
| Client contact | Not part of desktop prototype; future client portal may provide requested source documents | Cannot see firm workpapers or internal review notes by default |

## 6.3 Increment 5 minimum test scenarios

- No approved discovery brief means calculation cannot be activated.
- Each cost input traces to a source and preserves original currency.
- FX rate has source, date and authorization where required.
- Allocation is reproducible and versioned.
- Missing source/rate blocks readiness rather than receiving a guessed default.
- Changing an input creates a new revision and invalidates prior review for current version.
- Reviewer sees exact submitted snapshot.
- Calculation reproduces independently verified expected results.
- Optional selling-price analysis remains separate and is not represented as a statutory or audit result.

---

# 7. Cross-increment role and action matrix

This matrix is a prototype planning aid, not a production authorization policy.

| Capability | Demo administrator | Demo manager | Demo auditor | Demo read-only reviewer |
|---|---|---|---|---|
| View client register/profile | Yes | Yes | Yes | Yes |
| Create/edit client profile (I1) | Yes | Yes | Yes | No |
| Archive/restore client (I1) | Yes | Yes | Yes | No |
| Maintain product/category (I2) | Yes | Yes | Yes | No |
| Capture invoice (I2) | Yes | Yes | Yes | No |
| Mark invoice reviewed | Only if explicit demo review assignment is implemented | Only if explicit demo review assignment is implemented | No self-review | No |
| Capture historical data (I3) | Yes for demo administration | Yes | Yes | View only |
| Run approved draft workpaper calculation (I3) | No tax-rule approval implied | Only if authorized in future matrix | Only with approved rule pack | View only |
| Create/assign staff tasks (I4) | Demo admin oversight | Yes | No unless delegated | No |
| Complete assigned task (I4) | Only if assigned | If assigned | If assigned | No |
| Review task output (I4) | No automatic authority | If explicitly assigned and not preparer | No self-review | No |
| Prepare import-cost worksheet (I5) | No automatic authority | If assigned | If assigned | No |
| Review import-cost calculation (I5) | No automatic authority | If qualified and explicitly assigned | No self-review | No |

Before each increment is coded, replace any “only if explicit” row with a precise command-level demo permission and negative tests. Do not infer production permissions from this matrix.

---

# 8. Shared states, audit events and persistence requirements

Every increment must use explicit state transitions and append-only history. The following are minimum event families; physical event names may be mapped to repository conventions if documented.

| Increment | Event families |
|---|---|
| 1 | client created/updated/activated/archived/restored; duplicate acknowledged |
| 2 | category/product created/updated/archived; invoice created/revised/submitted/returned/reviewed; source attached; duplicate disposition |
| 3 | period created/updated/locked; historical record created/revised/imported; import confirmed/failed; reconciliation recorded; workpaper generated/revised/reviewed |
| 4 | staff demo record created/updated/deactivated; task created/assigned/reassigned/started/blocked/submitted/returned/completed/cancelled; review note opened/responded/cleared/reopened |
| 5 | discovery decision recorded; import case created/updated; cost input captured/revised; FX assumption recorded; calculation generated/revised/submitted/reviewed/returned; pricing scenario generated if separately approved |

Event payloads must include event UUID, workspace/client scope, actor, target, UTC timestamp, correlation ID, schema version and minimal structured detail. Do not store secrets or unnecessary personal data. A business mutation and its event commit atomically.

For records with review:
- Save the submitted version/snapshot.
- Identify exactly which version was reviewed.
- Preserve reviewer, timestamp, decision and note.
- Any subsequent change creates a new version and clearly marks prior review as applying to the prior version only.

## 8.1 Offline contract across Increments 1–5

Until a later sync increment is separately designed and implemented:
- Local save is the only persistence acknowledgment.
- Every workflow must work without internet once its local dependencies are present.
- No screen may say Synced, Uploaded or Cloud backup complete.
- A future sync implementation must define outbox/idempotency, server acknowledgments, conflicts, authorization leases, revocation and recovery before UI claims are added.
- Offline availability does not authorize restricted professional actions indefinitely. Production offline authorization remains a separate security gate.

---

# 9. Common navigation and window naming map

| Increment | Navigation entry | Window IDs | Main user outcome |
|---|---|---|---|
| 1 | Clients | I1-W01 to I1-W06 | Create and maintain client profile |
| 2 | Client workspace → Products & invoices | I2-W01 to I2-W10 | Catalogue and invoice capture/review |
| 3 | Client workspace → Historical data / VAT workpaper | I3-W01 to I3-W10 | Traceable historical data and controlled draft workpaper |
| 4 | Work → Staff / My work / Review queue | I4-W01 to I4-W09 | Assign, complete and review work |
| 5 | Client workspace → Import costing | I5-W01 to I5-W06 | Discover and, only after approval, calculate import cost |

Navigation entries appear only when the increment is implemented and enabled. Do not expose unfinished modules as clickable empty pages.

---

# 10. Developer handoff checklist per window

Before implementing a window, the developer and product owner must confirm:
- [ ] User problem and intended outcome are stated in plain language.
- [ ] Entry point and exit/next action are defined.
- [ ] Fields have definition, type, example, source, requiredness and validation.
- [ ] Role/action matrix includes positive and negative permissions.
- [ ] State transitions and invalid transitions are defined.
- [ ] Save, cancel, dirty-form and restart behavior are defined.
- [ ] Offline and failure behavior are defined.
- [ ] Audit event payload and history display are defined.
- [ ] Empty, loading, validation, conflict and error states are designed.
- [ ] Keyboard, accessibility and responsive/resizing behavior are reviewed.
- [ ] Synthetic demo scenario and expected result exist.
- [ ] Automated tests cover normal, negative, persistence and authorization paths.
- [ ] Practitioner feedback and unresolved decisions are linked.

## 10.1 Required artifacts at increment exit

1. Working Windows desktop build using synthetic data.
2. Screenshots or recording of every listed primary window and role journey.
3. Field dictionary and state-transition tables updated from practitioner feedback.
4. Schema/migration and event-contract documentation.
5. Unit, integration, UI, persistence, offline and negative-permission test results.
6. Independent calculation fixtures where a calculation is in scope.
7. Defect/deferred-decision register with owner and risk.
8. Updated PRD traceability from requirement → window → service → schema → test.
9. Practitioner demonstration record and explicit acceptance/rejection notes.
10. No claim of production readiness, legal compliance, tax correctness or CI success without corresponding evidence.

---

# 11. Decisions that must be resolved at the right time

Do not ask the product owner to decide every future detail before Increment 1. Ask only when the next increment reaches the relevant workflow demonstration.

| Decision | Needed before | Who should inform/approve |
|---|---|---|
| Client legal/entity classifications and additional identity fields | Extending I1 beyond the approved minimum | Practitioners, legal/tax specialist, product owner |
| Product category hierarchy, product fields and ownership scope | I2 catalogue implementation | Auditors/accountants and product owner |
| Invoice types, MRC meaning, line/header structure, duplicate and correction rules | I2 invoice review implementation | Auditors and tax specialist |
| Document storage, malware scanning, real-data upload and retention | Any real document handling | Security/privacy/legal owners |
| Historical periods, opening data, “revolving VAT” meaning and source coverage | I3 historical capture | Accountant and tax specialist |
| Tax matrix, VAT treatment, effective dates, rounding and expected results | Any VAT calculation | Qualified tax specialist; independent reviewer |
| Staff roles, delegation, self-review and approval authority | I4 role enforcement | Firm leadership, audit methodology owner, security architect |
| Task lifecycle, priority meanings, due-date rules and review-note categories | I4 workflow finalization | Managers and practitioners |
| Import cost components, FX source, allocation and valuation purpose | I5 calculation prototype | Import accountant, reviewer, tax/customs specialist |
| Markup vs margin and pricing purpose | Optional I5-W06 | Management/accounting practitioner and product owner |

**No preference question is required to complete this document.** The listed questions are intentionally sequenced to the increment where practitioners can demonstrate the real work. The next immediate implementation can proceed with the already approved Increment 1 contract; Increment 2–5 should be treated as detailed workflow proposals whose explicitly marked business-rule decisions are confirmed before those rules are coded.
