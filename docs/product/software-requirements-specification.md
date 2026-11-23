# ARM Software Requirements Specification

## 1. Purpose

This document defines the product scope and baseline functional and non-functional requirements for ARM (Audit Report Management System), an offline-first auditing, accounting and tax practice platform for Ethiopia.

It is a requirements baseline for design, estimation, implementation and verification. It is not legal advice, an audit methodology certification or confirmation that a government integration interface is available.

## 2. Product vision

Provide professional firms and independent practitioners with one dependable environment to manage client accounting, audit engagements, tax preparation, evidence, approvals and reporting. The Windows desktop client must support substantive work during unreliable or absent connectivity. Cloud services provide centralized control, collaboration, synchronization, client access and authorized integrations.

## 3. User groups

- Independent auditors and audit partners.
- Professional accountants and bookkeepers.
- Tax professionals.
- Firm owners, engagement managers and reviewers.
- Client business representatives and authorized employees.
- Platform operations and security administrators.

Government officers are not an initial user group. Any regulator-facing access requires a separately authorized, explicitly scoped integration or portal.

## 4. Product surfaces

### 4.1 Windows professional desktop

The primary work surface for auditors and accountants. It must support local engagement/accounting data, document access, workpaper preparation, accounting operations, calculations, report drafting and offline task management.

### 4.2 Client web portal

Responsive browser application for firm-invited client users. Supports secure document submission, requests, questionnaires, clarifications, authorized review/confirmation, messages and access to released deliverables.

### 4.3 Firm management workspace

Web and/or desktop-accessible workspace for firm administration, user assignments, engagement oversight, review queues, subscription seats, client access and reporting.

### 4.4 Platform administration

Restricted operational workspace for platform support, tenant lifecycle, subscription administration, security monitoring and system configuration. Platform administrators do not automatically receive access to client content; exceptional support access must be authorized, time-limited and audited.

## 5. Functional scope

### 5.1 Identity, organizations and licensing

- Register individual professionals and firms.
- Support users who belong to more than one firm with distinct organization-scoped permissions.
- Support individual and firm subscriptions.
- Manage invitations, seat assignments, renewals, suspension and cancellation.
- Register and revoke desktop devices.
- Support MFA, secure recovery and session management.
- Maintain professional credential information where required; verification status must distinguish self-declared from independently verified credentials.

### 5.2 Client and engagement management

- Create firm-controlled client workspaces.
- Record legal/business identity, contacts, tax identifiers and branches subject to lawful collection.
- Record engagement type, scope, period, terms, team, budget, milestones and status.
- Perform client acceptance, continuance, independence and conflict checks.
- Maintain engagement letters, approvals and change history.
- Separate client-provided records from firm-created workpapers and professional conclusions.

### 5.3 Accounting

- Chart of accounts and accounting periods.
- Journals, general ledger and trial balance.
- Accounts receivable and payable.
- Bank statement import and reconciliation.
- Payroll accounting.
- Inventory and fixed-asset accounting.
- Budgets and variance analysis.
- Financial statements and supporting schedules.
- Period close, controlled reopening and adjustment workflows.
- Multi-currency transactions and documented exchange-rate policy.

### 5.4 Audit

- Financial statement, internal, compliance, tax-related and NGO/donor-funded engagement workflows.
- Planning, materiality, risk assessment and control documentation.
- Audit programme and procedure assignment.
- Sampling and substantive testing.
- Evidence collection, indexing and cross-referencing.
- Exceptions, findings, proposed adjustments and management responses.
- Review notes, sign-offs and partner approval.
- Completion procedures, report drafting, issuance and archiving.
- Methodology and template version tracking.

### 5.5 Tax

- Taxpayer and tax-period profiles.
- Configurable, versioned tax rules with effective dates.
- VAT, income tax, withholding, PAYE, turnover tax and excise modules as applicable.
- Reconciliation between accounting records and tax schedules.
- Draft calculations, review, approval, export and submission tracking.
- Filing receipts, payment references, amendments and status history.
- Manual filing support as a complete initial capability; electronic filing only through verified and authorized interfaces.

### 5.6 Client collaboration

- Firm-issued document and information requests.
- Secure uploads and resumable transfer.
- Questionnaires and structured responses.
- Clarification threads and messages.
- Client review/confirmation of selected records.
- Explicit authorization before access to reports or sensitive workspaces.
- Submission receipt and review status visibility.

### 5.7 Documents and reports

- PDF, XLSX/CSV, DOCX, scans and photographs.
- Safe file validation and malware scanning on server intake.
- OCR/extraction with human verification.
- Version history, checksums, source attribution and retention policy.
- Templates for financial statements, audit reports, management letters and tax schedules.
- Export to PDF and editable formats where appropriate.
- Every generated report records its period, template version, data snapshot and approval state.

## 6. Non-functional requirements

### Offline reliability
- Core assigned-workspace operations remain available without internet.
- Local writes are transactional and durable.
- Synchronization resumes after application restart and network interruption.
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
- Tenant and engagement isolation.
- Least-privilege access.
- Encryption in transit and at rest.
- Protected local key storage.
- Device revocation and bounded offline access.
- Append-only/tamper-evident audit events.
- Data minimization, retention and export controls.
- Security event monitoring and incident response.

### Performance and scale
- Initial target: 100–500 registered users; concurrent usage and document volume must be benchmarked.
- Local search and ordinary desktop workflows must not depend on round trips to the server.
- Large document transfers must be resumable.
- Server capacity must be measured under realistic synchronization bursts and reporting loads.

### Maintainability
- Modular domain boundaries.
- Automated unit, integration, migration, contract and end-to-end tests.
- Reproducible builds and signed releases.
- Documented APIs, schemas and architecture decisions.
- Backward-compatible sync protocol evolution or explicit migration handling.

## 7. Out of scope for initial release

- Autonomous audit opinion issuance.
- Autonomous tax filing without professional approval and explicit authorization.
- Unverified direct government API access.
- Real-time collaborative editing of the same local accounting database.
- Replacing government systems or serving as an official government record repository.
- Full ERP, banking core or payroll disbursement infrastructure.

## 8. Product success measures

- Core desktop workflows complete during a prolonged offline test.
- No acknowledged posted financial transaction lost in fault-injection tests.
- All sync conflicts surfaced and resolved through authorized workflows.
- Cross-tenant access tests pass.
- Accounting calculations reconcile with independently prepared expected results.
- Audit evidence is traceable to procedures and conclusions.
- Tax outputs are reproducible against approved test cases.
- Backup restoration meets agreed recovery objectives.
- Pilot users complete representative engagements without relying on hidden manual workarounds.

## 9. Requirements governance

Each requirement must receive a stable identifier, priority, owner, implementation reference and test evidence. Changes to accounting, audit methodology, tax rules, security or data ownership require documented impact analysis and approval. No feature is complete solely because its UI exists.
