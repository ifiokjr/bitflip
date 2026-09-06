# Release readiness

Bitflip releases in three deliberately separate milestones. Passing one does not imply the later milestones are safe.

## 1. Web devnet beta

- All CI checks pass, including the assembled-app journey.
- A staging deployment uses explicit `staging`, devnet RPC, Serverpod URL, and game index values.
- Health and readiness probes pass from outside the hosting network.
- RPC failures, wallet cancellation, insufficient funds, and stale state are exercised against staging.
- Database backup and restore are tested.
- The [Serverpod release policy](decisions/0001-serverpod-4-release-policy.md) is satisfied for this milestone.

## 2. Native beta

- Embedded-wallet creation, restoration, balance display, claim, flip, seal, message signing, and device-loss disclosure are exercised on Android and iOS physical devices.
- Android Mobile Wallet Adapter discovery, funding transfer, cancellation, and confirmation are exercised with at least two wallets on Android 11, 13, and the current Android release.
- The Android app bundle is signed with the release key and installed from an internal-testing track.
- iOS signs through its embedded Keychain-backed wallet and accepts third-party funding through the displayed address. macOS remains explicitly view-only.
- iOS and macOS builds are signed/notarized using store credentials. CI only proves unsigned release compilation until those credentials are provisioned.
- Privacy policy, support URL, store disclosures, screenshots, and irreversible fee/sealing terms have been approved.
- The complete [native store release packet](store-release.md) has evidence attached to the release record.
- The [native validation plan](native-validation.md), including release-mode performance budgets and deliberate dark appearance, passes on the target matrix.

### Physical-device evidence

| Device        | OS                  | Artifact                                                    | Embedded creation/restoration | MWA discovery | MWA authorization/funding | Unavailable-state guard                                         |
| ------------- | ------------------- | ----------------------------------------------------------- | ----------------------------- | ------------- | ------------------------- | --------------------------------------------------------------- |
| Solana Seeker | Android 16 / API 36 | Release APK, staging configuration, explicitly debug-signed | Pass                          | Pass          | Not submitted             | Pass: gameplay remains disabled when chain state is unavailable |

The 2026-09-06 smoke installed SHA-256 `bc3bdf672075ade1e71ca7f58423e5a65870a9dafd6f7ecb278d94f60f24419a`, verified that the same embedded address returned after a forced process stop and cold relaunch, and reached the Seeker Mobile Wallet Adapter authorization sheet before cancelling. It does not replace the native-beta matrix above: no funding or gameplay transaction was submitted, the installed artifact was not signed with the production key, and first-time MWA rejection remains covered by automated tests until it can be exercised without deleting wallet-owned key material.

## 3. Mainnet

- Complete every step in [mainnet-ceremony.md](mainnet-ceremony.md).
- Obtain an independent review of the Pina program and mint operator. Findings must be fixed or explicitly accepted by the named release owner.
- Use newly generated production keys. Never reuse repository or development key material.
- Run a canary claim, flip, seal, and compressed mint with bounded funds before opening the application publicly.
- Confirm game initialization creates only sector `0`, its logical owner is the game PDA, and the authority paid rent for no later sector.
- Exercise activity and scheduled unlocks, claimant-funded sector creation, listing, cancellation, purchase slippage, and ownership transfer on staging.
- Replace the beta's user-driven mint reconciliation with the durable worker described in the [operations runbook](operations/runbook.md).
- Record the final marketplace-authenticity choice from [ADR 0002](decisions/0002-marketplace-authenticity.md).
- Do not enable BIT rewards or section campaigns until the token, custody, economic, integrity, and product-copy gates in [ADR 0003](decisions/0003-section-economy.md) are implemented and independently reviewed. A release without those features may proceed only if the UI continues to make their absence explicit.
- Before enabling paid BIT distribution, satisfy the simulation, property-test, real-SBF capacity, wallet-quote, and devnet evidence requirements in [ADR 0004](decisions/0004-section-price-controller.md).
- Remove the shared writable game counter and direct global-treasury transfer from `FlipPixels` before making any sharded-throughput claim; the [economics simulation report](economics-simulation.md) records the real-SBF contention finding.

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

Android production releases additionally require `ANDROID_KEYSTORE_PATH`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, and `ANDROID_KEY_PASSWORD`. Non-production `build:android:release` runs opt in to debug signing so the exact release artifact can be installed on physical devices; production can never take that fallback.

Staging and production servers must set:

```text
BITFLIP_CLUSTER=devnet|mainnet
BITFLIP_GAME_INDEX=0..255
SOLANA_RPC_URL=https://...
BITFLIP_METADATA_BASE_URL=https://...
BITFLIP_MERKLE_TREE=<private Bubblegum tree address>
BITFLIP_OPERATOR_PRIVATE_KEY=<secret JSON byte array>
BITFLIP_PRIORITY_FEE_MICROLAMPORTS=<explicit non-negative integer>
```

Optional bounded reliability settings are `BITFLIP_RPC_TIMEOUT_SECONDS` (2–60, default 12) and `BITFLIP_MINT_MAX_ATTEMPTS` (1–5, default 3). Serverpod database passwords and service secrets remain required by Serverpod. The server validates the tree address, signer encoding, and operator fee policy before it starts.

Staging requires `BITFLIP_CLUSTER=devnet`; production requires `BITFLIP_CLUSTER=mainnet`. Both require public HTTPS RPC and metadata origins plus explicit tree, operator, game, and priority-fee values. Local development retains loopback defaults.

The public challenge endpoint is rate limited per source and globally within each process before any Solana RPC work begins. Production ingress must enforce the same policy across replicas; abandoned challenges never consume a wallet-specific quota. Mint work is rejected when operator capacity is full, runs outside database transactions, uses explicit RPC deadlines and priority fees, and retries from fresh on-chain state. Until a durable single-consumer worker exists, a beta deployment must run exactly one mint-capable server process so two replicas cannot race private-tree leaf allocation.

Serverpod exposes `/livez`, `/readyz`, and `/startupz` on the API service. Production deployment probes all three and alerts on readiness failures. The production configuration disables Serverpod Insights; logs and metrics are exported through the private observability pipeline instead.

Deployment, migration rehearsal, staged rollout, rollback, and reconciliation are defined in the [operations runbook](operations/runbook.md). The exact required signals, budgets, and alert thresholds are defined in the [observability contract](operations/observability.md). Provider-specific deployment and telemetry configuration remains an external gate until the production hosting platform is selected.

## Metadata permanence

The NFT points to HTTPS metadata rendered deterministically from the sealed on-chain bitmap. The pixels are permanent; the HTTP metadata service is not. Until metadata is pinned to content-addressed storage, release copy must not claim that the NFT metadata itself is immutable. Production operation therefore commits to retaining the metadata and art routes for the lifetime of the collection, including redirects during any domain move.
