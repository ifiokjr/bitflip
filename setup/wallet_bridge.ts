import { SOLANA_CHAINS, type SolanaChain } from "@solana/wallet-standard-chains";
import {
	SolanaSignAndSendTransaction,
	type SolanaSignAndSendTransactionFeature,
	SolanaSignMessage,
	type SolanaSignMessageFeature,
} from "@solana/wallet-standard-features";
import { getWallets } from "@wallet-standard/app";
import type { Wallet, WalletAccount, WalletWithFeatures } from "@wallet-standard/base";
import {
	StandardConnect,
	type StandardConnectFeature,
	StandardEvents,
	type StandardEventsChangeProperties,
	type StandardEventsFeature,
} from "@wallet-standard/features";

const maxWireTransactionBytes = 2_048;
const maxSignedMessageBytes = 4_096;
const maxWalletOptions = 64;
const ed25519SignatureBytes = 64;
const base58Alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

type RequiredFeatures =
	& StandardConnectFeature
	& SolanaSignAndSendTransactionFeature
	& SolanaSignMessageFeature;
type CompatibleWallet = WalletWithFeatures<
	RequiredFeatures & Partial<StandardEventsFeature>
>;

interface WalletOption {
	readonly id: string;
	readonly name: string;
}

interface ActiveSession {
	readonly wallet: CompatibleWallet;
	account: WalletAccount;
	readonly chain: SolanaChain;
}

interface BitflipWalletBridge {
	readonly isSupported: true;
	address(): string | null;
	listWallets(chain: string): string;
	connect(walletId: string, chain: string): Promise<string>;
	signAndSend(wireTransactionBase64: string): Promise<string>;
	signMessage(message: string): Promise<string>;
}

declare global {
	var bitflipWallet: BitflipWalletBridge | undefined;
}

const registry = getWallets();
const walletIds = new WeakMap<Wallet, string>();
let nextWalletId = 1;
let activeSession: ActiveSession | null = null;
let unsubscribeFromWallet: (() => void) | null = null;

function parseChain(value: string): SolanaChain {
	if (!SOLANA_CHAINS.some((chain) => chain === value)) {
		throw new Error(`Unsupported Solana chain: ${value}`);
	}
	return value as SolanaChain;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
	return typeof value === "object" && value !== null;
}

function isCompatible(wallet: Wallet, chain: SolanaChain): wallet is CompatibleWallet {
	if (!wallet.chains.includes(chain)) return false;

	const connect = wallet.features[StandardConnect];
	const signAndSend = wallet.features[SolanaSignAndSendTransaction];
	const signMessage = wallet.features[SolanaSignMessage];
	return (
		isRecord(connect)
		&& connect.version === "1.0.0"
		&& typeof connect.connect === "function"
		&& isRecord(signAndSend)
		&& signAndSend.version === "1.0.0"
		&& typeof signAndSend.signAndSendTransaction === "function"
		&& Array.isArray(signAndSend.supportedTransactionVersions)
		&& signAndSend.supportedTransactionVersions.includes(0)
		&& isRecord(signMessage)
		&& (signMessage.version === "1.0.0" || signMessage.version === "1.1.0")
		&& typeof signMessage.signMessage === "function"
	);
}

function decodeBase58Address(value: string): Uint8Array | null {
	if (value.length < 32 || value.length > 44) return null;

	const bytes = [0];
	for (const character of value) {
		const alphabetIndex = base58Alphabet.indexOf(character);
		if (alphabetIndex < 0) return null;

		let carry = alphabetIndex;
		for (let index = 0; index < bytes.length; index += 1) {
			const byte = bytes[index];
			if (byte === undefined) return null;
			carry += byte * 58;
			bytes[index] = carry & 0xff;
			carry >>= 8;
		}
		while (carry > 0) {
			bytes.push(carry & 0xff);
			carry >>= 8;
		}
	}

	for (let index = 0; index < value.length - 1 && value[index] === "1"; index += 1) {
		bytes.push(0);
	}
	if (bytes.length !== 32) return null;
	return Uint8Array.from(bytes.reverse());
}

function accountSupports(account: WalletAccount, chain: SolanaChain): boolean {
	const decodedAddress = decodeBase58Address(account.address);
	return (
		account.publicKey.byteLength === 32
		&& decodedAddress !== null
		&& constantTimeEqual(decodedAddress, Uint8Array.from(account.publicKey))
		&& account.chains.includes(chain)
		&& account.features.includes(SolanaSignAndSendTransaction)
		&& account.features.includes(SolanaSignMessage)
	);
}

function compatibleWallets(chain: SolanaChain): readonly CompatibleWallet[] {
	return registry
		.get()
		.filter((wallet) => isCompatible(wallet, chain))
		.slice(0, maxWalletOptions);
}

function walletId(wallet: Wallet): string {
	const existing = walletIds.get(wallet);
	if (existing !== undefined) return existing;
	const id = `wallet-${nextWalletId}`;
	nextWalletId += 1;
	walletIds.set(wallet, id);
	return id;
}

function listWallets(chainValue: string): string {
	const chain = parseChain(chainValue);
	const options: readonly WalletOption[] = compatibleWallets(chain).map(
		(wallet) => ({ id: walletId(wallet), name: safeWalletName(wallet.name) }),
	);
	return JSON.stringify(options);
}

