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
	checksum_command=(sha256sum)
else
	checksum_command=(shasum -a 256)
fi

# The writer creates the sidecar exclusively; shell noclobber still follows
# symlinks to devices and FIFOs. A prior existence check would also be racy.
"${checksum_command[@]}" "$BITFLIP_BACKUP_OUTPUT" |
	node --experimental-strip-types \
		"$(dirname "${BASH_SOURCE[0]}")/write_backup_checksum.ts" \
		"$BITFLIP_BACKUP_OUTPUT.sha256"

echo "Backup created and readable: $BITFLIP_BACKUP_OUTPUT"
