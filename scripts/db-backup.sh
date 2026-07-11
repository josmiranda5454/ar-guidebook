#!/usr/bin/env sh
set -eu

: "${CLIMBAR_DATABASE_URL:?Set CLIMBAR_DATABASE_URL to the database URL}"
output_path="${1:-climbar-$(date -u +%Y%m%dT%H%M%SZ).dump}"

umask 077
pg_dump \
  --format=custom \
  --no-owner \
  --no-acl \
  --file "$output_path" \
  "$CLIMBAR_DATABASE_URL"

printf 'Created database backup: %s\n' "$output_path"
