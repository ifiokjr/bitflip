#!/usr/bin/env bash

set -euo pipefail

readonly source_commit="087e5f0edc9284df573201a3b4aad7d1b43d646c"
readonly source_root="https://raw.githubusercontent.com/openbudgetfun/solana_kit/${source_commit}/config/programs"
readonly destination="${DEVENV_ROOT:?Run this script through devenv}/.tools/surfpool-programs"

checksum() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | awk '{ print $1 }'
	else
		shasum -a 256 "$1" | awk '{ print $1 }'
	fi
}

fetch() {
	local name="$1"
	local expected="$2"
	local target="${destination}/${name}"
	local temporary="${target}.download"

	if [[ -f "$target" ]] && [[ "$(checksum "$target")" == "$expected" ]]; then
		return
	fi

	mkdir -p "$destination"
	curl --fail --location --silent --show-error "${source_root}/${name}" --output "$temporary"

	local actual
	actual="$(checksum "$temporary")"
	if [[ "$actual" != "$expected" ]]; then
		rm -f "$temporary"
		echo "Checksum mismatch for ${name}: expected ${expected}, received ${actual}." >&2
		exit 1
	fi

	mv "$temporary" "$target"
}

fetch \
	"mpl_bubblegum-v0.12.0.so" \
	"5b86d4092287e989893ef8bc039a7f02229cd65ac1d1aa96201a1ead99ef51ff"
fetch \
	"spl_account_compression-v0.3.3.so" \
	"8b1bbb64e086e96fcf4a7db8bebd2db661eaf61d5692e0330517f1a96e75ee1f"
fetch \
	"noop-v0.2.0.so" \
	"7e5c4e0bc5eec7f1b0aa273f55f3c86b12c1c1de40e2a3d43f3e9624dba737dc"

echo "Verified Bubblegum Surfpool programs in ${destination}."
