# Contributing to ARM

## Current state

This repository contains product/architecture specifications and Phase 0 governance scaffolding. It is not yet an application repository. Do not introduce application frameworks, database schemas, migrations, production credentials or real client records until the relevant Phase 0 gates are approved.

## Change process

1. Read the relevant product, domain, security and architecture documents.
2. Open/update a work item with requirement IDs, scope, acceptance criteria, risks and expected test evidence.
3. Work on a focused branch; do not commit directly to main.
4. Preserve existing behavior and invariants. Never weaken controls to make tests pass.
5. Update specifications and ADRs when decisions change.
6. Add normal, negative, authorization, failure and offline tests as applicable.
7. Run scripts/validate_repository.py and all relevant project checks.
8. Submit a PR with risk assessment, evidence, limitations and rollback/compatibility notes.

## Engineering rules

- Use exact decimal arithmetic and explicit rounding for money; binary floating-point is not an accounting source of truth.
- Posted accounting records are immutable and corrected through authorized reversal/replacement.
- Sync commands must be retry-safe and idempotent. Never silently discard conflicting changes.
- Enforce tenant, firm, client, engagement and role boundaries on the server.
- Client-supplied tenant/workspace identifiers are selectors, never proof of authorization.
- Never commit secrets, client records, production exports or real credentials.
- Record dependency purpose, license, maintenance status, security posture and update owner before adoption.
- Do not claim Ethiopian regulatory compliance without current authoritative evidence and professional/legal validation.
- Automation may prepare or suggest; designated professionals remain accountable for approvals and submissions.

## Pull request checklist

- [ ] Requirement and acceptance criteria identified.
- [ ] Existing architecture/code checked before adding an implementation.
- [ ] Security, privacy, tenant isolation and retention impact considered.
- [ ] Offline, retry, duplicate delivery and conflict behavior considered.
- [ ] Accounting, audit and tax invariants preserved.
- [ ] Tests and validation evidence included.
- [ ] Documentation and ADRs updated where needed.
- [ ] No secrets, real client data or sensitive generated artifacts included.
- [ ] Rollback, compatibility and operational impact described.
