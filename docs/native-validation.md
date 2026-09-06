# Native validation plan

Embedded-wallet signing and funding are release-critical native code paths. Android and iOS sign with a device-bound spending wallet; Android additionally uses Mobile Wallet Adapter to fund it. macOS remains view-only but still requires signed-artifact, networking, layout, accessibility, and store validation.

## Required device matrix

- Android 11, Android 13, and the current Android release;
- Solana Seeker plus one non-Solana Android phone;
- at least two compatible Mobile Wallet Adapter wallets;
- a current physical iPhone for Keychain-backed signing and a supported macOS machine for view-only behavior;
- compact phone, large phone, and desktop window sizes;
- system text at 100%, 160%, and the platform maximum used by the accessibility test plan.

For each signing-capable combination, exercise first creation, restoration after process death/reboot, address copying, third-party funding, insufficient funds, claim, batched flip, confirmation, permanent seal, mint authorization, timeout recovery, and explorer links. On Android, also exercise MWA discovery, authorization, cancellation, rejection, exact transfer amount, wrong-network handling, and confirmation. Verify that uninstall/device-loss warnings are visible before funding. Never clear wallet-owned data simply to manufacture a first-use state; use a separate test profile or wallet.

## Performance budgets

Profile a release build, never debug mode. Capture Flutter DevTools traces and platform metrics for at least 60 seconds of canvas pan/zoom, coordinate selection, section changes, and live refresh.

- cold launch to first useful frame: p90 below 2 seconds;
- missed-frame/jank rate during interaction: below 1%;
- Flutter build and raster p99: within the device refresh-rate frame budget;
- no continuously growing memory after 20 section-change and zoom cycles;
- no network request or polling burst when the app is backgrounded;
- canvas input remains responsive while a refresh request is slow or failing.

The 2026-09-06 Seeker embedded-wallet staging smoke recorded a 545 ms Android cold activity relaunch. Flutter renders through a surface, so this activity timing is preliminary evidence only—not acceptance of the interaction budgets above.

## Deliberate dark appearance

Bitflip intentionally forces `ThemeMode.dark` rather than following the system theme. Validate every native build in bright daylight and a dark room, with platform high-contrast/increased-contrast settings where available. Text and disabled controls must remain distinguishable without relying on color alone. Store screenshots must reflect the same dark appearance; a light theme is not promised by this release.

## Evidence

Attach the exact commit, signed artifact hash, device/OS, wallet/version, screen recording, trace export, result, and defect links to the release record. Automated widget and integration tests are supporting evidence, not a substitute for the physical matrix.

The 2026-09-06 Seeker smoke used the staging/devnet release APK with SHA-256 `bc3bdf672075ade1e71ca7f58423e5a65870a9dafd6f7ecb278d94f60f24419a`. It verified automatic embedded-wallet creation, the full address and zero balance, device-loss disclosure, address restoration after a forced process stop and cold relaunch, and a successful handoff to the Seeker Mobile Wallet Adapter authorization sheet. The sheet was cancelled before authorization, so no funding or gameplay transaction was submitted. The APK was explicitly debug-signed for staging and is not a production-signing result.
