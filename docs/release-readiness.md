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

- Android Mobile Wallet Adapter discovery, connect, cancellation, claim, flip, seal, and message signing are exercised on physical devices with at least two wallets and Android 11, 13, and the current Android release.
- The Android app bundle is signed with the release key and installed from an internal-testing track.
- iOS and macOS remain explicitly view-only until a supported wallet handoff is implemented and tested. Their disabled transaction controls are a product contract, not a temporary error path.
- iOS and macOS builds are signed/notarized using store credentials. CI only proves that unsigned release compilation succeeds until those credentials are provisioned.
- Privacy policy, support URL, store disclosures, screenshots, and irreversible fee/sealing terms have been approved.
- The complete [native store release packet](store-release.md) has evidence attached to the release record.
- The [native validation plan](native-validation.md), including release-mode performance budgets and deliberate dark appearance, passes on the target matrix.

### Physical-device evidence

| Device        | OS                  | Artifact                                                    | MWA discovery | Cancellation                                                                | Authorization | Unavailable-state guard                                                          |
| ------------- | ------------------- | ----------------------------------------------------------- | ------------- | --------------------------------------------------------------------------- | ------------- | -------------------------------------------------------------------------------- |
| Solana Seeker | Android 16 / API 36 | Release APK, staging configuration, explicitly debug-signed | Pass          | Returns safely to Bitflip; the wallet may restore an existing authorization | Pass          | Pass: claim remains disabled after authorization when chain state is unavailable |

This smoke test does not replace the native-beta matrix above. In particular, no transaction was submitted, the installed artifact was not signed with the production key, and first-time rejection remains covered by automated tests until it can be exercised without deleting wallet-owned key material.

## 3. Mainnet

- Complete every step in [mainnet-ceremony.md](mainnet-ceremony.md).
- Obtain an independent review of the Pina program and mint operator. Findings must be fixed or explicitly accepted by the named release owner.
- Use newly generated production keys. Never reuse repository or development key material.
- Run a canary claim, flip, seal, and compressed mint with bounded funds before opening the application publicly.
- Replace the beta's user-driven mint reconciliation with the durable worker described in the [operations runbook](operations/runbook.md).
- Record the final marketplace-authenticity choice from [ADR 0002](decisions/0002-marketplace-authenticity.md).

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
