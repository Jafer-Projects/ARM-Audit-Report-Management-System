#!/usr/bin/env bash
set -Eeuo pipefail
: "${AGE_RECIPIENT:?Set AGE_RECIPIENT to the offline backup public key}"
: "${BACKUP_DIR:?Set BACKUP_DIR to a mounted, access-controlled backup directory}"
command -v age >/dev/null
mkdir -p "$BACKUP_DIR"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
docker compose exec -T postgres pg_dump -U arm_owner -Fc arm | age -r "$AGE_RECIPIENT" -o "$BACKUP_DIR/arm-db-$stamp.dump.age"
docker compose exec -T api tar -C /data/objects -czf - . | age -r "$AGE_RECIPIENT" -o "$BACKUP_DIR/arm-objects-$stamp.tar.gz.age"
sha256sum "$BACKUP_DIR/arm-db-$stamp.dump.age" "$BACKUP_DIR/arm-objects-$stamp.tar.gz.age" > "$BACKUP_DIR/arm-$stamp.sha256"
chmod 600 "$BACKUP_DIR/arm-"*
echo "Encrypted database and document-object backups created: $stamp"
