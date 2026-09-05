# Release readiness

Bitflip releases in three deliberately separate milestones. Passing one does not imply the later milestones are safe.

## 1. Web devnet beta

- All CI checks pass, including the assembled-app journey.
- A staging deployment uses explicit `staging`, devnet RPC, Serverpod URL, and game index values.
- Health and readiness probes pass from outside the hosting network.
- RPC failures, wallet cancellation, insufficient funds, and stale state are exercised against staging.
- Database backup and restore are tested.

## 2. Native beta

- Android Mobile Wallet Adapter discovery, connect, cancellation, claim, flip, seal, and message signing are exercised on physical devices with at least two wallets and Android 11, 13, and the current Android release.
- The Android app bundle is signed with the release key and installed from an internal-testing track.
- iOS and macOS remain explicitly view-only until a supported wallet handoff is implemented and tested. Their disabled transaction controls are a product contract, not a temporary error path.
- iOS and macOS builds are signed/notarized using store credentials. CI only proves that unsigned release compilation succeeds until those credentials are provisioned.
- Privacy policy, support URL, store disclosures, screenshots, and irreversible fee/sealing terms have been approved.

## 3. Mainnet

- Complete every step in [mainnet-ceremony.md](mainnet-ceremony.md).
- Obtain an independent review of the Pina program and mint operator. Findings must be fixed or explicitly accepted by the named release owner.
- Use newly generated production keys. Never reuse repository or development key material.
- Run a canary claim, flip, seal, and compressed mint with bounded funds before opening the application publicly.

## Required application configuration

Every release build must set all five compile-time values:

```text
BITFLIP_ENVIRONMENT=development|staging|production
SOLANA_WALLET_CHAIN=solana:devnet|solana:testnet|solana:mainnet
SOLANA_RPC_URL=https://...
SERVERPOD_URL=https://.../
BITFLIP_GAME_INDEX=0..255
```

Use `build:web`, `build:android:release`, `build:ios:release`, or `build:macos:release`. These commands refuse to start without every value. Production additionally requires mainnet and public HTTPS endpoints. The app performs the same validation at startup so bypassing the wrapper cannot create a silently misconfigured release.

The production server must set:

```text
BITFLIP_CLUSTER=mainnet
BITFLIP_GAME_INDEX=0..255
SOLANA_RPC_URL=https://...
BITFLIP_METADATA_BASE_URL=https://...
BITFLIP_MERKLE_TREE=<private Bubblegum tree address>
BITFLIP_OPERATOR_PRIVATE_KEY=<secret JSON byte array>
BITFLIP_PRIORITY_FEE_MICROLAMPORTS=<explicit non-negative integer>
```

Optional bounded reliability settings are `BITFLIP_RPC_TIMEOUT_SECONDS` (2–60, default 12) and `BITFLIP_MINT_MAX_ATTEMPTS` (1–5, default 3). Serverpod database passwords and service secrets remain required by Serverpod. The server validates the tree address, signer encoding, and operator fee policy before it starts.

The public challenge endpoint is rate limited per source and globally within each process before any Solana RPC work begins. Production ingress must enforce the same policy across replicas; abandoned challenges never consume a wallet-specific quota. Mint work is rejected when operator capacity is full, runs outside database transactions, uses explicit RPC deadlines and priority fees, and retries from fresh on-chain state.

Serverpod exposes `/livez`, `/readyz`, and `/startupz` on the API service. Production deployment probes all three and alerts on readiness failures. The production configuration disables Serverpod Insights; logs and metrics are exported through the private observability pipeline instead.

## Metadata permanence

The NFT points to HTTPS metadata rendered deterministically from the sealed on-chain bitmap. The pixels are permanent; the HTTP metadata service is not. Until metadata is pinned to content-addressed storage, release copy must not claim that the NFT metadata itself is immutable. Production operation therefore commits to retaining the metadata and art routes for the lifetime of the collection, including redirects during any domain move.
