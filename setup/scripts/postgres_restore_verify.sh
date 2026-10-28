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

: "${BITFLIP_BACKUP_SHA256:?Set BITFLIP_BACKUP_SHA256 from the authenticated release record.}"

if [[ ! "$BITFLIP_BACKUP_SHA256" =~ ^[[:xdigit:]]{64}$ ]]; then
	echo "BITFLIP_BACKUP_SHA256 must contain exactly 64 hexadecimal SHA-256 digits." >&2
	exit 1
fi

expected_sha256="$(printf '%s' "$BITFLIP_BACKUP_SHA256" | tr '[:upper:]' '[:lower:]')"
umask 077
restore_directory="$(mktemp -d "${TMPDIR:-/tmp}/bitflip-restore-verify.XXXXXXXX")"
restore_started=false

cleanup() {
	rm -rf -- "$restore_directory"
	if [[ "$restore_started" == "true" ]] &&
		[[ "${BITFLIP_KEEP_RESTORE_DATABASE:-false}" != "true" ]]; then
		dropdb --if-exists "$BITFLIP_RESTORE_DATABASE"
	fi
}
trap cleanup EXIT

# Authenticate and restore the same private bytes, even if storage replaces the source.
restore_file="$restore_directory/backup.dump"
cp -- "$BITFLIP_BACKUP_FILE" "$restore_file"

if command -v sha256sum >/dev/null 2>&1; then
	checksum="$(sha256sum <"$restore_file")"
else
	checksum="$(shasum -a 256 <"$restore_file")"
fi

if [[ "${checksum%% *}" != "$expected_sha256" ]]; then
	echo "Backup SHA-256 digest mismatch; refusing restore verification." >&2
	exit 1
fi

restore_started=true
dropdb --if-exists "$BITFLIP_RESTORE_DATABASE"
createdb "$BITFLIP_RESTORE_DATABASE"
pg_restore \
	--exit-on-error \
	--no-owner \
	--no-acl \
	--dbname="$BITFLIP_RESTORE_DATABASE" \
	"$restore_file"

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
