# ARM — Audit Report Management System

ARM is a proposed multi-tenant professional platform for independent auditors, accounting professionals and firms serving business clients in Ethiopia.

Its defining product requirement is a **Windows-first, fully offline-capable desktop workspace** connected to a cloud platform for controlled synchronization, collaboration, centralized administration and authorized external integrations. Clients use a responsive web portal; auditors and accountants use the desktop application for substantive professional work.

> **Status:** Architecture and product specification baseline. This repository currently documents the approved direction; it does not claim that the application, government integrations, tax rules or production controls have already been implemented or certified.

## Approved product decisions

- Business model: multi-tenant platform serving individual professionals and firms.
- Users: auditors, accountants, clients, managers and platform administrators.
- Initial market: Ethiopia.
- Initial scale target: 100–500 registered users, subject to capacity testing.
- Desktop: Windows 10/11, with a Windows-first release.
- Offline: core professional workflows remain usable without internet.
- Devices: individually assigned computers.
- Synchronization: automatic background synchronization plus user-controlled retry/status.
- Data storage: encrypted local storage and centralized cloud storage.
- Audit services: financial, internal, compliance, tax and NGO/donor-funded audits.
- Accounting: professional accountants/bookkeepers maintain client books.
- Standards: IFRS/IFRS for SMEs where applicable, ISA and applicable Ethiopian requirements.
- Automation: high, with human professional review and authorization.
- Client functions: document upload, requests, questionnaires, approvals and messages.
- Documents: PDF, spreadsheets, word-processing files, scans, photographs and bank exports.
- Tax scope: VAT, income tax, withholding, PAYE, turnover tax and excise, subject to applicability and validated legislation.
- Government integration: tax/revenue and electronic invoicing workflows, subject to official technical access and authorization.
- Hosting: Linux VPS initially.
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

These are architectural selections, not a claim that every dependency has already been implemented. Final versions, support windows, licensing, security posture and compatibility must be verified before adoption.

## Specification index

- [Product requirements and scope](docs/product/software-requirements-specification.md)
- [Roles, permissions and workflows](docs/product/user-roles-and-permissions.md)
- [Ownership and licensing model](docs/product/data-ownership-and-licensing.md)
- [System architecture](docs/architecture/system-architecture.md)
- [Offline-first and synchronization protocol](docs/architecture/offline-first-and-sync.md)
- [Data and security architecture](docs/architecture/data-and-security-architecture.md)
- [Accounting, audit and tax domain specifications](docs/domains/accounting-audit-tax-domains.md)
- [Ethiopian government integration strategy](docs/domains/ethiopian-regulatory-integrations.md)
- [Implementation roadmap and acceptance criteria](docs/delivery/implementation-roadmap-and-acceptance.md)
- [Deployment and operations](docs/delivery/deployment-and-operations.md)
- [Architecture decisions](docs/governance/architecture-decision-records.md)

## Phase 0 discovery and governance

- [Phase 0 technical discovery plan](docs/discovery/phase-0-technical-discovery-plan.md)
- [Open questions and evidence register](docs/discovery/open-questions-and-evidence-register.md)
- [Technical spike and validation matrix](docs/discovery/technical-spike-and-validation-matrix.md)
- [Phase 0 exit gates and approval record](docs/governance/phase-0-gate-record.md)
- [Contribution and engineering rules](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

Phase 0 is in progress. Repository scaffolding is not equivalent to completed discovery or approval; Phase 1 remains gated by the evidence and approvals recorded above.

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

This commit establishes documentation only. There is no production-ready desktop client, accounting engine, cloud service, tax engine, government connector or deployment environment implied by these documents. Implementation must proceed through the acceptance gates in the roadmap.
