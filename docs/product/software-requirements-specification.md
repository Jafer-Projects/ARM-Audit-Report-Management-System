# ARM Product Requirements Document / Software Requirements Specification

## 1. Purpose and status

This document defines the product direction, initial scope and evolving requirements baseline for ARM (Audit Report Management System), an offline-first auditing, accounting and tax practice platform for Ethiopia.

**Delivery model:** progressive, workflow-led development. Phase 0 is approved by the product owner for controlled initial development. Detailed requirements will be elaborated through demonstrable increments and auditor/client feedback. This approval does not mean every legal, security, accounting, tax, operational or production gate is closed.

This document is a living baseline. Requirements must retain status (proposed, accepted for increment, implemented, verified, deferred or rejected), owner, evidence and change history. It is not legal advice, an audit methodology certification or confirmation that a government integration interface is available.

## 2. Product vision

Provide professional firms and independent practitioners with one dependable environment to manage client accounting, audit engagements, tax preparation, evidence, approvals and reporting. The Windows desktop client must support substantive work during unreliable or absent connectivity. Cloud services provide centralized control, collaboration, synchronization, client access and authorized integrations.

The product should reduce auditors' administrative burden by guiding them through familiar tasks, collecting information once, reusing verified records, surfacing missing information and linking reports to their underlying evidence.

## 3. Users and product surfaces

### 3.1 User groups
- Independent auditors and audit partners.
- Professional accountants and bookkeepers.
- Tax professionals.
- Firm owners, engagement managers and reviewers.
- Client business representatives and authorized employees.
- Platform operations and security administrators.

Government officers are not an initial user group. Any regulator-facing access requires a separately authorized, explicitly scoped integration or portal.

### 3.2 Product surfaces
- **Windows professional desktop:** primary auditor/accountant work surface for assigned clients, engagements, accounting, invoice processing, workpapers, reports and offline tasks.
- **Client portal:** firm-invited client users submit documents, answer requests, clarify information and access released deliverables.
- **Firm workspace:** manage staff, clients, engagements, assignments, review queues, subscription seats and operational reporting.
- **Platform administration:** restricted tenant, subscription, security and operational controls. Platform administrators do not automatically receive client-content access; exceptional support access must be authorized, time-limited and audited.

## 4. Product principles

1. Workflow before feature catalogue: deliver complete tasks from input to reviewed output.
2. Auditor language first: use familiar terms and examples; do not require auditors to specify technical designs.
3. Progressive disclosure: show the next useful action, explain missing information and reveal advanced fields only when relevant.
4. Human accountability: automation assists; qualified professionals remain responsible for judgments, postings, approvals, report signing and tax submissions.
5. Evidence traceability: material outputs link to source records, calculations, versions and review history.
6. Offline-first: local work remains durable without connectivity; synchronization is explicit and recoverable.
7. Safe iteration: learn from pilot use without weakening tenant isolation, financial integrity, privacy or audit trails.
8. No false completion: draft, saved locally, synchronized, reviewed, approved, posted, issued and submitted are distinct states.

## 5. Functional scope

### 5.1 Identity, organizations and licensing
- Register individual professionals and firms.
- Support users who belong to multiple firms with distinct organization-scoped permissions.
- Support individual and firm subscriptions.
- Manage invitations, seat assignments, renewals, suspension and cancellation.
- Register and revoke desktop devices.
- Support MFA, secure recovery and session management.
- Maintain professional credential information; distinguish self-declared from independently verified credentials.
- Regulated services and signing permissions must be gated by verified applicable professional and firm authorization.

### 5.2 Client and engagement management
- Create firm-administered client workspaces subject to contract, confidentiality, applicable law and client rights.
- Record approved legal/business identity, contacts, tax identifiers and branches.
- Record engagement type, scope, period, terms, team, budget, milestones and status.
- Support acceptance, continuance, independence and conflict checks.
- Maintain engagement letters, approvals and change history.
- Distinguish client-provided source records from firm-created workpapers and professional conclusions.

