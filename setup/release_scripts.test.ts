import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const repositoryRoot = process.cwd();

function runScript(path: string, environment: NodeJS.ProcessEnv) {
	return spawnSync("bash", [join(repositoryRoot, path)], {
		cwd: repositoryRoot,
		encoding: "utf8",
		env: { PATH: process.env.PATH, ...environment },
	});
}

test("release smoke rejects non-HTTPS deployment origins", () => {
	const result = runScript("setup/scripts/release_smoke.sh", {
		BITFLIP_API_BASE_URL: "http://staging.example.com",
		BITFLIP_WEB_BASE_URL: "https://staging.example.com",
	});

	assert.notEqual(result.status, 0);
	assert.match(result.stderr, /credential-free HTTPS origin/);
});

test("release smoke checks health, web, metadata, and artwork", () => {
	const directory = mkdtempSync(join(tmpdir(), "bitflip-smoke-"));
	try {
		const fakeCurl = join(directory, "curl");
		writeFileSync(
			fakeCurl,
			`#!/usr/bin/env bash
set -euo pipefail
headers=""
url=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dump-header) headers="$2"; shift 2 ;;
    http*) url="$1"; shift ;;
    *) shift ;;
  esac
done
case "$url" in
  */metadata/*) type="application/json; charset=utf-8" ;;
  */art/*) type="image/svg+xml" ;;
  *) type="text/html; charset=utf-8" ;;
esac
printf 'HTTP/2 200\\r\\ncontent-type: %s\\r\\n\\r\\n' "$type" >"$headers"
`,
		);
		chmodSync(fakeCurl, 0o755);

		const result = runScript("setup/scripts/release_smoke.sh", {
			BITFLIP_API_BASE_URL: "https://api-staging.example.com",
			BITFLIP_SMOKE_GAME_INDEX: "0",
			BITFLIP_SMOKE_SECTION_INDEX: "7",
			BITFLIP_WEB_BASE_URL: "https://staging.example.com",
			PATH: `${directory}:${process.env.PATH ?? ""}`,
		});

		assert.equal(result.status, 0, result.stderr);
		assert.match(result.stdout, /PASS readiness/);
		assert.match(result.stdout, /PASS metadata/);
		assert.match(result.stdout, /PASS artwork/);
	} finally {
		rmSync(directory, { force: true, recursive: true });
	}
});

test("backup refuses to overwrite an existing artifact", () => {
	const directory = mkdtempSync(join(tmpdir(), "bitflip-backup-"));
	try {
		const output = join(directory, "existing.dump");
		writeFileSync(output, "do not overwrite");
		const result = runScript("setup/scripts/postgres_backup.sh", {
			BITFLIP_BACKUP_OUTPUT: output,
			PGDATABASE: "bitflip",
			PGHOST: "database.example.com",
			PGPASSWORD: "test-only",
			PGPORT: "5432",
			PGUSER: "bitflip",
		});

		assert.notEqual(result.status, 0);
		assert.match(result.stderr, /Refusing to overwrite/);
	} finally {
		rmSync(directory, { force: true, recursive: true });
	}
});

test("restore verification requires a named disposable target", () => {
	const result = runScript("setup/scripts/postgres_restore_verify.sh", {
		BITFLIP_BACKUP_FILE: "/tmp/not-read.dmp",
		BITFLIP_RESTORE_CONFIRM: "production",
		BITFLIP_RESTORE_DATABASE: "production",
		PGDATABASE: "production",
		PGHOST: "database.example.com",
		PGPASSWORD: "test-only",
		PGPORT: "5432",
		PGUSER: "bitflip",
	});

	assert.notEqual(result.status, 0);
	assert.match(result.stderr, /distinct, confirmed database/);
});
