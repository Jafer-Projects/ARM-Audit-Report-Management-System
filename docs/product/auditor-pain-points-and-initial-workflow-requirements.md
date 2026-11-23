# Auditor pain points and initial workflow requirements

**Source document:** *Audit Report Management Software (ARMS) — Proposal for the development of the ARMS*, dated 9 January 2025, supplied by the product owner.  
**Source reference:** uploaded `auditing system.pdf`, 8 pages.  
**Status:** Accepted as an initial problem-discovery input for controlled development. It is not a complete functional specification, legal validation, tax-rule authority or approval of every proposed feature.

## 1. Why this document is retained

The proposal describes concrete operational hassles reported from an external-auditor perspective. It is valuable because it expresses the work in the auditor's own task language rather than requiring auditors to design software abstractions. The requirements below translate those pain points into initial product capabilities while preserving unresolved details as questions.

The proposal's original technology and delivery estimates are not adopted as architecture commitments. Its React/Angular, Node/Laravel/Django, PostgreSQL/MySQL and cloud-only suggestions (page 6) are superseded by the repository's offline-first architecture decision. Its 13-week schedule (page 7) is not a delivery commitment.

## 2. Source-derived pain points and product response

| Source pain point (PDF pages 2–3) | Initial product capability | Initial acceptance evidence | Open detail |
|---|---|---|---|
| Manual registration/tracking of product categories, products and invoices | Client-scoped product/category register with search, add/edit permissions and audit history; invoice register linked to client, period and product where applicable | Auditor can create a category/product, find it, link an invoice and view its history | Product codes, units, variants, duplicate rules, inventory ownership and who maintains catalogue |
| Inconsistent customer/company registration and incomplete fields | Guided client/entity onboarding with required/optional fields, validation and completeness indicators | Required fields are enforced; incomplete profile is visibly marked and cannot silently pass a required workflow gate | Legal entity types, branches, TIN validation, registration evidence and regional variations |
| Difficulty encoding historical financial data (purchases, expenses, tax records, revolving VAT) | Historical opening/import workspace with source attachments, period, category, amounts, provenance and review status | Imported/entered history is traceable to source and visibly marked draft/reviewed/approved | Historical period coverage, opening balances, VAT carry-forward terminology and reconciliation policy |
| Inefficient collection/processing of local and import/export invoices | Invoice intake, manual entry, spreadsheet import, attachment, duplicate detection, validation and review queue | User can capture, attach, filter and export invoice records; invalid/incomplete entries are surfaced | Official invoice layouts, import declarations, exchange rate, customs evidence, credit notes and source-system formats |
| No automated VAT reporting | VAT workpaper/schedule generated from approved invoice and historical data, with drill-down to source records | Totals reconcile to included/excluded invoice rows and reviewer can inspect exceptions | Applicable law/rules, tax periods, adjustments, exemptions, rounding, carry-forward and review approval |
| No imported-goods cost breakdown | Import-cost worksheet linked to shipment/import batch and source documents | Cost components and calculation assumptions are visible and reproducible | Original proposal explicitly says an additional brief is needed for input fields and relationships; no complete cost schema is supplied |
| No tool to suggest selling prices from imported costs | Optional management-accounting/pricing analysis, separate from audit opinion and statutory tax output | Any suggested price shows inputs, formula, assumptions and editable margin/markup; user approval required | Cost allocation, duties/taxes treatment, overhead, FX, margin/markup, market assumptions and whether this belongs in first release |

## 3. Initial functional requirements

### ARMS-AUD-001 — Product and category register
- Create, view, update and archive client-scoped product categories and products, subject to role permissions.
- Capture only fields approved by the product owner/accounting lead; do not invent mandatory attributes.
- Preserve history and prevent one client from seeing another client's catalogue.
- Allow product/category additions without requiring a software release.

### ARMS-AUD-002 — Client/company profile
- Capture legal/business name, TIN, sector and additional approved identity/contact information.
- Display required-field completeness and validation feedback.
- Preserve source and verification status for identifiers; do not imply government validation without an authorized interface.

### ARMS-AUD-003 — Historical financial data
- Support controlled entry/import of purchases, expenses, tax history and revolving VAT/carry-forward information as described in the source.
- Attach source records and capture reporting period, preparer, review status and import provenance.
- Keep historical opening/import data distinguishable from live posted accounting entries until reconciled and approved.

### ARMS-AUD-004 — Local sales and import/export invoice register
The source proposal lists the following fields for both local sales invoices and import/export invoices:
- Company/individual name
- TIN
- Quantity
- Amount before VAT
- VAT
- MRC
- Invoice number
- Date
- Year
- Remark

The term **MRC** is not defined in the proposal. Do not assign it a meaning until the auditor/tax specialist clarifies it. The fields are an initial source list, not a final universal schema. Additional fields must be validated against actual invoice types and current law.

Capabilities:
- Manual capture and controlled spreadsheet import.
- Attach original invoice or import/export evidence.
- Duplicate and missing-field warnings.
- Filter by client, transaction type, period, supplier/customer and review state.
- Preserve original values and change history.