### 5.3 Accounting
- Chart of accounts and accounting periods.
- Journals, general ledger and trial balance.
- Accounts receivable/payable, bank import/reconciliation, payroll accounting, inventory and fixed assets.
- Budgets, variance analysis, financial statements and supporting schedules.
- Period close, controlled reopening and adjustment workflows.
- ETB as primary currency with foreign-currency transactions, subject to approved functional-currency, FX, precision and rounding policies.

### 5.4 Audit
- Financial statement, internal, compliance, tax-related and NGO/donor-funded engagement workflows.
- Planning, materiality, risk assessment and control documentation.
- Audit programme and procedure assignment; sampling and substantive testing.
- Evidence collection, indexing and cross-referencing.
- Exceptions, findings, proposed adjustments and management responses.
- Review notes, preparer/reviewer separation, partner approval and report issuance.
- Methodology and template version tracking.

### 5.5 Auditor pain-point workflows (source proposal, 9 January 2025)

The supplied *Audit Report Management Software (ARMS) — Proposal for the development of the ARMS* identifies practical auditor hassles. These are included as initial requirements, not silently treated as fully specified features. Detailed traceability and unresolved questions are in [Auditor pain points and initial workflow requirements](auditor-pain-points-and-workflow-requirements.md).

Initial capabilities:
- **Product management:** client-scoped product categories and product register, with the ability to add products as needed.
- **Customer/company registration:** guided profile with legal/business name, TIN, sector and validated completeness.
- **Historical data encoding:** controlled entry/import of purchases, expenses, tax history and revolving VAT/carry-forward records, with period and source evidence.
- **Invoice collection:** local sales and import/export invoice register, manual entry/import, attachments, duplicate checks and review. The source proposal's field list includes company/individual name, TIN, quantity, amount before VAT, VAT, MRC, invoice number, date, year and remark. “MRC” is undefined and must be clarified before implementation.
- **VAT reporting:** draft VAT schedules from approved invoice/history data, drill-down to source rows, exception review and PDF/spreadsheet export.
- **Imported-goods costing:** traceable cost worksheet; its detailed input schema remains open because the source proposal explicitly requests an additional brief.
- **Selling-price analysis:** optional management decision support, not an audit opinion or guaranteed “optimal” price; defer from the first workflow until scope and calculation assumptions are agreed.
- **Staff/task management:** role-based staff profiles, task assignment to individuals/groups, priority, due date, progress and completion reports. Task completion is not professional approval.

### 5.6 Tax
- Taxpayer and tax-period profiles.
- Configurable, versioned tax rules with effective dates and authoritative source references.
- VAT, income tax, withholding, PAYE, turnover tax and excise modules as applicable.
- Reconciliation between accounting records and tax schedules.
- Draft calculations, review, approval, export and submission tracking.
- Filing receipts, payment references, amendments and status history.
- Manual filing support as an initial capability; electronic filing only through verified and authorized interfaces.

### 5.7 Client collaboration
- Firm-issued document and information requests.
- Secure uploads and resumable transfer.
- Questionnaires, structured responses, clarification threads and messages.
- Client review/confirmation of selected records.
- Explicit authorization before access to reports or sensitive workspaces.
- Submission receipt and review status visibility.

### 5.8 Documents and reports
- PDF, XLSX/CSV, DOCX, scans and photographs.
- Safe file validation, malware scanning, provenance and checksums.
- OCR/extraction with human verification; Amharic and English document support is required, while initial UI language is English.
- Version history and retention policy.
- Templates for financial statements, audit reports, management letters, VAT schedules and supporting registers.
- Export to PDF and editable formats where appropriate.
- Every generated report records its period, template/rule version, data snapshot and approval state.

## 6. Progressive delivery and requirement discovery

The product will be developed in small demonstrable increments. Each increment must provide a usable workflow, not merely disconnected screens.

For each workflow, the team will observe a practitioner completing a real task using synthetic or properly de-identified data. The team records trigger, actor, inputs, steps, judgment, output, exceptions, evidence, approval and offline needs. The team then demonstrates the proposed workflow back to practitioners and records corrections.

