# PR #2 Binding Decision — Prototype Completion Contract for Increments 0–5

**Status:** Binding product-owner instruction  
**Applies to:** The existing `implementation/increments-0-to-5` branch and PR #2 only  
**Product stage:** Controlled prototype build using synthetic data  
**Acceptance:** One collective verification and test event after Increment 5 implementation is complete

## 1. Binding decision

The current ARM application is a **prototype build**. The purpose of the current work is to close the outstanding implementation gaps across the already-defined Increments 0–5 to the best practical professional standard. The implementation must remain coherent, usable, traceable, and suitable for practitioner demonstrations with synthetic data.

This is not authorization for production operation, real client data, live accounting or tax services, regulated report issuance, public distribution, or government filing.

## 2. Branch and PR binding

- All work remains on branch `implementation/increments-0-to-5`.
- All commits remain in PR #2, targeting `main`.
- Do not create another implementation branch or another PR for this work.
- Keep PR #2 open and draft while implementation is in progress.
- Do not merge or close PR #2 as part of this work.
- Preserve existing working behavior and history. Use additive, forward-only migrations and backward-conscious changes where possible.
- Before every write, confirm the current branch head and use the current file/blob SHA. Never force-push or rewrite history.

## 3. Prototype-first implementation rule

Implement the best complete prototype behavior that can be supported safely by the available evidence and established domain practice.

- Prefer a complete, demonstrable workflow over a disconnected screen or placeholder.
- Use explicit synthetic/demo identities and data. Clearly label simulated or local-only behavior.
- Do not claim cloud synchronization, production authentication, tenant isolation, encryption, OCR quality, tax compliance, customs compliance, or production readiness unless those capabilities actually exist and have been proven at the later collective gate.
- Where an external integration is not yet available, implement a real adapter boundary, job lifecycle, validation contract, and honest unavailable/error state. Do not fake successful processing.
- Where a professional rule is not yet validated for Ethiopia, implement configurable inputs and transparent draft calculations only where the method is supported; otherwise preserve the workflow as review-required and visibly identify the unresolved rule.
- Do not silently invent accounting, audit, tax, customs valuation, foreign-exchange, or import-cost assumptions.
- Do not expand into unrelated ERP, inventory, payroll, banking, filing, or CRM functionality.

## 4. Verification and testing timing — explicit product-owner override

The product owner has directed that **formal verification and testing of the Increments 0–5 delivery take place after Increment 5 implementation is complete**.

Accordingly:

1. During implementation, prioritize completing the connected workflows and closing identified gaps.
2. Do not run or report increment-level acceptance, sign-off, pass/fail, or release decisions.
3. Do not declare an increment complete, accepted, passed, or independently verified.
4. Do not use isolated smoke checks or partial test results as acceptance evidence.
5. Once Increment 5 implementation is complete, run the consolidated verification and test program across the integrated Increments 0–5 system.
6. The collective gate must cover clean setup, migrations, persistence/restart, all screens and commands, roles/personas, workflow transitions, provenance, source evidence, import reconciliation, calculations, failure paths, security boundaries, Windows build/runtime, and regression.
7. Record one overall result only: `NOT READY`, `READY FOR PRACTITIONER ACCEPTANCE`, or `ACCEPTED FOR CONTROLLED SYNTHETIC-DATA USE`.
8. A failed criterion keeps the collective result at `NOT READY`. Fix the defect and repeat the collective suite.
9. This timing rule does not permit suppressing tests, deleting tests, weakening assertions, or claiming that untested behavior works. Test design and test fixtures may be prepared during implementation; execution and formal verification are deferred until after Increment 5 is implemented.

## 5. Increments 0–5 work boundaries

The current increment definitions remain the implementation scope:

- **Increment 0:** Local desktop shell, workspace, safe local persistence, migrations, synthetic data, notes/activity and restart durability.
- **Increment 1:** Client/company registration, validation, duplicate handling, lifecycle, archive/restore and change history.
- **Increment 2:** Client-scoped product/category register and sales/purchase invoice capture, source metadata, duplicate review and workpaper linkage.
- **Increment 3:** Historical data capture/import, source retention, row-level provenance, period assignment, reconciliation and draft VAT workpaper organization.
- **Increment 4:** Staff/task workflow, assignments, due-state/workload views, preparer/reviewer workflow, review notes and task history.
- **Increment 5:** Import-cost worksheet prototype, evidence capture, cost component structure, allocation model, transparent calculations where the method is supported, and explicit review/exception states.

