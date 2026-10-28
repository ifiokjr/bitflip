import { open } from "node:fs/promises";

const checksumPath = process.argv[2];

if (checksumPath === undefined) {
	throw new Error("Specify the backup checksum path.");
}

// O_CREAT | O_EXCL refuses every existing directory entry, including dangling
// symlinks. Keeping the descriptor open prevents a later replacement redirecting
// writes to an attacker-selected target.
const checksum = await open(checksumPath, "wx", 0o600);

try {
	await checksum.writeFile(process.stdin);
	await checksum.sync();
} finally {
	await checksum.close();
}