Auditors are not expected to define database schemas, APIs or technical architecture. The product/engineering team translates observed practice into requirements and confirms that translation with the auditor.

Every increment must include:
- A named user/problem and measurable outcome.
- Role and authorization rules.
- Input fields with definitions, requiredness and examples.
- Workflow states and allowed transitions.
- Validation, exceptions and correction paths.
- Evidence and audit-trail requirements.
- Offline and synchronization behavior where applicable.
- Acceptance examples and negative tests.
- Demonstration and feedback record.
- Explicit deferred items and known limitations.

### 6.1 First demonstrable workflow recommendation
1. Create/select a client and complete its profile.
2. Open a reporting period or engagement workspace.
3. Capture/import a small invoice batch and attach source documents.
4. Review duplicate, missing and inconsistent records.
5. Generate a draft VAT schedule with drill-down to invoices.
6. Record reviewer adjustments and approval.
7. Export the schedule and supporting invoice register.
8. Reopen the period offline and after synchronization to verify traceability and persistence.

This is a candidate first workflow; the first invoice class and tax treatment must be confirmed with pilot practitioners. Use synthetic/de-identified data until real-data gates are approved.

## 7. Non-functional requirements and invariants

### Offline reliability
- Core assigned-workspace operations remain available without internet.
- Local writes are transactional and durable.
- Synchronization resumes after restart and network interruption.
- Users can see local/cloud status, pending items and conflicts.
- Local access is subject to a bounded offline authorization lease.

### Financial integrity
- Decimal arithmetic for monetary values.
- Double-entry validation and balanced posted journals.
- Immutable posted records with reversal workflows.
- Period locks and controlled reopening.
- Reproducible calculations and reports.
- Explicit currency, precision and rounding policies.

### Security and privacy
- Tenant and engagement isolation; least privilege.
- Encryption in transit and at rest; protected local key storage.
- Device revocation and bounded offline access.
- Append-only/tamper-evident audit events.
- Data minimization, retention and export controls.
- Security monitoring and incident response.

### Performance and scale
- Initial capacity assumptions are hypotheses to be load-tested.
- Local search and ordinary desktop workflows must not depend on server round trips.
- Large document transfers must be resumable.
- Server capacity must be measured under realistic sync bursts and reporting loads.

### Maintainability
- Modular domain boundaries.
- Automated unit, integration, migration, contract and end-to-end tests.
- Reproducible builds and signed releases.
- Documented APIs, schemas and architecture decisions.
- Backward-compatible sync protocol evolution or explicit migration handling.

## 8. Out of scope for initial increments
- Autonomous audit opinion issuance.
- Autonomous posting of professional accounting entries without required authorization.
- Autonomous tax filing without professional approval and explicit authority.
- Unverified direct government API access.
- Real-time collaborative editing of the same local accounting database.
- Full ERP, banking core or payroll disbursement infrastructure.
- Selling-price optimization until its advisory purpose and calculation model are specified.

## 9. Product success measures
- Pilot practitioners complete selected workflows without hidden manual workarounds.
- Core desktop workflows complete during prolonged offline tests.
- No acknowledged posted financial transaction is lost in fault-injection tests.
- All material sync conflicts are surfaced and resolved through authorized workflows.
- Cross-tenant access tests pass.
- Accounting calculations reconcile with independently prepared expected results.
- Audit evidence is traceable to procedures and conclusions.
- Tax outputs are reproducible against approved test cases.
- Backup restoration meets agreed recovery objectives.
- Feedback results in traceable, prioritized requirement changes.

## 10. Requirements governance
Each requirement receives a stable ID, priority, owner, status, implementation reference and test evidence. Changes to accounting, audit methodology, tax rules, security or data ownership require impact analysis and the appropriate qualified approval. A feature is not complete solely because its UI exists.

Requirement statuses: **Proposed → Accepted for increment → In development → Demonstrated → Verified/Accepted**, or **Deferred/Rejected** with rationale. A demonstrated prototype is not automatically production-approved.
