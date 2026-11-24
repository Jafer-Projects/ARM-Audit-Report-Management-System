#!/usr/bin/env bash
set -Eeuo pipefail
: "${AGE_IDENTITY:?Set AGE_IDENTITY to the offline backup decryption identity}"
: "${DB_BACKUP:?Set DB_BACKUP to the encrypted database backup}"
: "${OBJECTS_BACKUP:?Set OBJECTS_BACKUP to the encrypted object backup}"
: "${RESTORE_S3_ENDPOINT:?Set isolated restore S3 endpoint}"
: "${RESTORE_S3_BUCKET:?Set a dedicated empty restore bucket}"
: "${RESTORE_S3_ACCESS_KEY_ID:?Set restore bucket access key}"
: "${RESTORE_S3_SECRET_ACCESS_KEY_FILE:?Set restore bucket secret file}"
command -v age >/dev/null
command -v aws >/dev/null
name="arm-restore-drill-$RANDOM"
tmp="$(mktemp -d)"
cleanup(){ docker rm -f "$name" >/dev/null 2>&1 || true; rm -rf "$tmp"; }
trap cleanup EXIT
docker run -d --name "$name" -e POSTGRES_PASSWORD=restore-drill-only -e POSTGRES_DB=arm_restore postgres:16.8-bookworm >/dev/null
for _ in $(seq 1 60); do
  if docker exec "$name" pg_isready -U postgres -d arm_restore >/dev/null 2>&1; then break; fi
  sleep 2
done
docker exec "$name" pg_isready -U postgres -d arm_restore >/dev/null
age -d -i "$AGE_IDENTITY" "$DB_BACKUP" | docker exec -i "$name" pg_restore -U postgres -d arm_restore --no-owner --no-privileges
mkdir -p "$tmp/objects"
age -d -i "$AGE_IDENTITY" "$OBJECTS_BACKUP" | tar -xzf - -C "$tmp/objects"
export AWS_ACCESS_KEY_ID="$RESTORE_S3_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="$(tr -d '\r\n' < "$RESTORE_S3_SECRET_ACCESS_KEY_FILE")"
export AWS_EC2_METADATA_DISABLED=true
aws s3 sync "$tmp/objects/" "s3://$RESTORE_S3_BUCKET" --endpoint-url "$RESTORE_S3_ENDPOINT" --region "${RESTORE_S3_REGION:-us-east-1}" --no-progress
docker exec "$name" psql -U postgres -d arm_restore -v ON_ERROR_STOP=1 -c "SELECT version FROM schema_migrations ORDER BY version"
while IFS=' ' read -r key expected; do
  [ -n "$key" ] || continue
  aws s3 cp "s3://$RESTORE_S3_BUCKET/$key" "$tmp/check-object" --endpoint-url "$RESTORE_S3_ENDPOINT" --region "${RESTORE_S3_REGION:-us-east-1}" --no-progress >/dev/null
  actual="$(sha256sum "$tmp/check-object" | cut -d' ' -f1)"
  [ "$actual" = "$expected" ] || { echo "Object hash mismatch for $key" >&2; exit 1; }
done < <(docker exec "$name" psql -U postgres -d arm_restore -At -F ' ' -c "SELECT object_key,sha256 FROM document_objects ORDER BY object_key")
docker exec "$name" psql -U postgres -d arm_restore -v ON_ERROR_STOP=1 -c "SELECT count(*) AS tenants FROM tenants; SELECT count(*) AS users FROM users; SELECT count(*) AS audit_reports FROM audit_reports; SELECT count(*) AS journals FROM accounting_journals; SELECT count(*) AS documents FROM document_objects;"
echo "Restore drill completed. Database was restored to an isolated instance and every referenced object was restored to the isolated S3 bucket and SHA-256 verified."
