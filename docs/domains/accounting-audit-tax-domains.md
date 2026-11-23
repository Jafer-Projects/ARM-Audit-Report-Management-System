# Accounting, Audit and Tax Domain Specifications

## 1. Domain separation

Accounting, audit and tax are separate bounded domains. They exchange controlled, versioned data and references. Audit procedures must not mutate accounting books. Tax calculations must not silently post accounting entries. Any adjustment originating from an audit or tax workflow becomes a separately approved accounting journal.

## 2. Accounting engine

### Core capabilities
- Chart of accounts with account type, code, description and effective dates.
- Fiscal years and accounting periods.
- Journal drafts, review, posting and reversal.
- General ledger and trial balance.
- Receivables/payables subledgers.
- Bank statement import, matching and reconciliation.
- Payroll journals and liabilities.
- Inventory valuation and movement accounting.
- Fixed asset acquisition, depreciation and disposal.
- Budget and variance schedules.
- Financial statement mapping and report generation.

### Posting rules
- Debits and credits balance for each journal under the selected currency/accounting policy.
- Posting is atomic and authorized.
- Posted records are immutable.
- Reversals reference the original entry and preserve reason/approval.
- Period locks are enforced by the server and local domain engine.
- Duplicate import detection uses source identity, hashes and configurable matching—not only amount/date.
- Every journal retains source documents and preparer/reviewer identity where applicable.

### Multi-currency
Record transaction currency, functional/reporting currency, rate, source, date and realized/unrealized treatment. Avoid silently changing historical rates. Currency precision and rounding are explicit configuration, validated against applicable accounting requirements.

## 3. Audit engine

### Engagement lifecycle
1. Prospect/client acceptance and continuance.
2. Independence and conflict assessment.
3. Engagement letter and scope approval.
4. Planning, materiality and risk assessment.
5. Understanding entity and internal controls.
6. Audit strategy, programme and assignment.
7. Evidence collection and procedure execution.
8. Sampling and substantive testing.
9. Exceptions, findings and proposed adjustments.
10. Management responses and subsequent events.
11. Review notes and partner sign-off.
12. Completion, report authorization, issuance and archiving.

### Traceability graph
Support links between:
- Financial statement line/assertion.
- Risk and relevant control.
- Audit objective and procedure.
- Population and sample.
- Evidence item and test result.
- Exception and finding.
- Proposed adjustment and management response.
- Reviewer conclusion and report section.

A material conclusion without supporting evidence or an explicit documented rationale must be flagged for review.

### Specialized engagements
- Internal audit: audit universe, risk-based plan, control design/operating effectiveness, recommendations and action follow-up.
- Compliance: requirement/control mapping, test evidence, exceptions and remediation.
- Tax audit/review: tax schedules, return-to-ledger reconciliation and supporting evidence.
- NGO/donor-funded audit: grant/project dimensions, donor budget, eligible expenditure, procurement evidence, restrictions, fund utilization and donor-specific reporting.
- Financial statement audit: applicable ISA methodology and firm-approved templates.

### Professional judgment
The system may guide and automate defined procedures, but the auditor remains responsible for judgments, sufficiency/appropriateness of evidence and opinion. Do not hardcode a generic audit opinion decision tree as a substitute for professional methodology.

## 4. Tax engine

### Supported initial tax areas
- VAT.
- Business income tax.
- Withholding tax.
- PAYE.
- Turnover tax.
- Excise where applicable.

### Rule architecture
Each rule version must include:
- Tax type and jurisdiction.
- Effective-from and effective-to dates.
- Taxpayer/entity applicability.
- Tax base and calculation method.
- Rates, thresholds, exemptions and exceptions.
- Required source/reference and publication date.
- Interpretation/approval status.
- Test vectors and expected results.
- Version author, reviewer and release date.

Do not populate rates or legal interpretations from unverified summaries. Tax professionals and qualified Ethiopian legal/tax reviewers must approve each production rule set. Historical returns retain the exact rule version used.

### Tax workflow
1. Select taxpayer and tax period.
2. Confirm registrations and applicable tax profile.
3. Import/derive accounting and source data.
4. Reconcile tax base to ledger and supporting schedules.
5. Apply effective-dated rules.
6. Show calculations, exceptions and assumptions.
7. Prepare return and attachments.
8. Independent review and taxpayer/authorized firm approval as required.
9. Manual export/submission or authorized electronic filing.
10. Record official receipt, payment reference, status and amendments.

## 5. Automation and AI

Permitted assistance:
- OCR and document extraction.
- Bank transaction classification suggestions.
- Duplicate invoice and anomaly indicators.
- Sampling support with reproducible parameters and seed.
- Reconciliation suggestions.
- Risk indicators.
- Draft workpaper summaries and report language.
- Tax calculation explanations from approved rules.

Controls:
- Label generated suggestions.
- Preserve source passages/data references.
- Record model/tool/version and material prompts or processing metadata where appropriate.
- Require human verification of extracted values.
- Do not train on confidential client data without an explicit lawful basis and appropriate agreement.
- Never autonomously issue audit opinions, post material adjustments, approve reports or submit returns.
- Provide deterministic non-AI fallback for critical calculations and workflows.

## 6. Reporting

Reports must be generated from approved data and versioned templates. Include reporting period, entity, currency, framework, preparation date, preparer, reviewer, approval status and data/rule snapshot references. Drafts must be visibly distinguished from approved and issued reports. Corrections after issue create a new version and controlled reissue history.
