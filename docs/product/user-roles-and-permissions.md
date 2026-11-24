# ARM User Roles, Permissions and Workflows

## 1. Authorization model

Use organization-scoped role-based access control (RBAC), supplemented by engagement assignments, client workspace membership, record sensitivity and explicit grants.

A user's role in one firm must not grant access in another firm. Authorization is evaluated on every server request and independently enforced by local offline policy snapshots. Client-supplied role names or tenant identifiers are never trusted.

## 2. Roles

### Platform operations administrator
Manages platform availability, tenant lifecycle, subscriptions and operational configuration. Does not receive blanket content access. Break-glass support access must require a documented reason, explicit authorization, short duration and immutable audit record.

### Firm owner / firm administrator
Manages firm profile, staff invitations, seats, client workspaces, role assignments, firm policies and billing. Cannot bypass engagement-level professional review controls.

### Engagement partner
Accepts and oversees engagements, approves scope and key judgments, reviews completion and authorizes final reports according to firm policy.

### Engagement manager / reviewer
Assigns work, reviews workpapers, raises review notes, monitors milestones and recommends completion. Cannot approve matters reserved to the engagement partner.

### Auditor
Works on assigned engagements, records risks and procedures, collects and evaluates evidence, documents findings and prepares report drafts. Cannot approve their own restricted work where segregation of duties is required.

### Accountant / bookkeeper
Maintains assigned client accounting records, posts journals within granted authority, reconciles accounts, prepares financial statements and tax schedules. Cannot issue audit opinions merely by holding an accounting role.

### Tax preparer / reviewer
Prepares tax calculations and returns, reconciles supporting records and routes filings for review and authorization. Submission permissions are separate from preparation permissions.

### Client owner / authorized representative
Views permitted client workspace information, uploads records, responds to requests, confirms selected information and views released deliverables. Cannot edit firm workpapers, audit findings or accounting postings unless separately authorized under a documented workflow.

### Client contributor
Uploads documents and answers assigned requests within a restricted scope. Cannot view unrelated client records, firm notes or other users' submissions.

## 3. Permission dimensions

Every permission decision must consider:
- User identity and account status.
- Active organization membership.
- Organization and subscription status.
- Role and specific permission.
- Client workspace membership.
- Engagement assignment.
- Record classification and sensitivity.
- Workflow state.
- Device authorization and offline lease.
- Required separation of duties.

## 4. Data access boundaries

### Firm-controlled client workspace
The firm administers the workspace and controls staff access. Client representatives receive explicit, revocable grants. Contractual terms, applicable privacy law and client rights govern collection, processing, retention, export and disclosure. “Firm-controlled” is an application administration decision, not a blanket assertion that the firm owns every underlying client record or may disregard legal rights.

### Engagement workpapers
Firm-created workpapers, internal review notes, risk assessments and professional conclusions are restricted to authorized firm personnel. Client visibility is explicit and selective.

### Client-supplied documents
Preserve submitter identity, submission time, source, version and receipt status. Do not silently replace earlier versions.

### Accounting books
Professional accounting staff maintain books. Client users can submit source records and review/confirm selected information. Posted transactions require authorized correction workflows.

### Audit access
Auditors receive scoped access to relevant accounting records and source documents. Audit workpapers and conclusions remain separately permissioned. Where the auditor belongs to a different firm, access requires an explicit engagement authorization and conflict/independence controls.

## 5. Segregation of duties

At minimum, support policy controls for:
- Journal preparation versus posting approval.
- Tax preparation versus tax review and filing authorization.
- Audit procedure execution versus review.
- Engagement completion versus final report authorization.
- Subscription/payment administration versus refund approval where applicable.
- Platform support access versus approval of access.

A firm may configure thresholds and approval chains, but cannot disable mandatory platform safety invariants.

## 6. Offline permissions

- Device receives only assigned workspace data and a signed, time-bounded authorization policy.
- The desktop records the policy version and expiry.
- Sensitive actions that require current server validation remain pending while offline.
- Offline lease duration is configurable within a safe platform maximum.
- On expiry, the app restricts access to protected data and queues no new privileged actions.
- A revocation made while a device is disconnected cannot take effect on that device until reconnection or lease expiry; this limitation must be disclosed and mitigated with encryption, device controls and short leases.
- Device revocation invalidates future cloud access and triggers local key/data handling on next connection.

## 7. Subscription entitlements

Individual subscription:
- Enables the named professional's personal account features and permitted workspaces.
- Does not grant access to any firm/client data without membership and assignment.

Firm subscription:
- Enables firm workspace, purchased seats and firm administration.
- Firm controls invitations and seat assignment.
- Seat removal does not erase historical authorship or audit trails.
- Account membership and subscription entitlements are separate from data access permissions.

## 8. Auditability

Log access grants, role changes, invitations, exports, report releases, journal postings/reversals, tax approvals, filing attempts, device registration/revocation and break-glass access. Logs must include actor, tenant, target, action, timestamp, correlation ID and relevant before/after references without unnecessarily copying sensitive document contents.


## 8. Cloud RBAC implementation

The cloud service now supports a tenant permission catalogue, custom role creation/renaming/deletion, permission replacement, member role assignment/removal, member listing, invitation creation/acceptance/revocation and membership activation/suspension/revocation. Invitation acceptance requires an active verified account whose email matches the invitation. Owner/system roles are protected from ordinary deletion and membership-status changes.

Runtime authorization checks both tenant membership and permission on each service operation. PostgreSQL RLS and non-superuser database roles provide additional isolation. The offline desktop still needs signed device/permission leases and equivalent local policy enforcement before disconnected role changes or immediate revocation can be treated as authoritative.
