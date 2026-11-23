# Phase 0 — Technical discovery and foundation plan

**Status:** In progress. Repository scaffolding is being established; discovery evidence and prototype gates remain open.

## 1. Purpose

Convert the approved product direction into evidence-backed decisions before building the desktop application or accepting real client data. Phase 0 validates professional workflows, Ethiopian requirements, offline durability, local security, synchronization, data ownership and delivery readiness.

The current technology direction (Tauri 2, Rust, React/TypeScript, Node.js/TypeScript, SQLite and PostgreSQL) is a candidate baseline to validate. It is not permission to select arbitrary versions/plugins or process production data.

## 2. Workstreams and required evidence

| ID | Workstream | Activities | Required evidence | Accountable role |
|---|---|---|---|---|
| P0-01 | Professional workflows | Interview auditors, partners, accountants, tax reviewers and clients; map engagement lifecycle and exceptions. | Consented interview notes, role journeys, process maps, exception cases, approved glossary. | Product/domain lead |
| P0-02 | Ethiopian regulation | Verify standards, tax obligations, filing calendars, retention, privacy, licensing, e-invoice and portal processes using authoritative sources. | Source register with issuer, effective date, applicability and legal/professional review. | Ethiopian tax/audit specialists + counsel |
| P0-03 | Data governance | Define responsibility per data class; client access, correction, export, retention, legal hold, termination and portability. | Data-flow map, classification, responsibility matrix, contract clauses and legal review. | Legal/privacy owner |
| P0-04 | Security threat model | Model tenant isolation, staff misuse, compromised/stolen devices, key extraction, malicious files, support access, backup compromise and sync replay. | Threat model, trust boundaries, abuse cases, controls and residual-risk register. | Security lead |
| P0-05 | Offline storage/key protection | Prototype SQLite encryption, Windows DPAPI/CNG key wrapping, backup/restore, device migration and corruption recovery. | Reproducible benchmark, license/advisory review, key lifecycle and recovery demonstration. | Desktop/security engineers |
| P0-06 | Tauri native boundary | Prototype command allowlist, capability model, filesystem sandboxing, update behavior and signed build chain. | Attack-surface inventory, permission matrix, signed test build and negative tests. | Desktop lead |
| P0-07 | Sync correctness | Prototype local transaction/outbox, server inbox/deduplication, ordering, idempotency, conflicts, resumable transfer and protocol evolution. | Fault-injection report, invariant tests, diagrams and entity conflict policy. | Backend/sync lead |
| P0-08 | Accounting correctness | Validate accounts, journals, periods, currencies, precision, rounding, imports, corrections and statements. | Domain model, approved examples, independent expected-result fixtures and accounting review. | Accounting lead |
| P0-09 | Audit methodology | Validate acceptance, independence, materiality, risk, procedures, evidence, sampling, review and report issuance. | Methodology mapping, evidence graph, segregation matrix and partner review. | Audit methodology lead |
| P0-10 | Tax rule governance | Establish versioned rule lifecycle, effective dates, source hierarchy, approvals and amendment/history behavior. | Tax rule register, test pack and review process. Never guess rates/thresholds. | Tax lead + counsel |
| P0-11 | Hosting/operations | Compare VPS/network/storage options; define environments, secrets, observability, backup isolation, RPO/RTO and support. | Threat model, cost model, restore test design and runbooks. | Operations lead |
| P0-12 | Delivery readiness | Confirm skills, maintainer roles, release approvals, dependency review, incident handling and pilot support. | Responsibility matrix, estimates, support process and pilot checklist. | Product owner |

## 3. Discovery participants

Minimum discovery targets: two audit practitioners/partners, two accountants/bookkeepers, two tax practitioners, two client business representatives, one Ethiopian legal/regulatory adviser, and desktop/backend/security engineering reviewers. These are targets, not completed interviews. Record role and consent; avoid unnecessary personal or client-confidential data.

## 4. Required artefacts before Phase 1

1. Approved domain glossary and role-based process maps.
2. Prioritized requirements with measurable acceptance criteria.
3. Data classification, data-flow map and firm/client responsibility matrix.
4. Threat model and local device/key lifecycle.
5. Approved offline/sync protocol and fault-injection results.
6. Accounting, audit and tax models reviewed by qualified practitioners.
7. Ethiopian regulatory source register with unresolved questions explicit.
8. Technology report covering versions, licenses, maintenance, vulnerabilities and Windows compatibility.
9. Costed operations plan with RPO/RTO and recovery design.
10. Phase 1 backlog, named owners, estimates and release gates.

## 5. Decision method

For each decision record options, evidence, assumptions, security/operational consequences, cost, reversibility, owner, approval date and revisit trigger. Update the relevant ADR. A prototype compiling is not validation: include negative/failure cases and reproducible evidence. Use synthetic data only.

## 6. Exit criteria

Phase 0 is complete only when every mandatory item in docs/governance/phase-0-gate-record.md has an evidence link, named reviewer and explicit approval. Unresolved high-severity security, data ownership, financial integrity, regulatory or offline durability issues block Phase 1.

## 7. Out of scope

Production features, real client/taxpayer data, production credentials, shared/production migrations, government portal automation, production VPS provisioning and unverified tax/retention rules.
