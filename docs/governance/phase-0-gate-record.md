# Phase 0 — Exit gate and approval record

**Overall status: NOT APPROVED.** Repository scaffolding is present; discovery, technical evidence and independent approvals remain outstanding.

Phase 1 application implementation must not start until all mandatory gates are approved or formally re-scoped by the product owner with documented risk acceptance. Security, legal, accounting-integrity and data-governance blockers cannot be waived by a developer.

| Gate | Mandatory evidence | Status | Reviewer / approval | Evidence |
|---|---|---|---|---|
| G0-01 Product workflows | Signed role journeys, process maps, glossary and prioritized requirements. | Not started | Product owner + practitioner representatives | |
| G0-02 Ethiopian regulation | Current authoritative sources for professional, accounting, audit, tax, privacy, retention and filing requirements; legal/professional review. | Not started | Ethiopian counsel + domain leads | |
| G0-03 Data governance | Classification, responsibility matrix, contract model, access/export/retention/termination policy. | Not started | Legal/privacy owner + pilot firm | |
| G0-04 Threat model | Threat model, abuse cases, controls, residual risks and independent review. | Not started | Security reviewer | |
| G0-05 Local security | SQLite encryption and Windows key protection prototype, lifecycle, recovery and negative tests. | Not started | Desktop/security reviewer | |
| G0-06 Native boundary | Tauri capability/command allowlist and filesystem isolation proven by negative tests. | Not started | Security reviewer | |
| G0-07 Sync correctness | Versioned protocol, entity conflict policy, idempotency and crash/retry fault-injection evidence. | Not started | Sync lead + domain leads | |
| G0-08 Financial correctness | Accounting model and independently verified decimal, journal, period and reversal fixtures. | Not started | Qualified accounting reviewer | |
| G0-09 Audit methodology | Audit lifecycle and evidence/review/report workflow approved by qualified reviewer. | Not started | Audit partner/methodology reviewer | |
| G0-10 Tax governance | Effective-dated tax rule lifecycle and authoritative source/approval process validated. | Not started | Tax specialist + counsel | |
| G0-11 Operations | Hosting, RPO/RTO, backup/restore, incident response and support plan. | Not started | Operations owner | |
| G0-12 Delivery readiness | Named owners, skills/capacity, estimates, dependency policy, CI and Phase 1 backlog. | Not started | Product owner + technical lead | |

## Approval record

Every gate must contain an evidence reference/date, named accountable reviewer, explicit pass/conditional pass/fail, conditions and risk owner where applicable, and product owner approval of the complete gate set.

| Role | Name | Decision | Date | Evidence / notes |
|---|---|---|---|---|
| Product owner | To assign | Pending | | |
| Technical lead | To assign | Pending | | |
| Security reviewer | To assign | Pending | | |
| Accounting reviewer | To assign | Pending | | |
| Audit methodology reviewer | To assign | Pending | | |
| Tax reviewer | To assign | Pending | | |
| Ethiopian legal/privacy reviewer | To assign | Pending | | |
| Operations owner | To assign | Pending | | |

## Phase 1 entry constraints

- Use synthetic data until privacy, security, retention and contractual controls are approved for pilot data.
- No live government filing until official access, authorization, certification and sandbox evidence are complete.
- No production accounting posting until financial invariants and independent expected-result tests pass.
- No desktop distribution until signing, update integrity, rollback and local-vault recovery are validated.
- No production deployment until backup restoration, incident response, monitoring and access controls are demonstrated.
