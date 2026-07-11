#!/usr/bin/env sh
set -eu

: "${CLIMBAR_DATABASE_URL:?Set CLIMBAR_DATABASE_URL to the target database URL}"
: "${CONFIRM_RESTORE:?Set CONFIRM_RESTORE=YES to restore a backup}"

if [ "$CONFIRM_RESTORE" != "YES" ]; then
  echo "Refusing to restore without CONFIRM_RESTORE=YES" >&2
  exit 1
fi

backup_path="${1:?Usage: CONFIRM_RESTORE=YES CLIMBAR_DATABASE_URL=... $0 backup.dump}"
pg_restore \
  --clean \
  --if-exists \
  --no-owner \
  --no-acl \
  --dbname "$CLIMBAR_DATABASE_URL" \
  "$backup_path"

printf 'Restored database backup: %s\n' "$backup_path"
