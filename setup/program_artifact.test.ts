import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

const repositoryRoot = process.cwd();
const programDirectory = join(repositoryRoot, "bitflip_program");

const SECURITY_TXT_BEGIN = "=======BEGIN SECURITY.TXT V1=======\u0000";
const SECURITY_TXT_END = "=======END SECURITY.TXT V1=======\u0000";

/// Read the deployable artifact named by the `verify:release-artifact` task.
/// Returns `undefined` when the artifact has not been built, so a plain `test:*
/// run does not fail for a missing build product.
function releaseArtifact(): Buffer | undefined {
	const path = process.env.PINA_SBF_ARTIFACT;

	if (!path || !existsSync(path)) {
		return undefined;
	}

	return readFileSync(path);
}

test("the security.txt block is embedded in the release artifact", () => {
	const artifact = releaseArtifact();
	if (!artifact) {
		// `verify:release-artifact` builds the artifact first; without it there is
		// nothing to inspect and no failure to report.
		return;
	}

	const start = artifact.indexOf(SECURITY_TXT_BEGIN, 0, "utf8");
	assert.notEqual(
		start,
		-1,
		"the release artifact must contain the security.txt begin marker",
	);

	const end = artifact.indexOf(SECURITY_TXT_END, start, "utf8");
	assert.notEqual(end, -1, "the release artifact must contain the end marker");

	// Parse exactly as `query-security-txt` does: alternating NUL-terminated
	// field names and values. A block that scans but does not parse is invisible
	// to explorers while the source claims disclosure.
	const body = artifact
		.subarray(start + SECURITY_TXT_BEGIN.length, end)
		.toString("utf8");
	assert.ok(body.endsWith("\u0000"), "the block body must end with a NUL");

	const parts = body.slice(0, -1).split("\u0000");
	assert.equal(parts.length % 2, 0, "fields and values must alternate");

	const fields = new Map<string, string>();
	for (let index = 0; index < parts.length; index += 2) {
		fields.set(parts[index], parts[index + 1]);
	}

	for (const required of ["name", "project_url", "policy", "contacts"]) {
		const value = fields.get(required);
		assert.ok(value, `embedded security.txt must carry ${required}`);
		assert.ok(value.length > 0, `${required} must not be empty`);
	}

	for (const contact of fields.get("contacts")!.split(",")) {
		const [kind, value] = contact.split(":", 2);
		assert.ok(
			["email", "discord", "telegram", "twitter", "link", "other"].includes(
				kind.trim(),
			),
			`unsupported contact type ${kind}`,
		);
		assert.ok(value?.trim(), "contact value must not be empty");
	}
});

// `link_section = ".security.txt"` and `#[used]` each emit a writable,
// allocatable section into the SBF ELF, which makes the loader misplace the
// program image: instructions return success without executing. The real-SBF
// suite catches the behaviour, but it needs a validator, so this cheap source
// check keeps the attributes from being reintroduced.
test("the security.txt static avoids image-breaking attributes", () => {
	const source = readFileSync(join(programDirectory, "src", "lib.rs"), "utf8");

	// Only attribute lines count: the doc comment above the static explains why
	// these attributes are harmful and would otherwise trip the check.
	const attributeLines = source
		.split("\n")
		.map((line) => line.trim())
		.filter((line) => line.startsWith("#["));

	assert.ok(
		attributeLines.some((line) => line.includes("no_mangle")),
		"SECURITY_TXT must be exported with no_mangle so it survives optimization",
	);
	assert.ok(
		!attributeLines.some((line) => line.includes("link_section")),
		"link_section must not be used: it corrupts the SBF program image",
	);

	const usedLine = attributeLines.findIndex((line) => line === "#[used]");
	assert.equal(
		usedLine,
		-1,
		"#[used] must not appear in the source: it corrupts the SBF program image",
	);
});
