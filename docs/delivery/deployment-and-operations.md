# ARM Deployment and Operations

## 1. Initial hosting

Use an Ubuntu LTS VPS for the initial cloud environment. A planning baseline of 4–8 vCPU, 16 GB RAM and 200–400 GB NVMe may be evaluated for a pilot/early deployment, but is not a capacity guarantee. Benchmark API concurrency, PostgreSQL workload, synchronization bursts, OCR and document traffic before production sizing.

## 2. Initial topology

- Reverse proxy and TLS termination.
- Stateless Node.js/TypeScript API.
- Separate worker process.
- PostgreSQL on a private network or separate protected host.
- Redis only if queue/cache requirements justify it.
- S3-compatible object storage outside the VPS where feasible.
- Centralized logs, metrics and alerting.
- Independent encrypted backup destination.

Never expose PostgreSQL or Redis directly to the public internet. Use firewall allowlists, private networking and least-privilege service credentials.

## 3. Environments

Maintain separate development, staging and production environments. Production data must not be copied to development without a documented lawful basis and strong anonymization. Use separate secrets, databases, object-storage buckets and signing keys.

## 4. Containers and releases

Docker Compose is acceptable for an initial controlled deployment. Pin image versions, scan images, run as non-root, minimize capabilities, mount only required volumes and define health checks. Keep deployment configuration and infrastructure changes reviewed and version-controlled. Do not put secrets in the repository or image layers.

Use backward-compatible database migrations. Deploy additive schema changes before application code that depends on them. Preserve supported offline desktop protocol compatibility. Have a tested rollback plan; database rollback may require forward repair rather than destructive schema reversal.

## 5. Desktop distribution

- Produce signed Windows installers.
- Sign update manifests and packages.
- Verify signatures before applying updates.
- Support staged rollout and rollback.
- Provide release notes and minimum supported version.
- Do not force an update that strands unacknowledged local work.
- Back up/migrate local databases safely before schema upgrades.
- Keep an offline installer available through a trusted channel.

## 6. Observability

Monitor:
- API availability and latency.
- PostgreSQL health, connections, locks and storage.
- Worker queue depth and age.
- Sync acknowledgment latency and failed events.
- Conflict rates and unresolved conflict age.
- Document upload and malware scan status.
- Backup freshness and restore-test status.
- Authentication anomalies and device revocations.
- Disk capacity, certificate expiry and service restarts.

Logs must avoid passwords, tokens, full tax identifiers and document contents. Use correlation IDs for tracing.

## 7. Backup and disaster recovery

Set product-approved recovery point objective (RPO) and recovery time objective (RTO). Use encrypted PostgreSQL backups with point-in-time recovery where available, independent object storage versioning/replication and off-provider backup copies. Test restoration regularly, including consistency between database document metadata and object blobs.

A successful backup job is not proof of recoverability. Record restore test results, duration, missing-object checks and corrective actions.

## 8. Operations and incident response

Prepare runbooks for database outage, VPS loss, storage outage, credential compromise, malware upload, sync backlog, corrupt local database, failed desktop update, suspected data exposure and government connector failure. Assign operational owners and escalation contacts before production.

## 9. Cost management

Begin with one appropriately sized VPS and independent object storage/backup. Track actual bandwidth, storage growth, backups, database IOPS, worker CPU and support costs. Split services only when measurements or availability requirements justify it. Moderate budget must not be achieved by omitting encryption, backups, monitoring or recovery testing.
