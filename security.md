# Security

Please report vulnerabilities privately to `security@ifiokjr.com`. Do not open a public issue for an exploitable finding.

Before a mainnet launch:

- deploy the verified program artifact and record its immutable hash;
- use a dedicated collection-authority key held in managed secret storage;
- create a private Bubblegum V1 tree delegated only to that authority;
- put Serverpod behind TLS and restrict database and Insights access;
- preserve the Serverpod web CSP, WASM isolation, clickjacking, MIME-sniffing, permissions, and referrer headers at the edge proxy;
- configure RPC rate limits, alerts, backups, and operator-key rotation;
- run `lint:all`, `test:all`, `cargo audit`, `cargo deny check`, and `gitleaks`;
- commission an independent review of both the program and mint operator.

The backend intentionally holds a PostgreSQL advisory transaction lock while a compressed mint is submitted. This serializes leaf allocation across replicas. Because the tree is private, only the same operator can increment it. If a process dies after chain confirmation but before the database commit, retrying is safe: the on-chain Bitflip receipt is authoritative and the mint service returns the already-recorded asset.

The browser wallet boundary uses the official Wallet Standard registry rather than wallet-specific injected globals. Wallets are filtered by configured Solana chain, versioned-transaction support, and message-signing support. Mint authorization accepts only the exact challenge bytes and a canonical 64-byte Ed25519 signature. The mobile app applies the same exact-message check to Mobile Wallet Adapter responses.

Never reuse any key previously committed to this repository. The removed legacy `setup/keypairs/admin.json` secret and its public key must be treated as permanently compromised. Production bootstrap, collection-authority, compression-tree delegate, and program deployment keys must be newly generated and held outside git in managed secret storage.
