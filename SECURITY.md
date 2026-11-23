# Security policy

## Scope and current status

ARM is at specification and Phase 0 foundation stage. This repository does not represent that a production application or production environment is available.

## Reporting a vulnerability

Do not publish credentials, client records, exploitable details or unredacted findings in public issues or PRs. Use GitHub private vulnerability reporting for this repository if enabled. Otherwise contact the repository owner through a private GitHub channel and request a secure disclosure route.

Include affected commit/component, impact, reproduction steps, prerequisites and safe mitigation. Do not access, modify or retain data that is not yours while validating a report.

## Security expectations

- Never commit credentials, signing keys, private certificates, production database dumps or client documents.
- Suspected secret exposure requires immediate rotation/revocation; deleting a later copy is insufficient.
- Security-sensitive changes require threat analysis, negative tests and authorized review.
- Support access to client content must be exceptional, scoped, approved, time-limited and audited.
- Offline revocation limitations and local data protection must be explicit to users and operators.
