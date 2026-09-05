import { SOLANA_DEVNET_CHAIN, SOLANA_MAINNET_CHAIN } from "@solana/wallet-standard-chains";
import { SolanaSignAndSendTransaction, SolanaSignMessage } from "@solana/wallet-standard-features";
import { getWallets } from "@wallet-standard/app";
import type { Wallet, WalletAccount } from "@wallet-standard/base";
import { StandardConnect } from "@wallet-standard/features";
import assert from "node:assert/strict";
import { test } from "node:test";

import { encodeBase58 } from "./wallet_bridge.ts";

const publicKey = new Uint8Array(32).fill(7);
const address = encodeBase58(publicKey);
const account: WalletAccount = Object.freeze({
	address,
	publicKey,
	chains: [SOLANA_DEVNET_CHAIN] as const,
	features: [SolanaSignAndSendTransaction, SolanaSignMessage] as const,
});

function mockWallet(
	name: string,
	alterSignedMessage = false,
	authorizedAccount: WalletAccount = account,
	connectVersion = "1.0.0",
): Wallet {
	const wallet: Wallet = {
		version: "1.0.0",
		name,
		icon: "data:image/png;base64,AA==",
		chains: [SOLANA_DEVNET_CHAIN] as const,
		accounts: [] as const,
		features: {
			[StandardConnect]: {
				version: connectVersion,
				connect: async () => ({ accounts: [authorizedAccount] }),
			},
			[SolanaSignAndSendTransaction]: {
				version: "1.0.0",
				supportedTransactionVersions: [0],
				signAndSendTransaction: async () => [
					{ signature: new Uint8Array(64).fill(1) },
				],
			},
			[SolanaSignMessage]: {
				version: "1.1.0",
				signMessage: async ({ message }: { readonly message: Uint8Array }) => [
					{
						signedMessage: alterSignedMessage
							? new Uint8Array(message.length).fill(9)
							: message,
						signature: new Uint8Array(64).fill(2),
						signatureType: "ed25519" as const,
					},
				],
			},
		},
	};
	return Object.freeze(wallet);
}

interface WalletOption {
	readonly id: string;
	readonly name: string;
}

function isWalletOption(value: unknown): value is WalletOption {
	return (
		typeof value === "object"
		&& value !== null
		&& "id" in value
		&& typeof value.id === "string"
		&& "name" in value
		&& typeof value.name === "string"
	);
}

function bridge() {
	const value = globalThis.bitflipWallet;
	assert.ok(value);
	return value;
}

function walletOption(walletName: string): WalletOption {
	const rawOptions = bridge().listWallets(SOLANA_DEVNET_CHAIN);
	const options: unknown = JSON.parse(rawOptions);
	if (!Array.isArray(options)) throw new Error("Expected wallet options.");
	const option = options.find(
		(value: unknown) => isWalletOption(value) && value.name === walletName,
	);
	if (!isWalletOption(option)) throw new Error(`Missing wallet: ${walletName}`);
	return option;
}

test("discovers only compatible wallets on the requested chain", () => {
	const unregisterCompatible = getWallets().register(mockWallet("Test Compatible"));
	const incompatible: Wallet = {
		...mockWallet("Wrong Chain"),
		chains: [SOLANA_MAINNET_CHAIN] as const,
	};
	const unregisterIncompatible = getWallets().register(incompatible);

	try {
		assert.equal(walletOption("Test Compatible").name, "Test Compatible");
		const options = bridge().listWallets(SOLANA_DEVNET_CHAIN);
		assert.equal(options.includes("Wrong Chain"), false);
	} finally {
		unregisterCompatible();
		unregisterIncompatible();
	}
});

test("connects, signs a transaction, and signs the exact challenge", async () => {
	const unregister = getWallets().register(mockWallet("Test Signer"));
	try {
		const option = walletOption("Test Signer");
		assert.equal(
			await bridge().connect(option.id, SOLANA_DEVNET_CHAIN),
			address,
		);
		assert.equal(bridge().address(), address);

		const transaction = btoa(String.fromCharCode(...new Uint8Array([1, 2, 3])));
		const signature = await bridge().signAndSend(transaction);
		assert.match(signature, /^[1-9A-HJ-NP-Za-km-z]{87,88}$/);

		await assert.rejects(bridge().signMessage(""), /invalid size/);
		await assert.rejects(
			bridge().signMessage("x".repeat(4_097)),
			/invalid size/,
		);
		const messageSignature = await bridge().signMessage(
			"bitflip challenge",
		);
		const decodedSignature = Uint8Array.from(
			atob(messageSignature),
			(character) => character.charCodeAt(0),
		);
		assert.deepEqual(
			decodedSignature,
			new Uint8Array(64).fill(2),
		);
	} finally {
		unregister();
	}
});

test("rejects a wallet that reports signing different message bytes", async () => {
	const unregister = getWallets().register(mockWallet("Altered Message", true));
	try {
		const option = walletOption("Altered Message");
		await bridge().connect(option.id, SOLANA_DEVNET_CHAIN);
		await assert.rejects(
			bridge().signMessage("canonical challenge"),
			/invalid message signature/,
		);
	} finally {
		unregister();
	}
});

test("rejects unknown feature contracts and inconsistent account identities", async () => {
	const unregisterVersion = getWallets().register(
		mockWallet("Future Contract", false, account, "9.0.0"),
	);
	const mismatchedAccount: WalletAccount = Object.freeze({
		...account,
		publicKey: new Uint8Array(32).fill(8),
	});
	const unregisterKey = getWallets().register(
		mockWallet("Key Mismatch", false, mismatchedAccount),
	);

	try {
		const options = bridge().listWallets(SOLANA_DEVNET_CHAIN);
		assert.equal(options.includes("Future Contract"), false);
		const option = walletOption("Key Mismatch");
		await assert.rejects(
			bridge().connect(option.id, SOLANA_DEVNET_CHAIN),
			/required Solana features/,
		);
	} finally {
		unregisterVersion();
		unregisterKey();
	}
});
