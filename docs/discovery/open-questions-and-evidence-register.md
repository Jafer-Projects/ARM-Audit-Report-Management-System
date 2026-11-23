# Phase 0 — Open questions and evidence register

**Status:** Initial register. Items remain open until authoritative evidence and accountable approval are recorded.

| ID | Priority | Question / evidence required | Required reviewer | Status |
|---|---|---|---|---|
| OQ-001 | Blocker | Which Ethiopian laws/directives govern personal data, client financial records, cross-border processing, breach response and data-subject rights? | Ethiopian legal/privacy counsel | Open |
| OQ-002 | Blocker | What professional licensing, audit practice, independence, workpaper retention and report issuance obligations apply? | Audit practitioner + counsel | Open |
| OQ-003 | Blocker | Which IFRS/IFRS for SMEs, ISA, local standards and regulator requirements apply by client segment/reporting period? | Audit/accounting leads | Open |
| OQ-004 | Blocker | What current tax categories, rates, thresholds, filing periods, exemptions, forms, amendments and effective dates apply? | Tax specialist + counsel | Open |
| OQ-005 | Blocker | Which official e-invoice/tax interfaces exist, who may access them, and what authorization, sandbox and certification rules apply? | Authorized integration contact + counsel | Open |
| OQ-006 | Blocker | For each data class, who is legal controller/owner, custodian, processor and authorized recipient? What applies on transfer, termination, dispute or legal hold? | Legal + firm/client representatives | Open |
| OQ-007 | Blocker | What local encryption, Windows key protection, recovery, rotation and lost-device design is acceptable? | Security + Windows engineering | Open |
| OQ-008 | Blocker | Can SQLite encryption, Tauri capabilities and document vault meet the endpoint threat model without exposing keys to web content/plugins? | Independent security reviewer | Open |
| OQ-009 | Blocker | What sync invariants apply to journals, workpapers, approvals, returns, uploads and metadata? Which conflicts require human resolution? | Domain leads + sync architect | Open |
| OQ-010 | Blocker | What offline lease, renewal, revocation, emergency access and device-loss policy balances confidentiality and prolonged outages? | Security + product + firms | Open |
| OQ-011 | High | Which Windows versions, architectures, endpoint controls, antivirus and user permission profiles must be supported? | Desktop lead + pilot firms | Open |
| OQ-012 | High | What document sizes, monthly volumes, scan quality, OCR languages (including Amharic/English) and network conditions are representative? | Client workflow + operations | Open |
| OQ-013 | High | What are initial concurrent users, firm count, engagements, document volume and synchronization burst targets? | Product + operations | Open |
| OQ-014 | High | Which VPS/object-storage locations satisfy latency, residency, availability, backup isolation and budget? | Operations + legal | Open |
| OQ-015 | High | What RPO, RTO, retention and restore-test frequency are acceptable? | Firm owners + operations | Open |
| OQ-016 | High | What currency, decimal precision, rounding, FX sources, fiscal periods and chart-of-account variants are required? | Accounting lead | Open |
| OQ-017 | High | Which audit methodologies, templates, sampling, sign-offs and report variants are required? | Audit methodology lead | Open |
| OQ-018 | High | What contract terms govern source records, workpapers, generated outputs, confidentiality, export and post-termination access? | Legal + pilot firms | Open |
| OQ-019 | High | Which credentials require verification, by which authority, and how are expiry/suspension handled? | Professional bodies + legal | Open |
| OQ-020 | Medium | Which languages are needed in desktop, portal, reports and OCR at pilot and later stages? | Product + pilot users | Open |
| OQ-021 | Medium | Which dependency licenses, support arrangements and maintenance commitments are acceptable? | Technical lead + legal | Open |
| OQ-022 | Medium | Which subscription/payment providers and tax invoice obligations apply to individual and firm licensing? | Product + finance + counsel | Open |
| OQ-023 | Medium | What support access, service hours, incident notification and escalation expectations apply? | Operations + firms | Open |
| OQ-024 | Medium | Which client/firm exports are mandatory, and how will completeness of records, attachments and audit history be proven? | Product + legal + pilot firms | Open |

## Evidence quality rules

- Prefer current primary sources: legislation, official regulator publications, professional standards, official technical documentation and signed stakeholder decisions.
- Record title, issuing body, URL/controlled reference, publication/effective date, access date and relevant section.
- Secondary commentary supports discovery but cannot alone validate a legal or tax rule.
- Record jurisdiction, client/taxpayer segment and reporting period; rules may not apply universally.
- If a source is ambiguous or unavailable, preserve uncertainty and block dependent functionality.
- Never put confidential client evidence, credentials or personal data in this repository. Store evidence in an approved private repository and link only to its controlled reference.
