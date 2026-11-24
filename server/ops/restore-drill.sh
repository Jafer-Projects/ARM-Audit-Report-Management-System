#!/usr/bin/env bash
set -Eeuo pipefail
: "${AGE_IDENTITY:?Set AGE_IDENTITY to the offline backup decryption identity}"
: "${DB_BACKUP:?Set DB_BACKUP to the encrypted database backup}"
: "${OBJECTS_BACKUP:?Set OBJECTS_BACKUP to the encrypted object backup}"
command -v age >/dev/null
name="arm-restore-drill-$RANDOM"
cleanup(){ docker rm -f "$name" >/dev/null 2>&1 || true; }
trap cleanup EXIT
docker run -d --name "$name" -e POSTGRES_PASSWORD=restore-drill-only -e POSTGRES_DB=arm_restore postgres:16.8-bookworm >/dev/null
for _ in $(seq 1 60); do
  if docker exec "$name" pg_isready -U postgres -d arm_restore >/dev/null 2>&1; then break; fi
  sleep 2
done
docker exec "$name" pg_isready -U postgres -d arm_restore >/dev/null
age -d -i "$AGE_IDENTITY" "$DB_BACKUP" | docker exec -i "$name" pg_restore -U postgres -d arm_restore --no-owner --no-privileges
age -d -i "$AGE_IDENTITY" "$OBJECTS_BACKUP" | tar -tzf - >/dev/null
docker exec "$name" psql -U postgres -d arm_restore -v ON_ERROR_STOP=1 -c "SELECT version FROM schema_migrations ORDER BY version"
docker exec "$name" psql -U postgres -d arm_restore -v ON_ERROR_STOP=1 -c "SELECT count(*) AS tenants FROM tenants; SELECT count(*) AS users FROM users; SELECT count(*) AS audit_reports FROM audit_reports; SELECT count(*) AS journals FROM accounting_journals; SELECT count(*) AS documents FROM document_objects;"
echo "Restore drill completed against an isolated disposable PostgreSQL instance. Document-object archive integrity was checked; restore it into an isolated object volume before declaring full recovery verified."
