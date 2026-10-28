import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import * as fs from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const runtime = process.argv[2];
assert(runtime === "docker" || runtime === "podman", "Expected Docker or Podman");

const server = "bitflip_server/bitflip_server_server";
const directory = fs.mkdtempSync(join(tmpdir(), "bitflip-container-"));
const context = join(directory, "context");
const probeImage = `bitflip-context-canary:${randomUUID()}`;
const secretMarker = `container-secret-canary-${randomUUID()}`;
const publicMarker = `container-public-canary-${randomUUID()}`;

/** Run build tools without hiding their diagnostics or tolerating failed commands. */
function run(command: string, args: string[], input?: string): void {
	execFileSync(command, args, {
		stdio: ["pipe", "inherit", "inherit"],
		...(input === undefined ? {} : { input }),
	});
}

/** Search every saved layer, including files removed or overwritten by later layers. */
function checkLayers(image: string, requirePublicMarker: boolean): void {
	const archive = join(directory, "image.tar");
	const layerPath = join(directory, "layer.tar");
	const contentsPath = join(directory, "layer-contents");
	run(runtime, ["image", "save", "--output", archive, image]);
	const manifest: unknown = JSON.parse(
		execFileSync("tar", ["-xOf", archive, "manifest.json"], { encoding: "utf8" }),
	);
	assert(Array.isArray(manifest) && manifest.length > 0, "Missing image manifest");
	const images: unknown[] = manifest;
	let foundPublicMarker = false;
	let layerCount = 0;

	for (const entry of images) {
		assert(typeof entry === "object" && entry !== null && "Layers" in entry);
		assert(Array.isArray(entry.Layers), "Missing image layers");
		const layers: unknown[] = entry.Layers;

		for (const layer of layers) {
			assert(typeof layer === "string" && !layer.startsWith("-"));
			const output = fs.openSync(layerPath, "w");

			try {
				execFileSync("tar", ["-xOf", archive, layer], { stdio: ["ignore", output, "inherit"] });
			} finally {
				fs.closeSync(output);
			}

			const contents = fs.openSync(contentsPath, "w");

			try {
				// tar also decodes compressed layers from OCI-backed image stores.
				execFileSync("tar", ["-xOf", layerPath], { stdio: ["ignore", contents, "inherit"] });
			} finally {
				fs.closeSync(contents);
			}

			foundPublicMarker = scanLayer(contentsPath) || foundPublicMarker;
			layerCount++;
		}
	}

	assert(layerCount > 0, "No layers inspected");
	assert(!requirePublicMarker || foundPublicMarker, "Context probe did not copy public source");
	console.info(`Inspected ${layerCount} layers of ${image}: no secret canary`);
}

/** Bound memory while retaining enough overlap to find a marker across read boundaries. */
function scanLayer(path: string): boolean {
	const file = fs.openSync(path, "r");
	const buffer = Buffer.alloc(64 * 1024);
	const overlap = Math.max(secretMarker.length, publicMarker.length);
	let tail = Buffer.alloc(0);
	let foundPublicMarker = false;

	try {
		let size: number;

		while ((size = fs.readSync(file, buffer)) > 0) {
			const chunk = Buffer.concat([tail, buffer.subarray(0, size)]);
			assert(!chunk.includes(secretMarker), "Secret canary entered an image layer");
			foundPublicMarker = foundPublicMarker || chunk.includes(publicMarker);
			tail = chunk.subarray(-overlap);
		}
	} finally {
		fs.closeSync(file);
	}

	return foundPublicMarker;
}

try {
	// Never read local ignored credentials. Build a clean tracked checkout plus public web output.
	fs.mkdirSync(context);
	const sources = join(directory, "sources.tar");
	run("git", ["archive", "--format=tar", "--output", sources, "HEAD"]);
	run("tar", ["-xf", sources, "-C", context]);
	fs.cpSync(join(server, "web/app"), join(context, server, "web/app"), { recursive: true });

	for (
		const path of [
			"config/firebase_service_account_key.json",
			"config/passwords.yaml",
			"config/unexpected-credential.json",
			"config/unexpected-credential.yaml",
			".env",
			".env.production",
			"operator-keypair.json",
			"private.pem",
			"private.key",
		]
	) {
		const target = join(context, server, path);
		fs.mkdirSync(dirname(target), { recursive: true });
		fs.writeFileSync(target, secretMarker, { flag: "wx" });
	}

	fs.writeFileSync(join(context, server, "lib/container_canary.dart"), `// ${publicMarker}\n`);

	run(runtime, [
		"build",
		"--file",
		"-",
		"--tag",
		probeImage,
		context,
	], "FROM scratch\nCOPY . /\n");
	checkLayers(probeImage, true);
	run(runtime, [
		"build",
		"--file",
		join(context, server, "Dockerfile"),
		"--tag",
		"bitflip-server:verification",
		context,
	]);
	checkLayers("bitflip-server:verification", false);
} finally {
	// Cleanup failures remain visible, and temporary canaries are always removed.
	try {
		run(runtime, ["image", "rm", "--force", probeImage]);
	} finally {
		fs.rmSync(directory, { recursive: true, force: true });
	}
}
