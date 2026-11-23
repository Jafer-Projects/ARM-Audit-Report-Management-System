# ARM Implementation Roadmap and Acceptance Criteria

## 1. Delivery strategy

Deliver a coherent, tested professional workflow in increments. The schedule below is indicative for a moderate-budget team and must be recalibrated after team composition, regulatory discovery and technical spikes are known. Do not treat calendar estimates as a promise.

## 2. Phases

### Phase 0 — Discovery and specification
- Confirm workflows with Ethiopian auditors, accountants, tax professionals and client businesses.
- Validate standards, tax obligations, retention and professional requirements.
- Define data classification, firm/client contractual model and threat model.
- Create domain glossary, process maps, data dictionary and API contracts.
- Prototype SQLite encryption, Tauri native bridge and sync conflict handling.

Exit: approved SRS, domain model, threat model, architecture decisions and acceptance tests.

### Phase 1 — Offline desktop and platform foundation
- Repository/toolchain, CI, coding standards and dependency policy.
- Tauri Windows shell and signed installer pipeline.
- React/TypeScript design system and role-aware navigation.
- Identity, firm membership, device registration and offline lease.
- SQLite migrations, encryption/key protection and local vault.
- Cloud API, PostgreSQL tenancy model and sync inbox/outbox.
- Basic client/engagement workspace and document metadata.
- Network interruption, restart and conflict tests.

Exit: a user can open an assigned workspace, create local drafts and documents offline, reconnect, synchronize, inspect status and resolve a controlled conflict without data loss.

### Phase 2 — Accounting foundation
- Chart of accounts and periods.
- Journals, ledger and trial balance.
- Posting/reversal and period lock.
- Bank import and reconciliation.
- Receivables/payables.
- Financial statement mapping.
- Independent expected-result fixtures and reconciliation tests.

Exit: complete representative accounting cycle reconciles to independently verified results; all posted journals balance and are immutable.

### Phase 3 — Audit management
- Acceptance, continuance and independence.
- Engagement letters and staffing.
- Planning, materiality, risks and controls.
- Audit programmes and procedures.
- Evidence and workpaper management.
- Sampling and testing.
- Findings, adjustments and review notes.
- Partner authorization and report issuance.

Exit: representative financial audit can be executed with complete evidence-to-conclusion traceability and review sign-offs.

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

Exit: integration certification/authorization completed where required; external receipts and uncertain submission states handled safely.

### Phase 6 — Production hardening and scale
- Independent security assessment and remediation.
- Load, soak, offline and fault-injection testing.
- Disaster recovery and restore exercises.
- Desktop update rollback and compatibility.
- Monitoring, alerting, incident response and support playbooks.
- Pilot feedback and release governance.

Exit: all release gates pass, operational owners assigned, backup restore demonstrated and pilot acceptance approved.

## 3. Indicative timeline

A 12–18 month programme is a planning hypothesis, not a guarantee. A smaller team may need longer. The first pilot should be limited to a small group of professionals and client businesses and should not depend on live government API access.

## 4. Mandatory release gates

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

## 5. Pilot plan

Start with approximately 5–10 professionals and a small number of consenting client businesses. Select representative workflows: client onboarding, offline bookkeeping, bank reconciliation, audit evidence collection, review, tax preparation and report generation. Compare outputs against independently prepared records. Use appropriately authorized data and a documented incident/feedback process.

## 6. Definition of done

A feature is complete only when:
- Requirements and role boundaries are documented.
- Server and local validation exist where applicable.
- Data migrations and rollback/compatibility strategy are tested.
- Automated tests cover normal, negative, offline and authorization paths.
- Audit events are emitted for material actions.
- UI exposes errors and pending states honestly.
- Accessibility and usability are reviewed.
- Documentation and operational runbooks are updated.
- CI passes and acceptance evidence is attached to the work item.
