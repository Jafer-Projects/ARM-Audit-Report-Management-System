# Phase 0 — Technical discovery and controlled development plan

**Status:** Approved for controlled initial development on 2026-11-23. Discovery and evidence validation continue alongside implementation. See `docs/governance/phase-0-gate-record.md`.

## 1. Purpose

Use the initial product direction and stakeholder interviews as a sufficient starting baseline for small, safe, demonstrable increments. Do not require auditors to produce exhaustive software specifications before development. The product team translates observed professional work into workflows, fields, validation, roles, states and acceptance tests, then demonstrates the result back to practitioners.

Phase 0 approval authorizes synthetic-data development and scoped technical spikes. It does not approve real client data, regulated operations, production deployment or closure of the legal/security/accounting/tax/operations evidence gates.

## 2. Workstreams and evidence

| ID | Workstream | Activities | Evidence to develop progressively | Accountable role |
|---|---|---|---|---|
| P0-01 | Professional workflows | Observe auditors, partners, accountants, tax reviewers and clients completing tasks; map lifecycle and exceptions. | Practitioner demo notes, role journeys, process maps, glossary, acceptance examples. | Product/domain lead |
| P0-02 | Ethiopian regulation | Verify standards, tax obligations, filing calendars, retention, privacy, licensing, e-invoice and portal processes. | Source register with issuer, effective date, applicability and qualified review. | Ethiopian tax/audit specialists + counsel |
| P0-03 | Data governance | Define responsibilities per data class; client access, correction, export, retention, legal hold, termination and portability. | Data-flow map, classification, responsibility matrix, contract clauses and legal review. | Legal/privacy owner |
| P0-04 | Security threat model | Model tenant isolation, staff misuse, compromised/stolen devices, key extraction, malicious files, support access, backup compromise and sync replay. | Threat model, trust boundaries, abuse cases, controls and residual-risk register. | Security lead |
| P0-05 | Offline storage/key protection | Prototype SQLite encryption, Windows DPAPI/CNG key wrapping, backup/restore, device migration and corruption recovery. | Reproducible benchmark, license/advisory review, key lifecycle and recovery demonstration. | Desktop/security engineers |
| P0-06 | Tauri native boundary | Validate command allowlist, capability model, filesystem sandboxing, update behavior and signed build chain. | Attack-surface inventory, permission matrix, signed test build and negative tests. | Desktop lead |
| P0-07 | Sync correctness | Prototype local transaction/outbox, server inbox/deduplication, ordering, idempotency, conflicts, resumable transfer and protocol evolution. | Fault-injection report, invariant tests, diagrams and entity conflict policy. | Backend/sync lead |
| P0-08 | Accounting correctness | Validate accounts, journals, periods, currencies, precision, rounding, imports, corrections and statements. | Domain model, approved examples, independent expected-result fixtures and accounting review. | Accounting lead |
| P0-09 | Audit methodology | Validate acceptance, independence, materiality, risk, procedures, evidence, sampling, review and report issuance. | Methodology mapping, evidence graph, segregation matrix and partner review. | Audit methodology lead |
| P0-10 | Tax rule governance | Establish versioned rule lifecycle, effective dates, source hierarchy, approvals and amendment/history behavior. | Tax rule register, test pack and review process. Never guess rates/thresholds. | Tax lead + counsel |
| P0-11 | Hosting/operations | Compare VPS/network/storage options; define environments, secrets, observability, backup isolation, RPO/RTO and support. | Threat model, cost model, restore test design and runbooks. | Operations lead |
| P0-12 | Delivery readiness | Confirm skills, maintainer roles, release approvals, dependency review, incident handling and pilot support. | Responsibility matrix, estimates, support process and pilot checklist. | Product owner |

## 3. Discovery method for practitioners

Auditors are not expected to describe technical systems. Ask them to demonstrate their work using synthetic or properly de-identified examples. For each observed task, record:
- Trigger and desired outcome.
- Actor and reviewer.
- Documents/inputs and where they originate.
- Actual steps, decisions and calculations.
- Exceptions, corrections and late/missing information.
- Output and evidence that must be retained.
- Approval/sign-off responsibility.
- What must work offline.

The product team creates the software requirement and acceptance tests, demonstrates the workflow back to the auditor, and records corrections. Silence or inability to explain technical terminology is not approval.

## 4. Minimum artefacts for the first increments

Do not create large documents for every possible feature upfront. For each increment maintain:
1. A short workflow/use-case description.
2. Field dictionary for fields actually being built.
3. Role/action and state-transition matrix.
4. Acceptance examples, negative cases and test evidence.
5. Demonstration notes, practitioner feedback and decisions.
6. Known limitations, assumptions and deferred questions.

The product requirements document and auditor pain-point annex provide the broader baseline. The Phase 1 incremental plan defines the initial order.

## 5. Safety-critical evidence that remains mandatory

The following evidence may be gathered while synthetic-data development proceeds, but must be approved before the relevant higher-risk use:
- Privacy/legal roles, contracts, registration/cross-border analysis and retention before real client data.
- Threat model, local encryption/key recovery and Tauri boundary review before distribution or sensitive data.
- Sync invariants and fault-injection before relying on multi-device synchronization for authoritative records.
- Accounting policies and independent fixtures before live posting or relying on statements.
- Audit methodology and professional authorization before operational report signing.
- Tax matrix source reconciliation and independent expected results before relying on tax calculations or filing.
- RPO/RTO, backup restore, incident response and hosting approval before production.

## 6. Decision method

For each decision record options, evidence, assumptions, security/operational consequences, cost, reversibility, owner, approval date and revisit trigger. Update the relevant ADR. A prototype compiling is not validation: include negative/failure cases and reproducible evidence. Use synthetic data only.

## 7. Revisit criteria

The product owner may authorize further increments as they are scoped and acceptance criteria are documented. Revisit the authorization before real client data, live posting/signing/filing, desktop distribution, production deployment, material changes to data ownership, or changes to cryptographic/storage architecture. Record explicit approval and evidence for each transition.
