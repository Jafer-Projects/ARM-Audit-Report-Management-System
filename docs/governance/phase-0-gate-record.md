# Phase 0 — Exit gate and approval record

**Overall status: NOT APPROVED.** Stakeholder interview findings have been incorporated into the open-questions/evidence register. They provide product direction but do not constitute legal opinions, technical validation, or independent gate approval.

Phase 1 application implementation must not start until all mandatory gates are approved or formally re-scoped by the product owner with documented risk acceptance. Security, legal, accounting-integrity and data-governance blockers cannot be waived by a developer.

**Review date:** 2026-11-23  
**Stakeholder findings register:** `docs/discovery/open-questions-and-evidence-register.md`

| Gate | Current assessment | Mandatory remaining evidence | Status | Reviewer / approval | Evidence |
|---|---|---|---|---|---|
| G0-01 Product workflows | Interview direction now exists for roles, workpapers, sign-off, export, language and support. | Signed end-to-end journeys; role/engagement variants; exception and approval paths; pilot acceptance criteria. | In progress — not approved | Product owner + practitioner representatives | |
| G0-02 Ethiopian regulation | Primary references identified for privacy, AABE licensing/standards, tax and record retention. Windows lifecycle conflict identified. | Counsel-reviewed legal applicability matrix; current gazetted texts and amendments; professional licence/scope analysis; regional/city tax coverage; retention and cross-border opinion. | Blocked — not approved | Ethiopian counsel + domain leads | |
| G0-03 Data governance | Provisional view that firm and client may have separate responsibilities by purpose. | Data inventory/flow map; controller/processor determination by purpose; registration/transfer analysis; approved contracts; retention, legal hold, export and termination rules. | Blocked — not approved | Legal/privacy owner + pilot firm | |
| G0-04 Threat model | Encrypted local store and bounded offline revalidation are preferred directions. | Threat model, abuse cases, attacker capabilities, assets/trust boundaries, controls, residual risks and independent review. | Not evidenced | Security reviewer | |
| G0-05 Local security | SQLite encryption plus Windows DPAPI/CNG-protected key material is a candidate direction only. | Synthetic-data prototype; library evaluation; key lifecycle/recovery; WAL/temp/backup inspection; lost-device, privilege and tamper negative tests. | Not evidenced | Desktop/security reviewer | |
| G0-06 Native boundary | Tauri prototype is permitted only as a scoped discovery activity. | Capability/command allowlist; IPC validation; filesystem isolation; webview boundary and plugin tests; independent security review. | Not evidenced | Security reviewer | |
| G0-07 Sync correctness | All conflicts must be visible; authorized resolution required for material conflicts. | Versioned protocol; outbox/inbox; idempotency; entity-specific merge policy; immutable financial/evidence history; crash/retry/race fault-injection results. | In progress — not approved | Sync lead + domain leads | |
| G0-08 Financial correctness | ETB primary with foreign currency transactions. | Functional/transaction currency policy; exact decimal and rounding model; FX source/date; period, journal, reversal and revaluation fixtures independently verified. | In progress — not approved | Qualified accounting reviewer | |
| G0-09 Audit methodology | Standard workpapers, evidence links, review notes and partner sign-off required. | Approved methodology aligned to applicable AABE-adopted/adapted standards; engagement variants; independence, sampling, materiality, sign-off and report-release tests. | In progress — not approved | Audit partner/methodology reviewer | |
| G0-10 Tax governance | Tax specialist matrix reported as verified; manual preparation/export first. | Inspect and version matrix; reconcile every rule to current official federal/regional/city sources; effective-date and transition tests; independent expected-result fixtures; approval/change process. | Blocked — not approved | Tax specialist + counsel | |
| G0-11 Operations | Hosting comparison and firm-defined RPO/RTO required before production. | Hosting/data-location assessment; signed RPO/RTO; backup/restore drills; incident response; monitoring; support access and escalation policy. | In progress — not approved | Operations owner | |
| G0-12 Delivery readiness | Controlled pilot before growth; commercial components may be considered. | Named owners/capacity; Windows 11 production baseline or approved ESU exception; pilot/load targets; dependency/vendor policy; CI and Phase 1 backlog. | In progress — not approved | Product owner + technical lead | |

## Mandatory safety decisions carried forward

- Synthetic data only until privacy, security, retention and contractual controls are approved for pilot data.
- No live government filing until official access, authorization, technical documentation, sandbox and certification evidence are complete.
- No production accounting posting until financial invariants and independent expected-result tests pass.
- No desktop distribution until supported Windows editions, code signing, update integrity, rollback and local-vault recovery are validated.
- Ordinary Windows 10 installations are not an approved production target: Microsoft ended standard Windows 10 support on 2025-10-14. Any exception must identify the exact edition/version, verified active ESU/support coverage, risk owner and sunset date.
- No production deployment until backup restoration, incident response, monitoring, access controls and RPO/RTO are demonstrated.
- No gate is approved by stakeholder selection alone.

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