Do not silently change the meaning of these increments. If a gap is cross-cutting, implement it once in the shared foundation and connect the relevant workflows.

## 6. Domain and international practice to apply

Use established international models as design references, not as a substitute for Ethiopian law or professional judgment.

### 6.1 Audit evidence and workpaper traceability

Use the principles in IAASB ISA 230 (Audit Documentation) and ISA 500 (Audit Evidence) as workflow design references:

- An experienced practitioner should be able to understand the work performed, evidence obtained, results, significant matters, conclusions, and professional judgments.
- Retain identifying characteristics of items tested, who performed the work, when it was completed, who reviewed it, and the extent/date of review.
- Preserve evidence that supports and contradicts a conclusion.
- Keep source evidence linked to records and workpapers.
- Keep preparer and reviewer identities and timestamps.
- Record review findings, responses, resolution, and unresolved exceptions.

These are product design controls; the prototype must not claim ISA compliance or replace firm methodology.

### 6.2 Financial statement boundaries

Use IAS 1 as a reference for the distinction between a complete financial statement set and limited monthly summaries. A complete set is not just an invoice total or VAT schedule. Do not label partial source summaries as complete financial statements or IFRS-compliant outputs.

### 6.3 Ethiopian tax configuration

Use current official Ethiopian legal publications as the authoritative source of jurisdiction-specific tax rules. At the time this contract was prepared, official references include:

- Value Added Tax Proclamation No. 1341/2024.
- Council of Ministers Value Added Tax Regulation No. 570/2025.

Tax rules must be stored as versioned, effective-dated configuration with legal citation, applicable period, taxable event, scope, conditions, calculation basis, rounding, exceptions, and approval metadata. Do not hardcode a generic rate as if it applies to every taxpayer, supply, period, or transaction. The prototype may organize source data and produce explicitly draft/review-required calculations only where the rule has been validated. No tax filing or official compliance claim.

### 6.4 Import-cost model

Keep three concepts separate:

1. **Commercial purchase cost:** amounts paid or payable to the supplier under the purchase contract.
2. **Customs value:** a legal valuation basis used for customs purposes, determined under the applicable national law and valuation method.
3. **Landed/acquisition cost:** the importer's actual cost of bringing the goods to the relevant location or condition, based on the firm's approved accounting policy.

Use the WTO Customs Valuation Agreement and WCO explanatory material as the international customs valuation reference:
- Transaction value is the primary method when its conditions are met.
- Customs value may require specific adjustments; invoice price is not automatically customs value.
- Alternative valuation methods follow the legally prescribed hierarchy.
- Transport/freight treatment depends on the importing country's law.

Use Incoterms 2020 as a commercial responsibility reference for identifying which party arranges or bears carriage, insurance, delivery, and related costs. Incoterms do not determine customs value, tax treatment, or accounting capitalization by themselves.

The prototype worksheet should capture, as separate and source-linked cost components where relevant:
- Supplier invoice amount and currency.
- Foreign exchange rate, source, date, and conversion basis.
- Freight and transport legs.
- Insurance.
- Loading, handling, terminal, and port charges.
- Customs brokerage and agency fees.
- Customs duties and other import charges.
- Non-recoverable taxes/levies where the accounting policy permits inclusion.
- Recoverable VAT separately from cost.
- Inland transport and delivery.
- Bank and transfer charges, where policy permits.
- Other directly attributable costs.
- Credits, refunds, rebates, and adjustments.
- Supporting document, payer, amount, currency, date, allocation basis, and review status.

Support allocation of shared costs across goods using an explicitly selected and recorded basis such as quantity, weight, volume, value, or another justified driver. Do not choose a hidden default. Show the allocation base and arithmetic for each line. Prevent double counting. Distinguish recoverable VAT from costs. Do not equate the worksheet's landed cost with customs value or tax base.

Because Ethiopia-specific customs valuation, FX, tax, and accounting treatment must be validated against applicable law and practitioner policy, unresolved components must remain configurable and flagged for review rather than silently assumed.

