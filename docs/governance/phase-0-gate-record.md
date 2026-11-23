# Phase 0 — Exit gate and controlled development authorization

**Phase 0 decision: APPROVED FOR CONTROLLED INITIAL DEVELOPMENT.**  
**Approval basis:** Product-owner direction recorded on 2026-11-23: begin incremental development and refine detailed requirements through auditor demonstrations and use.  
**Important distinction:** This is an authorization to begin scoped Phase 1 development. It is **not** a declaration that every Phase 0 evidence gate is closed, nor approval for real client data, regulated services, live tax filing, production distribution or production deployment.

## 1. What this approval permits

- Build and demonstrate approved foundational workflows using synthetic data.
- Run scoped technical spikes for desktop, local storage, synchronization and usability.
- Work with auditors and accountants to discover detailed fields, exceptions and acceptance examples through demonstrations.
- Update the PRD, workflow specifications, backlog and tests as evidence is gathered.
- Use properly de-identified data for practitioner sessions only under an approved session protocol.

## 2. What remains prohibited pending its own approval

- Real client/taxpayer data in development or pilot environments.
- Live accounting posting or report issuance as an operational professional service.
- Regulated audit services or signing permissions based only on self-declared credentials.
- Live government filing or unverified portal/API automation.
- Production deployment, general availability or desktop distribution.
- Any bypass of tenant isolation, immutable financial history, authorization, privacy, retention or recovery controls.

## 3. Gate register

| Gate | Current assessment | Remaining evidence / conditions | Status |
|---|---|---|---|
| G0-01 Workflows | Stakeholder interviews and auditor pain-point proposal provide a usable starting point. Detail will be discovered incrementally. | Demonstrate selected workflows; maintain glossary, role journeys, exception paths and practitioner acceptance notes. | Approved for iterative discovery; ongoing |
| G0-02 Ethiopian regulation | Primary references identified; counsel/professional interpretation remains incomplete. | Legal applicability matrix, current gazetted texts, licensing scope, regional/city tax coverage and retention review before affected production use. | Open; does not block synthetic-data development |
| G0-03 Data governance | Provisional firm/client responsibility model exists. | Purpose-based role mapping, contracts, cross-border/registration analysis and retention/export terms before real client data. | Open; real-data blocker |
| G0-04 Threat model | Security direction identified. | Threat model and independent review before real data or distribution. | Open |
| G0-05 Local security | Encrypted SQLite and protected Windows key material are candidate directions. | Synthetic-data prototype, key lifecycle/recovery and negative tests. | Open; spike authorized |
| G0-06 Native boundary | Tauri direction selected for validation. | Capability/command allowlist, IPC validation, filesystem isolation and security tests before distribution. | Open; spike authorized |
| G0-07 Sync correctness | Visible conflicts and authorized resolution required. | Outbox/inbox, idempotency, conflict policy and fault-injection tests before trusted multi-device operation. | Open; spike authorized |
| G0-08 Financial correctness | ETB primary and foreign currency required. | Approved accounting policies and independent fixtures before posting or financial statements are relied upon. | Open; design/prototype allowed |
| G0-09 Audit methodology | Workpapers, evidence, review notes and partner sign-off required. | Qualified methodology review, role separation and engagement templates before operational audit use. | Open; workflow prototyping allowed |
| G0-10 Tax governance | Tax specialist matrix reported, but not inspected. | Controlled matrix, source reconciliation and independently reviewed expected results before tax outputs are relied upon. | Open; mock/draft UI only |
| G0-11 Operations | Hosting comparison and firm-defined RPO/RTO required. | Hosting assessment, recovery objectives, backup/restore evidence, monitoring and incident response before production. | Open |
| G0-12 Delivery readiness | Controlled pilot and progressive delivery accepted. | Owners, estimates, supported Windows baseline, dependency policy, CI evidence and release process. | In progress |

## 4. Non-negotiable engineering constraints

- Synthetic data only until the real-data gate is explicitly approved.
- No last-write-wins overwriting of journals, audit evidence, approvals or issued reports.
- Posted accounting entries are corrected through controlled reversal/replacement, not destructive edits.
- Local changes are not represented as cloud-received until acknowledged by the server.
- Government submission is not represented as successful until an official acknowledgement is recorded.
- Automation assists professional work; required human review and authorization remain mandatory.
- Product-owner OS compatibility decision (2026-11-23): target Windows 10 x64 and Windows 11 x64 broadly, including ordinary Windows 10 installations that no longer receive Microsoft security updates. This is an application compatibility requirement, not a representation that unsupported Windows installations are secure or vendor-supported. The application must not disable or bypass OS security controls. Maintain a tested compatibility matrix by Windows edition/build and document known limitations. Production use with real client data on an unsupported OS remains a security-risk exception and is blocked until the security reviewer approves a documented risk assessment and compensating controls. Installer acceptance requires explicit warnings and verified behavior on each declared target.
- No production use until backup restoration, incident response, monitoring, access control and agreed RPO/RTO are demonstrated.

## 5. Approval record

| Role | Decision | Date | Notes |
|---|---|---|---|
| Product owner | Approved for controlled initial development | 2026-11-23 | Based on the user's explicit direction in the project conversation; not a legal or technical gate waiver |
| Technical lead | Pending | | |
| Security reviewer | Pending | | |
| Accounting reviewer | Pending | | |
| Audit methodology reviewer | Pending | | |
| Tax reviewer | Pending | | |
| Ethiopian legal/privacy reviewer | Pending | | |
| Operations owner | Pending | | |

## 6. Revisit triggers

Revisit this authorization before introducing real client data, enabling posting/signing/filing, distributing desktop builds, changing the data ownership model, changing cryptographic/storage architecture, or entering production. Record the evidence and explicit approval for each transition.
