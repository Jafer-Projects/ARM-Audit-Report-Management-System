# ARM — Audit Report Management System

ARM is a proposed multi-tenant professional platform for independent auditors, accounting professionals and firms serving business clients in Ethiopia.

Its defining product requirement is a **Windows-first, fully offline-capable desktop workspace** connected to a cloud platform for controlled synchronization, collaboration, centralized administration and authorized external integrations. Clients use a responsive web portal; auditors and accountants use the desktop application for substantive professional work.

> **Status:** Phase 0 is approved for controlled initial development using synthetic data. This does not mean all legal, security, tax, accounting or operational gates are closed, and it does not authorize real client data, live filing or production deployment.

## Approved product decisions

- Business model: multi-tenant platform serving individual professionals and firms.
- Users: auditors, accountants, clients, managers and platform administrators.
- Initial market: Ethiopia.
- Initial scale target: capacity hypothesis to be tested, not a production guarantee.
- Desktop OS compatibility target: Windows 10 x64 and Windows 11 x64, including ordinary Windows 10 installations that no longer receive Microsoft security updates, as explicitly requested by the product owner. This is an application compatibility target, not a claim that unsupported Windows versions are secure or vendor-supported. Production use with real client data on an unsupported OS remains a security-risk exception requiring documented risk acceptance and separate security approval.
- Offline: core professional workflows remain usable without internet.
- Devices: individually assigned computers.
- Synchronization: automatic background synchronization plus user-controlled retry/status.
- Data storage: encrypted local storage and centralized cloud storage.
- Audit services: financial, internal, compliance, tax and NGO/donor-funded audits.
- Accounting: professional accountants/bookkeepers maintain client books.
- Standards: applicable AABE-adopted/adapted frameworks, including IFRS, IFRS for SMEs and IPSAS where applicable; ISA as applicable.
- Automation: high, with human professional review and authorization.
- Client functions: document upload, requests, questionnaires, approvals and messages.
- Documents: PDF, spreadsheets, word-processing files, scans, photographs and bank exports; Amharic and English document support.
- Tax scope: VAT, income tax, withholding, PAYE, turnover tax and excise, subject to applicability and validated legislation.
- Government integration: tax/revenue and electronic invoicing workflows, subject to official technical access and authorization.
- Hosting: Linux VPS initially, subject to data-location and operations assessment.
- Licensing: individual professional subscriptions and firm subscriptions.
- Client workspace administration: firm-controlled, subject to contract, confidentiality, applicable law and client rights.
- Accounting operating model: professionals maintain client accounts; clients provide source data and may review/confirm it.

## Architecture direction

| Concern | Selected direction |
|---|---|
| Windows desktop shell | Tauri 2 |
| Desktop native services | Rust |
| Shared application UI | React + TypeScript |
| Backend/API | Node.js + TypeScript |
| Local database | SQLite with an evaluated encryption strategy |
| Cloud relational database | PostgreSQL |
| Document storage | S3-compatible object storage |
| Background work | Dedicated worker process; Redis only where justified |
| Initial architecture | Modular monolith plus workers |
| Deployment | Ubuntu LTS VPS, containerized services, reverse proxy |

These are architecture directions to validate; final versions, licenses, maintenance and security posture must be verified before adoption.

## Product and delivery documents

- [Product requirements baseline](docs/product/software-requirements-specification.md)
- [Auditor pain points and initial workflow requirements (derived from the supplied 9 January 2025 proposal)](docs/product/auditor-pain-points-and-initial-workflow-requirements.md)
- [Roles, permissions and workflows](docs/product/user-roles-and-permissions.md)
- [Ownership and licensing model](docs/product/data-ownership-and-licensing.md)
- [Implementation roadmap and acceptance criteria](docs/delivery/implementation-roadmap-and-acceptance.md)
- [Phase 1 incremental development and practitioner feedback](docs/delivery/phase-1-incremental-development-and-feedback.md)
- [Developer implementation specification for Increments 0–5](docs/delivery/developer-implementation-specification-increments-0-and-1.md)

## Architecture and domain references

- [System architecture](docs/architecture/system-architecture.md)
- [Offline-first and synchronization protocol](docs/architecture/offline-first-and-sync.md)
- [Data and security architecture](docs/architecture/data-and-security-architecture.md)
- [Accounting, audit and tax domain specifications](docs/domains/accounting-audit-tax-domains.md)
- [Ethiopian government integration strategy](docs/domains/ethiopian-regulatory-integrations.md)
- [Deployment and operations](docs/delivery/deployment-and-operations.md)
- [Architecture decisions](docs/governance/architecture-decision-records.md)

## Phase 0 discovery and governance

- [Phase 0 technical discovery plan](docs/discovery/phase-0-technical-discovery-plan.md)
- [Open questions and evidence register](docs/discovery/open-questions-and-evidence-register.md)
- [Technical spike and validation matrix](docs/discovery/technical-spike-and-validation-matrix.md)
- [Phase 0 gate and controlled development authorization](docs/governance/phase-0-gate-record.md)
- [Contribution and engineering rules](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

## Non-negotiable engineering principles

1. No silent loss or last-write-wins overwriting of posted financial records.
2. Posted accounting entries are corrected by controlled reversal/replacement, not destructive edits.
3. Every material professional conclusion is attributable to a person, evidence and review trail.
4. Firm/client/engagement authorization is enforced server-side and tested across tenants.
5. Offline authorization is bounded; revocation cannot be assumed to reach a disconnected device immediately.
6. A local change is not represented as cloud-received until the server acknowledges it.
7. Government submission is not represented as successful until an official acknowledgment is recorded.
8. Tax rules and accounting policies are versioned with effective dates and authoritative references.
9. AI and automation produce reviewable assistance, not autonomous audit opinions or unapproved filings.
10. Backups are considered valid only after successful restoration tests.

## Current delivery state

Phase 0 approval permits controlled initial development and practitioner demonstrations using synthetic data. Real client data, live accounting/tax operations, desktop distribution and production deployment remain subject to the corresponding legal, security, professional and operational approvals. The repository remains a specification and delivery-governance baseline until implementation work is completed and verified.
