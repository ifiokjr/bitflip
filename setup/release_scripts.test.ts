import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, existsSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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

/// Install a stub `curl` on `directory` that answers every request with a
/// content type derived from its path, and return the `PATH` entry to use.
/// Keeps smoke-script tests hermetic instead of resolving real hostnames.
function installFakeCurl(directory: string) {
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

	return `${directory}:${process.env.PATH ?? ""}`;
}

test("release smoke rejects non-HTTPS deployment origins", () => {
	const result = runScript("setup/scripts/release_smoke.sh", {
		BITFLIP_API_BASE_URL: "http://staging.example.com",
		BITFLIP_WEB_BASE_URL: "https://staging.example.com",
	});

	assert.notEqual(result.status, 0);
	assert.match(result.stderr, /credential-free HTTPS origin/);
});

// The workflow-dispatch inputs reach this script through the environment, which
// makes `validate_origin` the one untrusted-input boundary in the release path.
// Each rejected shape gets a case so a future edit cannot quietly widen it.
test("release smoke rejects credential-bearing and ambiguous origins", () => {
	const rejected = [
		"https://user:pass@staging.example.com",
		"https://staging.example.com?redirect=evil",
		"https://staging.example.com#fragment",
	];

	for (const origin of rejected) {
		const result = runScript("setup/scripts/release_smoke.sh", {
			BITFLIP_API_BASE_URL: origin,
			BITFLIP_WEB_BASE_URL: "https://staging.example.com",
		});

		assert.notEqual(result.status, 0, `${origin} must be rejected`);
		assert.match(result.stderr, /credential-free HTTPS origin/);
	}
});

test("release smoke requires both smoke indices, bound to a byte range", () => {
	const directory = mkdtempSync(join(tmpdir(), "bitflip-indices-"));
	try {
		const fakeCurl = installFakeCurl(directory);
		const origins = {
			BITFLIP_API_BASE_URL: "https://api.example.com",
			BITFLIP_WEB_BASE_URL: "https://example.com",
			PATH: fakeCurl,
		};

		// One without the other is a configuration mistake, not a partial run.
		const partial = runScript("setup/scripts/release_smoke.sh", {
			...origins,
			BITFLIP_SMOKE_GAME_INDEX: "1",
		});
		assert.notEqual(partial.status, 0);
		assert.match(partial.stderr, /Set both smoke-test indices, or neither/);

		// The indices address `u8` game and section fields on-chain, so values
		// that do not fit must fail before any request is built.
		for (
			const [game, section] of [
				["256", "0"],
				["-1", "0"],
				["0", "256"],
				["0", "1.5"],
			]
		) {
			const result = runScript("setup/scripts/release_smoke.sh", {
				...origins,
				BITFLIP_SMOKE_GAME_INDEX: game,
				BITFLIP_SMOKE_SECTION_INDEX: section,
			});

			assert.notEqual(
				result.status,
				0,
				`${game}/${section} must be rejected`,
			);
			assert.match(result.stderr, /between 0 and 255/);
		}
	} finally {
		rmSync(directory, { force: true, recursive: true });
	}
});

