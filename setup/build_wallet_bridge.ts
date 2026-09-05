import { build } from "esbuild";

await build({
	bundle: true,
	entryPoints: ["setup/wallet_bridge.ts"],
	format: "iife",
	logLevel: "info",
	minify: true,
	outfile: "bitflip_app/web/wallet_bridge.js",
	platform: "browser",
	target: "es2022",
});
