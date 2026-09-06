#!/usr/bin/env bash
set -euo pipefail

: "${BITFLIP_API_BASE_URL:?Set BITFLIP_API_BASE_URL to the deployed API origin.}"
: "${BITFLIP_WEB_BASE_URL:?Set BITFLIP_WEB_BASE_URL to the deployed web origin.}"

validate_origin() {
	local name="$1"
	local value="$2"

	if [[ "$value" != https://* ]] || [[ "$value" == *"@"* ]] || [[ "$value" == *"?"* ]] || [[ "$value" == *"#"* ]]; then
		echo "$name must be a credential-free HTTPS origin without a query or fragment." >&2
		exit 1
	fi
}

request() {
	local label="$1"
	local url="$2"
	local content_type="${3:-}"
	local headers

	headers="$(mktemp)"
	if ! curl \
		--fail \
		--silent \
		--show-error \
		--location \
		--proto '=https' \
		--proto-redir '=https' \
		--connect-timeout 5 \
		--max-time 15 \
		--retry 2 \
		--retry-all-errors \
		--dump-header "$headers" \
		--output /dev/null \
		"$url"; then
		rm -f "$headers"
		return 1
	fi
	if [[ -n "$content_type" ]] && ! grep -Eiq "^content-type:[[:space:]]*$content_type" "$headers"; then
		rm -f "$headers"
		echo "$label returned an unexpected content type." >&2
		exit 1
	fi
	rm -f "$headers"
	echo "PASS $label"
}

validate_origin BITFLIP_API_BASE_URL "$BITFLIP_API_BASE_URL"
validate_origin BITFLIP_WEB_BASE_URL "$BITFLIP_WEB_BASE_URL"

api="${BITFLIP_API_BASE_URL%/}"
web="${BITFLIP_WEB_BASE_URL%/}"

request startup "$api/startupz"
request liveness "$api/livez"
request readiness "$api/readyz"
request web-app "$web/" 'text/html([;[:space:]]|$)'

if [[ -n "${BITFLIP_SMOKE_GAME_INDEX:-}" || -n "${BITFLIP_SMOKE_SECTION_INDEX:-}" ]]; then
	: "${BITFLIP_SMOKE_GAME_INDEX:?Set both smoke-test indices, or neither.}"
	: "${BITFLIP_SMOKE_SECTION_INDEX:?Set both smoke-test indices, or neither.}"
	if [[ ! "$BITFLIP_SMOKE_GAME_INDEX" =~ ^[0-9]+$ ]] ||
		((10#$BITFLIP_SMOKE_GAME_INDEX > 255)) ||
		[[ ! "$BITFLIP_SMOKE_SECTION_INDEX" =~ ^[0-9]+$ ]] ||
		((10#$BITFLIP_SMOKE_SECTION_INDEX > 255)); then
		echo "Smoke-test game and section indices must be between 0 and 255." >&2
		exit 1
	fi
	request metadata \
		"$web/metadata/$BITFLIP_SMOKE_GAME_INDEX/$BITFLIP_SMOKE_SECTION_INDEX.json" \
		'application/json([;[:space:]]|$)'
	request artwork \
		"$web/art/$BITFLIP_SMOKE_GAME_INDEX/$BITFLIP_SMOKE_SECTION_INDEX.svg" \
		'image/svg\+xml([;[:space:]]|$)'
fi