test("release smoke checks health, web, metadata, and artwork", () => {
	const directory = mkdtempSync(join(tmpdir(), "bitflip-smoke-"));
	try {
		const path = installFakeCurl(directory);

		const result = runScript("setup/scripts/release_smoke.sh", {
			BITFLIP_API_BASE_URL: "https://api-staging.example.com",
			BITFLIP_SMOKE_GAME_INDEX: "0",
			BITFLIP_SMOKE_SECTION_INDEX: "7",
			BITFLIP_WEB_BASE_URL: "https://staging.example.com",
			PATH: path,
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

function restoreFixture() {
	const directory = mkdtempSync(join(tmpdir(), "bitflip-restore-"));
	const backup = join(directory, "source.dump");
	const calls = join(directory, "database-calls");
	const restored = join(directory, "restored.dump");
	const archive = Buffer.from("PGDMP original archive\n");
	const digest = createHash("sha256").update(archive).digest("hex");
	writeFileSync(backup, archive);

	for (const command of ["dropdb", "createdb", "pg_restore", "psql"]) {
		const executable = join(directory, command);
		writeFileSync(
			executable,
			`#!/usr/bin/env bash
set -euo pipefail
command="\${0##*/}"
printf '%s\\n' "$command" >> "$DATABASE_CALL_LOG"
case "$command" in
  dropdb)
    if [[ "\${REPLACE_SOURCE:-false}" == true ]]; then
      printf 'substituted after authentication' > "$BITFLIP_BACKUP_FILE"
    fi
    ;;
  pg_restore)
    printf '%s' "\${!#}" > "$RESTORED_BACKUP.path"
    if [[ "\${FAIL_RESTORE:-false}" == true ]]; then exit 42; fi
    cp -- "\${!#}" "$RESTORED_BACKUP"
    ;;
  psql) printf '1\\n' ;;
esac
`,
		);
		chmodSync(executable, 0o755);
	}

	return {
		archive,
		backup,
		calls,
		digest,
		directory,
		restored,
		run(overrides: NodeJS.ProcessEnv = {}) {
			return runScript("setup/scripts/postgres_restore_verify.sh", {
				BITFLIP_BACKUP_FILE: backup,
				BITFLIP_BACKUP_SHA256: digest,
				BITFLIP_RESTORE_CONFIRM: "bitflip_restore_verify",
				BITFLIP_RESTORE_DATABASE: "bitflip_restore_verify",
				DATABASE_CALL_LOG: calls,
				PATH: `${directory}:${process.env.PATH ?? ""}`,
				PGDATABASE: "bitflip",
				PGHOST: "127.0.0.1",
				PGPASSWORD: "test-only",
				PGPORT: "5432",
				PGUSER: "bitflip",
				RESTORED_BACKUP: restored,
				TMPDIR: directory,
				...overrides,
			});
		},
	};
}

test("restore rejects missing, malformed, and mismatched trusted digests before database commands", () => {
	const fixture = restoreFixture();
	try {
		for (
			const digest of [
				undefined,
				"",
				"a".repeat(63),
				"g".repeat(64),
				`${fixture.digest}  source.dump`,
				`${fixture.digest}\n`,
				"0".repeat(64),
			]
		) {
			const result = fixture.run({ BITFLIP_BACKUP_SHA256: digest });
			assert.notEqual(result.status, 0, `Digest ${String(digest)} must fail`);
			assert.match(result.stderr, /SHA256|SHA-256/);
			assert.equal(existsSync(fixture.calls), false, "No database command may run");
		}
	} finally {
		rmSync(fixture.directory, { force: true, recursive: true });
	}
});

test("restore rejects a one-byte substitution even with a matching adjacent checksum", () => {
	const fixture = restoreFixture();
	try {
		const tampered = Buffer.from(fixture.archive);
		tampered[0] = 0;
		writeFileSync(fixture.backup, tampered);
		const attackerDigest = createHash("sha256").update(tampered).digest("hex");
		writeFileSync(`${fixture.backup}.sha256`, `${attackerDigest}  ${fixture.backup}\n`);

		const result = fixture.run();
		assert.notEqual(result.status, 0);
		assert.match(result.stderr, /SHA-256.*mismatch/);
		assert.equal(existsSync(fixture.calls), false, "No database command may run");
		assert.equal(
			readdirSync(fixture.directory).some((name) => name.startsWith("bitflip-restore-verify.")),
			false,
			"Rejected snapshots must be cleaned up",
		);
	} finally {
		rmSync(fixture.directory, { force: true, recursive: true });
	}
});

test("restore uses the authenticated snapshot even if the original is replaced", () => {
	const fixture = restoreFixture();
	try {
		const result = fixture.run({
			BITFLIP_BACKUP_SHA256: fixture.digest.toUpperCase(),
			REPLACE_SOURCE: "true",
		});
		assert.equal(result.status, 0, result.stderr);
		assert.match(result.stdout, /Restore verified with 1 public tables/);
		assert.deepEqual(readFileSync(fixture.restored), fixture.archive);
		assert.notDeepEqual(readFileSync(fixture.backup), fixture.archive);
		assert.deepEqual(readFileSync(fixture.calls, "utf8").trim().split("\n"), [
			"dropdb",
			"createdb",
			"pg_restore",
			"psql",
			"dropdb",
		]);
		const snapshot = readFileSync(`${fixture.restored}.path`, "utf8");
		assert.notEqual(snapshot, fixture.backup);
		assert.equal(existsSync(snapshot), false, "Snapshot must be cleaned up");
	} finally {
		rmSync(fixture.directory, { force: true, recursive: true });
	}
});

test("restore fails closed when hashing fails", () => {
	const fixture = restoreFixture();
	try {
		const sha256sum = join(fixture.directory, "sha256sum");
		writeFileSync(sha256sum, "#!/usr/bin/env bash\nexit 41\n");
		chmodSync(sha256sum, 0o755);
		const result = fixture.run();
		assert.equal(result.status, 41);
		assert.equal(existsSync(fixture.calls), false, "No database command may run");
		assert.equal(
			readdirSync(fixture.directory).some((name) => name.startsWith("bitflip-restore-verify.")),
			false,
			"Unverified snapshots must be cleaned up",
		);
	} finally {
		rmSync(fixture.directory, { force: true, recursive: true });
	}
});

test("restore failure cleans up the disposable database and authenticated snapshot", () => {
	const fixture = restoreFixture();
	try {
		const result = fixture.run({ FAIL_RESTORE: "true" });
		assert.equal(result.status, 42);
		assert.deepEqual(readFileSync(fixture.calls, "utf8").trim().split("\n"), [
			"dropdb",
			"createdb",
			"pg_restore",
			"dropdb",
		]);
		const snapshot = readFileSync(`${fixture.restored}.path`, "utf8");
		assert.equal(existsSync(snapshot), false, "Snapshot must be cleaned up");
	} finally {
		rmSync(fixture.directory, { force: true, recursive: true });
	}
});
