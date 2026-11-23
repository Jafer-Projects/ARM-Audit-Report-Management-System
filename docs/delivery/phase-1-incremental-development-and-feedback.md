# Phase 1 — Incremental development and practitioner feedback plan

**Authorization:** Phase 0 is approved for controlled initial development by product-owner direction recorded on 2026-11-23. This authorizes synthetic-data development and scoped technical spikes. It does not close legal/security/accounting/tax/operations gates for real client data or production.

## 1. Delivery philosophy

Build software to help practitioners do familiar work. Do not wait for auditors to write complete software specifications. Observe the work, build a small workflow, demonstrate it, collect corrections, and update the requirements. Keep unresolved assumptions explicit.

A sprint/increment should end with something a practitioner can try, not only an internal technical layer. Foundational technical work may precede the UI where necessary, but each vertical slice should connect user action to stored data, validation, evidence and an observable outcome.

## 2. Initial increments

### Increment 0 — Working shell and safe local workspace
- Windows desktop shell and simple navigation.
- One synthetic demo firm and fictional clients; fixed demo personas are explicitly not production authentication or RBAC.
- Local SQLite persistence and a separately documented encryption/key-management spike; no production-encryption claim until verified.
- Clear local-only state; synchronization is not implemented or simulated.
- Append-only local activity history for material demo actions.
- No production client data.

**Demo:** user opens a demo client workspace offline, creates a local draft note (not a task or workpaper) and sees it remain after restart.

**Normative detail:** follow [Developer Implementation Specification — Increments 0 and 1](developer-implementation-specification-increments-0-and-1.md).

### Increment 1 — Client and company registration
- Guided client profile.
- Legal/business name, TIN, sector and approved contact fields.
- Completeness indicator and field-level help.
- Duplicate warning and change history.
- One demo-firm-scoped client profile; no production tenant authorization.
- Explicit field dictionary, validation, duplicate warning, completeness, archive/restore and change-history behavior.

**Demo:** auditor creates a company, sees missing required details, completes the profile and retrieves it later.

**Normative detail:** follow [Developer Implementation Specification — Increments 0 and 1](developer-implementation-specification-increments-0-and-1.md).

### Increment 2 — Product/category register and invoice capture
- Product categories and product register.
- Local sales/purchase invoice capture; import/export invoice class only after fields and sample evidence are validated.
- Source attachment, duplicate detection, missing-field warnings and spreadsheet import.
- Draft/reviewed states.

**Demo:** auditor records and attaches a batch of invoices, fixes validation issues and filters the register.

### Increment 3 — Historical data and VAT workpaper
- Historical purchases, expenses, tax history and revolving VAT/carry-forward capture.
- Period assignment and source provenance.
- Draft VAT schedule with invoice-level drill-down.
- Review adjustments, calculation version and export.
- Tax specialist approves test cases and rules before any authoritative calculation claim.

**Demo:** auditor generates a draft VAT schedule and traces every included amount to a source record.

### Increment 4 — Staff tasks and review workflow
- Staff profiles and role-based access.
- Task assignment, due date, priority, status and blockers.
- Review notes and authorized approvals as distinct actions.
- Overdue/workload view.

**Demo:** manager assigns invoice review to staff; staff completes work; reviewer clears the review note.

### Increment 5 — Import-cost worksheet discovery
- First conduct a practitioner workshop to obtain missing import-cost input brief.
- Prototype only after cost components, currency/FX, source documents and allocation method are agreed.
- Keep suggested selling price analysis optional and separate from audit/tax conclusions.

**Demo:** reviewer reproduces the cost calculation from the saved inputs and assumptions.

## 3. Practitioner demonstration protocol

For each increment:
1. Prepare a short scenario using synthetic or de-identified data.
2. Ask the practitioner to perform the task as they currently do it.
3. Observe and record their sequence without forcing software terminology.
4. Ask what they do when data is missing, contradictory, late or corrected.
5. Show the proposed system workflow.
6. Ask the practitioner to identify wrong terminology, missing steps, unnecessary fields and unsafe assumptions.
7. Record accepted changes, unresolved questions and rejected suggestions with rationale.
8. Update the PRD, workflow specification, backlog and tests.
9. Re-demonstrate the corrected workflow and record acceptance.

Do not interpret silence, a successful demo or a request for a new feature as professional approval of accounting/tax rules.

## 4. Required artefacts per increment

- User story and problem statement.
- Workflow diagram or numbered process.
- Field dictionary with definition, example, source, requiredness and validation.
- Role/action matrix.
- State-transition table.
- Acceptance examples and negative cases.
- Evidence/audit-trail specification.
- Offline/sync behavior.
- Demo notes and practitioner feedback.
- Deferred decisions and risks.

## 5. Release boundaries

| Use mode | Allowed now? | Conditions |
|---|---|---|
| Local development with synthetic data | Yes | Approved repository controls and secure development practices |
| Internal demo with synthetic data | Yes | No production credentials or live integrations |
| Practitioner usability session with synthetic/de-identified data | Yes | Consent, controlled access and feedback record |
| Pilot with real client data | Not yet | Privacy/legal role review, contracts, security assessment, retention, access, backup/restore and firm approval |
| Live accounting posting | Not yet | Approved accounting policies, immutable journal controls, authorization and independent tests |
| Tax filing or government submission | Not yet | Approved tax rules, authorized filer and verified official process/interface |
| Production hosting and general availability | Not yet | Security, operations, RPO/RTO, recovery, monitoring, support and release gates approved |

## 6. Change and feedback governance

Every feedback item is classified as:
- Defect (system differs from accepted behavior).
- Requirement clarification (current requirement is ambiguous).
- New requirement (new capability).
- Policy/legal question (requires qualified review).
- Usability improvement.
- Training/documentation issue.

Prioritize by professional risk, user impact, frequency, effort and dependency. Do not implement a request that weakens authorization, auditability, financial integrity or legal compliance without formal review. Record why a suggestion is accepted, deferred or rejected.

## 7. Exit from initial development

The first pilot candidate is ready only when:
- A complete chosen workflow passes acceptance tests.
- Practitioners have demonstrated it and recorded feedback.
- Role boundaries and audit history are tested.
- Offline/restart/sync behavior is demonstrated for the workflow.
- No unresolved critical defect or unsafe tax/accounting assumption remains.
- Applicable real-data and operational gates are approved before any real client data is introduced.
