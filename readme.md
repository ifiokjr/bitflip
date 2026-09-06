# Bitflip

Bitflip is a one-million-pixel, collaborative Solana canvas. Players claim a 64 × 64 sector, flip up to 16 pixels in one atomic move, permanently seal their artwork, and mint the sealed sector as a Bubblegum V1 compressed NFT.

The product is one responsive Flutter codebase for Android, iOS, macOS, and the web. Serverpod hosts the website, provides the typed mint API, publishes NFT metadata and deterministic SVG art, and keeps the private compression-tree operator behind a wallet-signed authorization boundary.

## Architecture

- `bitflip_program` — `no_std` Solana program built on Pina 0.12.2.
- `bitflip_program/clients/dart` — generated Pina/Codama Dart client.
- `bitflip_app` — responsive Flutter app and website with Mobile Wallet Adapter and browser Wallet Standard signing.
- `bitflip_server` — Serverpod 4 backend and generated typed client.
- `bitflip_program/tests/surfpool` — real-SBF program integration tests running in isolated Surfpool nodes.
- `bitflip_server/bitflip_server_server/test/surfpool` — full Bubblegum cNFT integration against checksum-pinned program artifacts.

The canvas is 1024 × 1024 pixels split into 256 sectors. Each sector coordinate is two `u8` values (`x`, `y`), avoiding ambiguous packed integers. A raw bitmap uses 512 bytes per sector and is easy to validate, render, and hash.

## Development

All commands run inside the reproducible `devenv` shell. `install:all` installs the exact Serverpod CLI into the gitignored workspace tool cache and runs it with the Dart SDK bundled by the pinned Flutter release, avoiding host SDK drift:

```bash
devenv shell
fvm install 3.47.2
install:all
```

Start the complete development stack—embedded PostgreSQL, migrations, Serverpod hot reload, API on 8080, and the auto-launched Flutter web app—through devenv:

```bash
server:start
```

To target a mobile device instead, disable Serverpod's web auto-launch and run Flutter in another devenv shell:

```bash
server:start --no-flutter
flutter:app run
```

Build the Flutter WASM website directly into Serverpod's web root. Release commands require explicit configuration; see [release readiness](docs/release-readiness.md) for the complete contract.

The [audit remediation ledger](docs/audit-remediation.md) distinguishes repository controls from deployment, store, and mainnet evidence that still has to be produced outside CI.

```bash
build:web
server:start
```

The Flutter app defaults to devnet signing/RPC and local Serverpod. The Serverpod development environment defaults to local Surfpool. Keep the wallet chain and RPC cluster aligned when running on a device or building a deployment:

```bash
flutter:app run \
  --dart-define=SOLANA_WALLET_CHAIN=solana:devnet \
  --dart-define=SOLANA_RPC_URL=https://api.devnet.solana.com \
  --dart-define=SERVERPOD_URL=https://api-staging.bitflip.xyz/
```

The web build bundles the official Wallet Standard registry and Solana extensions locally. It only offers wallets that support the selected chain, versioned transactions, and exact-message signing; mobile uses Solana Mobile Wallet Adapter.

## Generation and migrations

```bash
generate:clients  # Pina account/instruction client
generate:server   # Serverpod protocol and app client
migration:create  # after changing a .spy.yaml database model
```

Generated Pina and Serverpod source should never be edited by hand.

## Verification

```bash
audit:security
lint:all
test:rust
test:surfpool
test:surfpool:cnft
test:server
test:wallet
test:operations
test:flutter
test:flutter:integration
build:server
build:web
build:container
```

`test:surfpool` builds the actual SBF program before testing. `test:surfpool:cnft` deploys Bitflip, Bubblegum, Account Compression, and Noop to an offline Surfnet, then proves that minting and recording the Pina receipt are atomic and idempotent. The external binaries are fetched from a pinned commit and SHA-256 verified. Serverpod tests use its managed embedded PostgreSQL and do not require Docker. `test:wallet` checks chain/feature filtering and rejects wallets that return altered signed-message bytes.

## Compressed-NFT minting

Configure these as deployment secrets; never commit them:

- `BITFLIP_MERKLE_TREE` — a private Bubblegum V1 tree.
- `BITFLIP_OPERATOR_PRIVATE_KEY` — JSON bytes for the tree delegate and the on-chain Bitflip collection authority.
- `BITFLIP_METADATA_BASE_URL` — the public Serverpod web origin.
- `SOLANA_RPC_URL` — the cluster RPC endpoint.
- `SERVERPOD_DATABASE_PASSWORD` — the managed PostgreSQL password.
- `SERVERPOD_SERVICE_SECRET` — a unique, high-entropy Serverpod service secret.

The owner requests a five-minute challenge, signs its exact text with their wallet, and submits the signature to Serverpod. The backend re-reads the Pina accounts, verifies ownership and sealed state, rate-limits and consumes the challenge, then sends Bubblegum mint and Bitflip receipt instructions in one transaction. A private tree prevents outside mints from racing the predicted leaf index. The beta must run one mint-capable server process because its operator gate is process-local; durable single-consumer work is a mainnet gate.

## Security properties

- No reward faucet or no-op payout exists.
- Duplicate pixel coordinates are rejected on-chain.
- Claim and flip prices include explicit client slippage ceilings.
- Claims unlock deterministically and in sequence.
- Only the configured authority can start the next game and its unlock clock.
- Only a sector owner can seal; sealing is irreversible.
- Only the attested collection authority can record a mint.
- Authority changes use a two-step propose/accept flow.
- Mint authorizations bind wallet, game, sector, nonce, expiry, and action and are single-use.
- Compressed mint and on-chain asset receipt succeed or fail atomically.

See [security.md](./security.md) for reporting and operational requirements.

## License

See [LICENSE](./LICENSE).