function safeWalletName(value: string): string {
	const cleaned = value.replace(/[\u0000-\u001f\u007f]/g, "").trim().slice(0, 80);
	return cleaned.length === 0 ? "Solana wallet" : cleaned;
}

function requireActiveSession(): ActiveSession {
	const session = activeSession;
	if (
		session === null
		|| !isCompatible(session.wallet, session.chain)
		|| !accountSupports(session.account, session.chain)
	) {
		throw new Error("Connect a compatible wallet before signing.");
	}
	return session;
}

function watchActiveAccount(session: ActiveSession): void {
	unsubscribeFromWallet?.();
	unsubscribeFromWallet = null;

	const feature = session.wallet.features[StandardEvents];
	if (!isRecord(feature) || typeof feature.on !== "function") return;
	const events = feature as StandardEventsFeature[typeof StandardEvents];
	unsubscribeFromWallet = events.on(
		"change",
		(properties: StandardEventsChangeProperties) => {
			if (properties.accounts === undefined || activeSession !== session) return;
			const current = properties.accounts.find(
				(account) =>
					account.address === session.account.address
					&& accountSupports(account, session.chain),
			);
			if (current === undefined) {
				activeSession = null;
				unsubscribeFromWallet?.();
				unsubscribeFromWallet = null;
				return;
			}
			session.account = current;
		},
	);
}

async function connect(walletIdValue: string, chainValue: string): Promise<string> {
	const chain = parseChain(chainValue);
	const wallet = compatibleWallets(chain).find(
		(candidate) => walletId(candidate) === walletIdValue,
	);
	if (wallet === undefined) {
		throw new Error("The selected wallet is no longer available.");
	}

	const output = await wallet.features[StandardConnect].connect();
	const account = output.accounts.find((candidate) => accountSupports(candidate, chain));
	if (account === undefined) {
		throw new Error(
			"The wallet did not authorize an account with the required Solana features.",
		);
	}

	const session: ActiveSession = { wallet, account, chain };
	activeSession = session;
	watchActiveAccount(session);
	return account.address;
}

function address(): string | null {
	return activeSession?.account.address ?? null;
}

function decodeBase64(value: string): Uint8Array {
	if (
		value.length === 0
		|| value.length % 4 !== 0
		|| !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(
			value,
		)
	) {
		throw new Error("The transaction is not valid base64.");
	}
	const decoded = atob(value);
	if (decoded.length === 0 || decoded.length > maxWireTransactionBytes) {
		throw new Error("The serialized transaction has an invalid size.");
	}
	return Uint8Array.from(decoded, (character) => character.charCodeAt(0));
}

function encodeBase64(value: Uint8Array): string {
	let binary = "";
	for (const byte of value) binary += String.fromCharCode(byte);
	return btoa(binary);
}

function encodeBase58(value: Uint8Array): string {
	const digits = [0];
	for (const byte of value) {
		let carry = byte;
		for (let index = 0; index < digits.length; index += 1) {
			const digit = digits[index];
			if (digit === undefined) throw new Error("Invalid base58 encoder state.");
			carry += digit << 8;
			digits[index] = carry % 58;
			carry = Math.floor(carry / 58);
		}
		while (carry > 0) {
			digits.push(carry % 58);
			carry = Math.floor(carry / 58);
		}
	}
	for (let index = 0; index < value.length - 1 && value[index] === 0; index += 1) {
		digits.push(0);
	}
	return digits
		.reverse()
		.map((digit) => base58Alphabet[digit])
		.join("");
}

function constantTimeEqual(left: Uint8Array, right: Uint8Array): boolean {
	if (left.byteLength !== right.byteLength) return false;
	let difference = 0;
	for (let index = 0; index < left.byteLength; index += 1) {
		difference |= (left[index] ?? 0) ^ (right[index] ?? 0);
	}
	return difference === 0;
}

async function signAndSend(wireTransactionBase64: string): Promise<string> {
	const session = requireActiveSession();
	const transaction = decodeBase64(wireTransactionBase64);
	const outputs = await session.wallet.features[
		SolanaSignAndSendTransaction
	].signAndSendTransaction({
		account: session.account,
		chain: session.chain,
		transaction,
		options: { preflightCommitment: "confirmed" },
	});
	if (outputs.length !== 1 || outputs[0]?.signature.byteLength !== ed25519SignatureBytes) {
		throw new Error("The wallet returned an invalid transaction signature.");
	}
	return encodeBase58(outputs[0].signature);
}

async function signMessage(message: string): Promise<string> {
	const session = requireActiveSession();
	const messageBytes = new TextEncoder().encode(message);
	if (messageBytes.length === 0 || messageBytes.length > maxSignedMessageBytes) {
		throw new Error("The message has an invalid size.");
	}
	const outputs = await session.wallet.features[SolanaSignMessage].signMessage({
		account: session.account,
		message: messageBytes,
	});
	const output = outputs[0];
	if (
		outputs.length !== 1
		|| output === undefined
		|| output.signature.byteLength !== ed25519SignatureBytes
		|| (output.signatureType !== undefined && output.signatureType !== "ed25519")
		|| !constantTimeEqual(output.signedMessage, messageBytes)
	) {
		throw new Error("The wallet returned an invalid message signature.");
	}
	return encodeBase64(output.signature);
}

globalThis.bitflipWallet = Object.freeze({
	isSupported: true,
	address,
	listWallets,
	connect,
	signAndSend,
	signMessage,
});

export { constantTimeEqual, decodeBase64, encodeBase58, isCompatible };
