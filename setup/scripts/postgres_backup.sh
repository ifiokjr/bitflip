#!/usr/bin/env bash
set -euo pipefail

: "${PGHOST:?Set PGHOST.}"
: "${PGPORT:?Set PGPORT.}"
: "${PGDATABASE:?Set PGDATABASE.}"
: "${PGUSER:?Set PGUSER.}"
: "${PGPASSWORD:?Set PGPASSWORD through the secret manager.}"
: "${BITFLIP_BACKUP_OUTPUT:?Set BITFLIP_BACKUP_OUTPUT to a new .dump path.}"

if [[ -e "$BITFLIP_BACKUP_OUTPUT" ]]; then
	echo "Refusing to overwrite existing backup: $BITFLIP_BACKUP_OUTPUT" >&2
	exit 1
fi

umask 077
mkdir -p "$(dirname "$BITFLIP_BACKUP_OUTPUT")"
pg_dump \
	--format=custom \
	--compress=9 \
	--no-owner \
	--no-acl \
	--file="$BITFLIP_BACKUP_OUTPUT"
pg_restore --list "$BITFLIP_BACKUP_OUTPUT" >/dev/null

if command -v sha256sum >/dev/null 2>&1; then
	sha256sum "$BITFLIP_BACKUP_OUTPUT" >"$BITFLIP_BACKUP_OUTPUT.sha256"
else
	shasum -a 256 "$BITFLIP_BACKUP_OUTPUT" >"$BITFLIP_BACKUP_OUTPUT.sha256"
fi

echo "Backup created and readable: $BITFLIP_BACKUP_OUTPUT"
