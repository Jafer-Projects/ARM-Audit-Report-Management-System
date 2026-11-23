# Client Data Administration and Commercial Licensing

## 1. Approved model

ARM supports both individual professional subscriptions and firm subscriptions. Client workspaces are administered by the firm that establishes the client relationship and engagement.

This product decision means firm administrators control workspace configuration, staff access and engagement operations. It does **not** automatically determine legal ownership of every client source record, personal data item or statutory record. Contracts, professional obligations, applicable Ethiopian law and client rights determine those matters.

## 2. Data classification and stewardship

Separate the following categories:

1. Client-provided source records: invoices, bank statements, payroll inputs, contracts and responses.
2. Client accounting records maintained by the firm: ledgers, journals, reconciliations and financial statements.
3. Firm professional records: audit plans, workpapers, internal review notes, risk assessments and conclusions.
4. Shared deliverables: approved financial statements, tax returns, reports and agreed schedules.
5. Platform operational records: identity, subscription, security and service telemetry.

For each category, document controller/processor roles, permitted purposes, access, retention, export and deletion rules before production onboarding. Do not assume all categories have the same legal status.

## 3. Firm administration

- A firm creates a client workspace and identifies authorized client representatives.
- Firm administrators invite and remove staff.
- Engagement assignments limit professional access.
- Client users see only explicitly released or shared information.
- Internal workpapers and review notes remain restricted unless a defined professional/legal process permits disclosure.
- All grants, removals, exports and releases are audited.

## 4. Client rights and exit procedures

Provide controlled export and transition procedures. A firm closing an engagement or leaving the platform must be able to export relevant records in documented, usable formats, subject to lawful retention, confidentiality and professional obligations.

Deletion must be policy-driven and must account for statutory retention, legal holds, audit trail integrity, backups and contractual commitments. A user-facing “delete client” action must not destructively erase records that must be retained.

## 5. Individual professional subscriptions

- A professional can register and maintain an individual account.
- Personal subscription entitlements do not create firm membership or client access.
- The professional may be invited to one or more firms.
- Work performed for a firm remains associated with that firm's workspace and authorization model.
- When a professional leaves a firm, access is revoked without rewriting historical authorship.

## 6. Firm subscriptions

- Firm subscription includes a configurable number of staff seats and features.
- Firm owner/admin manages seat allocation.
- Seat additions, removals, renewal, suspension and billing events are recorded.
- Subscription status must not silently delete data or disable lawful access/export obligations.
- Grace periods and read-only behavior must be explicitly defined.

## 7. Shared professional accounts

A single human identity may hold an individual subscription and memberships in multiple firms. The UI must make the active organization clear. The API must require an explicit organization context and verify membership on every operation. No cross-firm search or document discovery is allowed by default.

## 8. Commercial billing

Initial scope should cover subscription plans, seat counts, invoices, payment status and entitlement enforcement. Payment provider integrations are selected only after commercial, security and local availability review. Financial accounting for ARM's own subscription revenue is separate from customer accounting workspaces.

## 9. Required legal review before launch

Obtain qualified Ethiopian legal and professional advice on privacy/data protection, professional confidentiality, record retention, cross-border hosting, client engagement terms, electronic signatures, tax filing authority and professional independence. Publish accurate privacy notices, terms, data-processing terms and client authorization language before onboarding real client data.