### 6.5 Multi-tenant and access patterns

Even though this is a local synthetic prototype, model ownership and access boundaries so the system does not need a conceptual rewrite later:
- Firm/workspace context is explicit.
- Every client-owned record has a client owner reference.
- Staff assignments and review roles are explicit.
- Access checks are centralized, not scattered UI-only checks.
- Demo personas are clearly distinguished from production identities.
- Future server authorization must not trust client-supplied owner IDs.

Use Microsoft Azure and AWS multi-tenant architecture guidance as design references. Authentication alone is not tenant isolation; every resource access must be scoped to the active tenant. The prototype must not claim production tenant isolation while it remains local-only and uses demo personas.

### 6.6 Import and data-quality patterns

Use controlled import practices:
- Preview before commit.
- Explicit column mapping.
- Type and required-field validation.
- Duplicate detection with human disposition.
- Atomic batch commit or explicit partial-row outcomes.
- Per-row provenance to source file, batch, and row.
- Reconciliation of source rows to accepted, rejected, skipped, and duplicate rows.
- Preserve original source and import configuration.
- Make re-import idempotent or detect prior batch reuse.

## 7. Implementation priorities

Close gaps in this order, while maintaining one connected application:

1. Correct and complete shared domain/persistence foundations and migration safety.
2. Finish client registration lifecycle and data quality.
3. Complete product/category/invoice workflow with source evidence and review.
4. Complete historical imports, row provenance, period assignment and reconciliation.
5. Complete staff tasks, assignments, review and history.
6. Implement the import-cost worksheet with separate commercial/customs/accounting views and transparent allocation.
7. Connect all increments through navigation, client context, shared status, and activity history.
8. Prepare one integrated collective verification matrix and synthetic fixtures for execution after Increment 5.

## 8. Required delivery evidence after Increment 5

At the collective gate, produce:
- Requirement-to-screen-to-command-to-schema traceability matrix.
- Final data model and migration inventory.
- Demonstration script and synthetic dataset.
- Test report covering all increments together.
- Import/OCR/costing limitations and evidence.
- Audit trail and provenance demonstration.
- Security and data-boundary findings.
- Windows build/runtime evidence.
- Known defects and deferred professional decisions.
- Overall collective gate decision.

## 9. Source references

- IAASB, 2025 Handbook, ISA 230 — Audit Documentation and ISA 500 — Audit Evidence: https://ifacweb.blob.core.windows.net/publicfiles/2025-09/IAASB-2025-Handbook-Volume-1.pdf
- IAASB, 2025 Handbook publication record: https://www.iaasb.org/publications/2025-handbook-international-quality-management-auditing-review-other-assurance-and-related-services
- IFRS Foundation, IAS 1 — Presentation of Financial Statements: https://www.ifrs.org/issued-standards/list-of-standards/ias-1-presentation-of-financial-statements/
- Ethiopian Ministry of Justice, VAT Regulation No. 570/2025: https://justice.gov.et/en/law/570-2025-value-added-tax-regulation/
- Ethiopian Ministry of Finance, VAT Proclamation No. 1341/2024: https://www.mofed.gov.et/media/filer_public/af/45/af45af2f-7959-4e8b-b736-9494dda9f017/vat_proclamation_no_1341-2016_with_annex.pdf
- WTO, Agreement on Implementation of Article VII (Customs Valuation): https://www.wto.org/english/docs_e/legal_e/20-val_01_e.htm
- World Customs Organization, Customs Valuation FAQs: https://www.wcoomd.org/en/topics/valuation/overview/faq-valuation.aspx
- ICC, Incoterms Rules: https://iccwbo.org/business-solutions/incoterms-rules/
- Microsoft Azure Architecture Center, SaaS and Multitenant Architecture: https://learn.microsoft.com/en-us/azure/architecture/guide/saas-multitenant-solution-architecture/
- AWS, Guidance for Multi-Tenant Architectures: https://docs.aws.amazon.com/solutions/multi-tenant-architectures-on-aws/

**Final instruction:** Proceed without asking the product owner about routine design or implementation choices. Preserve the prototype boundary, use synthetic data, close the Increments 0–5 gaps in the existing branch and PR, and defer formal verification and testing until Increment 5 implementation is complete.
