/* eslint-disable @typescript-eslint/no-non-null-assertion */
import fs from "node:fs/promises";
import { arch } from "node:os";
import { chromium } from "playwright";
import { Open } from "unzipper";
import { EXTENSIONS as E, EXTENSIONS_FOLDER } from "./constants.ts";

const EXTENSIONS: ExtensionMeta[] = Object.entries(E).map((
	[name, details],
) => ({ name, ...details }));

interface ExtensionMeta {
	name: string;
	id: string;
	version: string;
	path: string;
}

async function getBrowerVersion() {
	const browser = await chromium.launch({ headless: true });
	const version = browser.version();

	await browser.close();
	return version;
}

function getArch() {
	switch (arch()) {
		case "arm":
		case "arm64": {
			return "arm";
		}
		case "ia32":
		case "x32": {
			return "x86-32";
		}
		default: {
			return "x86-64";
		}
	}
}

const browserVersion = await getBrowerVersion();
const naclArch = getArch();

function bufferToBytes(buffer: ArrayBuffer) {
	const bytes = new Uint8Array(buffer);
	let publicKeyLength, signatureLength, header, zipStartOffset;

	if (bytes[4] === 2) {
		header = 16;
		publicKeyLength = 0 + bytes[8]! + (bytes[9]! << 8) + (bytes[10]! << 16) +
			(bytes[11]! << 24);
		signatureLength = 0 + bytes[12]! + (bytes[13]! << 8) + (bytes[14]! << 16) +
			(bytes[15]! << 24);
		zipStartOffset = header + publicKeyLength + signatureLength;
	} else {
		publicKeyLength = 0 + bytes[8]! + (bytes[9]! << 8) + (bytes[10]! << 16) +
			(bytes[11]! << 24 >>> 0);
		zipStartOffset = 12 + publicKeyLength;
	}
	// 16 = Magic number (4), CRX format version (4), lengths (2x4)

	return new Uint8Array(buffer, zipStartOffset);
}

async function getZip(url: string) {
	const response = await fetch(url);
	const buffer = await response.arrayBuffer();
	return bufferToBytes(buffer);
}

async function download(props: ExtensionMeta) {
	const zipFile = `${props.path}.zip`;
	try {
		const stat = await fs.lstat(props.path);

		if (stat.isDirectory()) {
			console.warn("Extension:", props.name, "has already been downloaded");

			return;
		}
	} catch {
		// ignore
	}

	// For now ignore if already exisits.
	// TODO check the version first to see if installation should happen.

	console.log("Downloading extension:", props.name, "with id:", props.id);

	const url =
		`https://clients2.google.com/service/update2/crx?response=redirect&prodversion=${browserVersion}&x=id%3D${props.id}%26installsource%3Dondemand%26uc&nacl_arch=${naclArch}&acceptformat=crx2,crx3`;
	const bytes = await getZip(url);

	await fs.writeFile(zipFile, bytes);

	const directory = await Open.file(zipFile);
	await directory.extract({ path: props.path });
}

await fs.mkdir(EXTENSIONS_FOLDER, { recursive: true });

const promises: Array<Promise<void>> = [];

for (const extension of EXTENSIONS) {
	// eslint-disable-next-line unicorn/prefer-top-level-await
	promises.push(download(extension));
}

await Promise.all(promises);