### ARMS-AUD-005 — VAT schedules and reports
- Generate draft VAT schedules from eligible invoice and historical records.
- Provide drill-down from each total to source invoice rows and supporting files.
- Identify excluded, duplicated, incomplete and manually adjusted records.
- Require review/approval before marking a schedule final.
- Version the tax calculation rules and record effective date/source.
- Export usable PDF and spreadsheet outputs.
- Do not present a draft schedule as an official tax filing or verified government submission.

### ARMS-AUD-006 — Imported-goods cost worksheet
- Provide a configurable worksheet for imported-goods cost analysis.
- Preserve source documents and each cost input.
- Show calculation formula, currency, FX rate/source/date and assumptions.
- Keep the worksheet separate from statutory tax calculations unless a qualified tax reviewer approves a mapping.
- Do not finalize the input schema until the additional brief requested in the original proposal is obtained from auditors/customs-accounting specialists.

### ARMS-AUD-007 — Optional selling-price analysis
- Treat price suggestions as optional management decision support, not audit assurance, statutory valuation or a guaranteed optimal price.
- Display cost basis, allocation assumptions, selected markup/margin and sensitivity.
- Make every assumption editable and require human review.
- Keep out of the first audit workflow release unless pilot auditors and clients demonstrate a clear need and approve the scope.

### ARMS-AUD-008 — Staff, roles and task follow-up
- Maintain user/employee profiles and role-based permissions.
- Assign tasks to individuals or groups with priority, due date, status and required inputs.
- Show workload, overdue items, blockers and completion history.
- Link tasks to client, engagement, period, invoice batch, workpaper or report.
- Task completion must not itself count as professional approval or sign-off.

### ARMS-AUD-009 — Usability and exports
- Provide guided, plain-language forms and progressive disclosure.
- Show examples, inline validation, saved-draft state and actionable errors.
- Export approved reports/schedules to PDF and spreadsheet formats.
- Preserve report period, template/rule version, preparer, reviewer and approval state.

## 4. Recommended first demonstrable workflow

Build one end-to-end workflow before expanding into all modules:

1. Create/select a client profile and verify required fields are complete.
2. Create a reporting period or engagement workspace.
3. Add/import a small batch of local sales or purchase invoices and attach originals.
4. Review duplicates, missing fields and exceptions.
5. Generate a draft VAT schedule with drill-down to source invoices.
6. Have a reviewer record adjustments and approval.
7. Export the schedule and a supporting invoice register.
8. Reopen the client/period and verify the history, evidence and report are preserved offline and after synchronization.

The exact first invoice type and tax treatment should be selected with the pilot auditor/tax specialist. Use synthetic or appropriately de-identified data until the relevant production-data gates are satisfied.

## 5. Progressive discovery rules for auditor-led development

Auditors should be asked to demonstrate work using real-world examples (synthetic or de-identified), not to design database tables or write technical specifications.

For each demonstration, the product team records:
- Trigger: what starts the task?
- Actor: who performs it and who reviews it?
- Inputs: which documents, fields and prior records are needed?
- Steps: what does the auditor actually do?
- Decision: what judgment or rule is applied?
- Output: what document, calculation or status is produced?
- Exception: what happens when information is missing, inconsistent or late?
- Evidence: what must be retained to support the result?
- Approval: who may review, approve, post, sign or release it?
- Offline behavior: what must remain possible without connectivity?

Demonstrate the workflow back to the auditor. Capture corrections as requirements and examples. Never interpret silence or inability to explain software terminology as approval.

## 6. Explicit unresolved questions from the proposal

1. What does “MRC” mean in the invoice field list, and is it required for all invoice classes?
2. Which invoice types and source formats are used in practice for local, import and export transactions?
3. What is the exact meaning and workflow of “revolving VAT” in the auditor's practice?
4. Which historical years and opening balances must be encoded, and how are they reconciled?
5. What cost components and allocation rules are required for imported goods?
6. Is suggested selling price analysis in scope for auditors, client management, or a separate advisory service?
7. Which roles can create, edit, review, approve and export each record?
8. Which reports and layouts are required by clients, firms and authorities?
9. What happens when an invoice is corrected, cancelled, duplicated or received after period close?

These questions can be resolved incrementally during workflow demonstrations. They do not all block the first safe prototype.

## 7. Source traceability

- Page 2: problem statement—manual product/invoice tracking, incomplete company registration, historical records and invoice-processing challenges.
- Page 3: VAT reporting, imported-goods cost breakdown and selling-price suggestion needs; objectives.
- Pages 4–5: product management, company registration (name, TIN, sector), historical purchases/expenses/tax/revolving VAT, local and import/export invoice field lists, VAT reports, import-cost worksheet, users and task management.
- Page 6: user-friendly interface, secure database, role access, possible tax integration and PDF/Excel export; technology options are not adopted as architecture decisions.
- Page 7: iterative methodology and original 13-week estimate; timeline is not accepted as a commitment.
