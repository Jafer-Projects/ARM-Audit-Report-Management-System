#!/usr/bin/env bash
set -Eeuo pipefail
: "${AGE_RECIPIENT:?Set AGE_RECIPIENT to the offline backup public key}"
: "${BACKUP_DIR:?Set BACKUP_DIR to a mounted, access-controlled backup directory}"
: "${S3_ENDPOINT:?Set S3_ENDPOINT}"
: "${S3_BUCKET:?Set S3_BUCKET}"
: "${S3_ACCESS_KEY_ID:?Set S3_ACCESS_KEY_ID}"
: "${S3_SECRET_ACCESS_KEY_FILE:?Set S3_SECRET_ACCESS_KEY_FILE}"
command -v age >/dev/null
command -v aws >/dev/null
export AWS_ACCESS_KEY_ID="$S3_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="$(tr -d '\r\n' < "$S3_SECRET_ACCESS_KEY_FILE")"
export AWS_EC2_METADATA_DISABLED=true
mkdir -p "$BACKUP_DIR"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
docker compose exec -T postgres pg_dump -U arm_owner -Fc arm | age -r "$AGE_RECIPIENT" -o "$BACKUP_DIR/arm-db-$stamp.dump.age"
aws s3 sync "s3://$S3_BUCKET" "$tmp/objects" --endpoint-url "$S3_ENDPOINT" --region "${S3_REGION:-us-east-1}" --no-progress
tar -C "$tmp/objects" -czf - . | age -r "$AGE_RECIPIENT" -o "$BACKUP_DIR/arm-objects-$stamp.tar.gz.age"
sha256sum "$BACKUP_DIR/arm-db-$stamp.dump.age" "$BACKUP_DIR/arm-objects-$stamp.tar.gz.age" > "$BACKUP_DIR/arm-$stamp.sha256"
chmod 600 "$BACKUP_DIR/arm-"*
echo "Encrypted database and private S3 object backups created: $stamp"
