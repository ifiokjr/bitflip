#!/usr/bin/env bash
set -euo pipefail

: "${PGHOST:?Set PGHOST.}"
: "${PGPORT:?Set PGPORT.}"
: "${PGUSER:?Set PGUSER.}"
: "${PGPASSWORD:?Set PGPASSWORD through the secret manager.}"
: "${BITFLIP_BACKUP_FILE:?Set BITFLIP_BACKUP_FILE to the .dump to verify.}"
: "${BITFLIP_RESTORE_DATABASE:?Set BITFLIP_RESTORE_DATABASE to a disposable database ending in _restore_verify.}"
: "${BITFLIP_RESTORE_CONFIRM:?Set BITFLIP_RESTORE_CONFIRM to the same disposable database name.}"

if [[ "$BITFLIP_RESTORE_DATABASE" != *_restore_verify ]] ||
	[[ "$BITFLIP_RESTORE_CONFIRM" != "$BITFLIP_RESTORE_DATABASE" ]] ||
	[[ "$BITFLIP_RESTORE_DATABASE" == "${PGDATABASE:-}" ]]; then
	echo "Restore verification requires a distinct, confirmed database ending in _restore_verify." >&2
	exit 1
fi
if [[ ! -f "$BITFLIP_BACKUP_FILE" ]]; then
	echo "Backup does not exist: $BITFLIP_BACKUP_FILE" >&2
	exit 1
fi

cleanup() {
	if [[ "${BITFLIP_KEEP_RESTORE_DATABASE:-false}" != "true" ]]; then
		dropdb --if-exists "$BITFLIP_RESTORE_DATABASE"
	fi
}
trap cleanup EXIT

dropdb --if-exists "$BITFLIP_RESTORE_DATABASE"
createdb "$BITFLIP_RESTORE_DATABASE"
pg_restore \
	--exit-on-error \
	--no-owner \
	--no-acl \
	--dbname="$BITFLIP_RESTORE_DATABASE" \
	"$BITFLIP_BACKUP_FILE"

table_count="$(
	PGDATABASE="$BITFLIP_RESTORE_DATABASE" psql \
		--no-align \
		--tuples-only \
		--set=ON_ERROR_STOP=1 \
		--command="SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';"
)"
if [[ ! "$table_count" =~ ^[1-9][0-9]*$ ]]; then
	echo "Restore verification found no public tables." >&2
	exit 1
fi

echo "Restore verified with $table_count public tables in $BITFLIP_RESTORE_DATABASE."
