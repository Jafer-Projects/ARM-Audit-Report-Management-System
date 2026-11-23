# Ethiopian Regulatory and Government Integration Strategy

## 1. Objective

Prepare ARM for Ethiopian tax and revenue workflows without assuming that a public API, sandbox, certification route or permission exists. The integration layer must isolate external system changes from accounting, audit and tax domain logic.

## 2. Integration principles

- Verify official documentation, legal authority, access terms and certification before implementation.
- Prefer official government interfaces and approved channels.
- Do not reverse engineer or automate a protected portal in violation of its terms.
- Never store taxpayer credentials in plain text or ask users to share passwords with ARM.
- Separate electronic invoicing from tax-return filing.
- Treat submission as pending until an official acknowledgment/receipt is obtained.
- Preserve request/response evidence, correlation IDs, timestamps and external references with sensitive data minimized.
- Support manual filing and export as a first-class workflow.
- Use sandbox/test environments before production.
- Maintain a versioned adapter for each external platform.

## 3. Initial tax preparation capability

Before any direct integration is available, ARM should:
- Maintain taxpayer profiles and tax periods.
- Prepare applicable VAT, income tax, withholding, PAYE, turnover and excise schedules.
- Reconcile calculations to the ledger and source evidence.
- Generate required reports and supporting schedules based on verified current specifications.
- Export files or print-ready packages where officially accepted.
- Track due dates, preparer/reviewer approvals, manual submission and payment references.
- Record receipts uploaded by the user and distinguish user-provided proof from independently verified status.

## 4. Electronic invoicing

E-invoicing must be its own domain adapter. Investigate official Ethiopian Electronic Invoice Management System requirements, taxpayer eligibility, device/software registration, invoice schema, authentication, signatures, offline behavior, certification, sandbox and production onboarding. Do not infer an API contract from the existence of a government e-invoice portal.

Where official integration is unavailable, ARM may support preparation/import/export workflows only to the extent permitted by published requirements and terms.

## 5. Tax return filing

Treat filing as a separate adapter from e-invoicing. Confirm whether authorized APIs exist for return submission, account data, payment references, status polling and receipts. Design each operation with idempotency, explicit approval, retry safety and reconciliation of uncertain outcomes.

If a request times out after submission, do not blindly resubmit. Query status or route to manual reconciliation using the external reference to avoid duplicate filings.

## 6. Adapter contract

Each adapter should define:
- External system and version.
- Supported operations.
- Eligibility and authorization.
- Credential/certificate lifecycle.
- Request/response schemas.
- Idempotency and duplicate behavior.
- Rate limits and retry policy.
- Error classification.
- Sandbox and certification status.
- Receipt/acknowledgment model.
- Audit evidence and retention.
- Operational owner and incident contact.

## 7. Regulatory change management

Maintain a regulatory register with source URL/document, issuing authority, publication/effective dates, affected tax types, reviewer, interpretation, implementation ticket, test vectors and release date. A proposed rule change must not become active until reviewed and approved. Support historical calculations using the rule version effective for the relevant period.

## 8. Legal and professional validation

Before production filing or invoicing, obtain written confirmation from qualified Ethiopian tax/legal professionals and the relevant authority or authorized integration program regarding:
- ARM's role and permitted activity.
- Taxpayer/agent authorization.
- Electronic signature and credential requirements.
- Recordkeeping and retention.
- Data residency and cross-border processing.
- E-invoice certification and approved software requirements.
- Filing formats and APIs.
- Amendment, cancellation and audit trail rules.

This specification does not assert that any named API is publicly available or that ARM is approved for government submission.
