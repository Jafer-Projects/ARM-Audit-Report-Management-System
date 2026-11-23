# ARM Implementation Roadmap and Acceptance Criteria

## 1. Delivery strategy

Deliver complete, usable workflows in small increments. Detailed requirements are progressively elaborated through practitioner demonstrations and feedback; auditors are not expected to author technical specifications. Each increment must preserve evidence, authorization, financial integrity and offline durability. Calendar estimates are hypotheses and must be recalibrated after team capacity and pilot scope are known.

## 2. Phase 0 — Approved for controlled initial development

The product owner approved Phase 0 for controlled initial development on 2026-11-23. This authorizes synthetic-data implementation and scoped technical spikes. It does not close the legal, security, accounting, tax or operations evidence gates, and does not authorize real client data or production.

See `docs/governance/phase-0-gate-record.md` for permitted/prohibited activities and gate status.

## 3. Incremental initial development

The first increments are defined in [Phase 1 incremental development and feedback](phase-1-incremental-development-and-feedback.md). The initial sequence is:

1. Working shell and safe local workspace.
2. Client/company registration.
3. Product/category register and invoice capture.
4. Historical data and draft VAT workpaper.
5. Staff tasks and review workflow.
6. Import-cost worksheet discovery and prototype only after required cost-input brief is obtained.

Demonstrate each vertical slice for feedback and run its relevant tests continuously, but do not accept or pass increments individually. All Increments 0–5 must be implemented on branch `implementation/increments-0-to-5` and PR #2. The single collective acceptance decision occurs only after Increment 5, when the integrated system is tested against every Increments 0–5 requirement and cross-increment workflow. A failure in any criterion blocks the collective gate. Separate legal, security, professional and production-release gates remain in force.

## 4. Broader product phases

### Phase 2 — Accounting foundation
- Chart of accounts and periods.
- Journals, ledger and trial balance.
- Posting/reversal and period lock.
- Bank import and reconciliation.
- Receivables/payables.
- Financial statement mapping.
- Independent expected-result fixtures and reconciliation tests.

Exit: representative accounting cycle reconciles to independently verified results; posted journals balance and are immutable.

### Phase 3 — Audit management
- Acceptance, continuance and independence.
- Engagement letters and staffing.
- Planning, materiality, risks and controls.
- Audit programmes and procedures.
- Evidence and workpaper management.
- Sampling and testing.
- Findings, adjustments and review notes.
- Partner authorization and report issuance.

Exit: representative engagement has complete evidence-to-conclusion traceability and qualified review sign-offs.

### Phase 4 — Client collaboration and tax preparation
- Client invitation and scoped access.
- Requests, uploads, questionnaires and messages.
- Document receipt states and resumable transfer.
- Tax profiles, periods and effective-dated rule engine.
- VAT, income, withholding, PAYE and turnover tax; excise where applicable.
- Review, approval, export and manual filing tracking.

Exit: client submits requested source records and professional prepares/reviews a reproducible tax package using approved rules.

### Phase 5 — Expanded operations and integrations
- Payroll, inventory and fixed assets.
- NGO/donor-funded audit schedules.
- Advanced analytics and AI-assisted extraction.
- Official e-invoice/tax adapter only after verified authorization and technical documentation.
- Subscription billing and entitlement lifecycle.
- Enhanced reporting and management analytics.

Exit: integration authorization/certification completed where required; external receipts and uncertain submission states handled safely.

### Phase 6 — Production hardening and scale
- Independent security assessment and remediation.
- Load, soak, offline and fault-injection testing.
- Disaster recovery and restore exercises.
- Desktop update rollback and compatibility.
- Monitoring, alerting, incident response and support playbooks.
- Pilot feedback and release governance.

Exit: release gates pass, operational owners assigned, backup restore demonstrated and pilot acceptance approved.

## 5. Mandatory release gates

### Offline and sync
- Extended offline operation.
- App crash/power loss at each sync stage.
- Duplicate event delivery.
- Partial upload and resume.
- Server outage and recovery.
- Expired offline lease and revoked device.
- Conflict preservation and resolution.
- Old-client/new-server protocol compatibility.

### Accounting
- Balanced journal invariant.
- Decimal precision and rounding fixtures.
- Reversal and closed-period behavior.
- Duplicate import detection.
- Currency and exchange-rate reproducibility.
- Concurrency and transaction atomicity.

### Audit
- Evidence-to-procedure-to-finding-to-report traceability.
- Review-note lifecycle.
- Separation of preparer/reviewer/partner roles.
- Report draft versus issued status.
- Reproducible sample and test results.

### Tax
- Effective-dated rule tests.
- Independent professional validation.
- Ledger-to-return reconciliation.
- Rounding and amendment tests.
- Filing timeout and duplicate submission scenarios.
- Historical return reproducibility.

### Security
- Cross-tenant IDOR and search tests.
- Role/engagement boundary tests.
- Device revocation and offline lease tests.
- Document upload malware and path traversal tests.
- Secrets and dependency scanning.
- Privileged support access audit.
- Backup access and restore security.

## 6. Pilot and real-data boundary

Start with a small, controlled practitioner cohort and representative clients only after the applicable real-data gates are approved. Select workflows such as client onboarding, invoice capture, VAT workpaper review, offline bookkeeping, bank reconciliation, audit evidence collection and report generation. Compare outputs against independently prepared records. Maintain a feedback and incident process.

Synthetic data is required for development and ordinary demos. Real client data requires approved privacy/legal roles and contracts, security assessment, retention, access controls, backups and firm authorization. Government filing requires its own verified authorization and official process.

## 7. Definition of done

For the initial Increments 0–5 programme, the criteria below are evaluated as one integrated acceptance decision after Increment 5. They are not increment-by-increment release gates.

A feature is complete only when:
- Requirements and role boundaries are documented.
- Server and local validation exist where applicable.
- Data migrations and rollback/compatibility strategy are tested.
- Automated tests cover normal, negative, offline and authorization paths.
- Material actions emit audit events.
- UI exposes errors, draft/review/approval states and pending sync honestly.
- Accessibility and usability are reviewed with practitioners.
- Documentation, acceptance evidence and operational notes are updated.
- CI passes and acceptance evidence is attached to the work item.
