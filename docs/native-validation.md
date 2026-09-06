# Native validation plan

Android signing and wallet behavior are release-critical native code paths. Apple builds are view-only until a supported handoff exists, but they still require signed-artifact, networking, layout, accessibility, and store validation.

## Required device matrix

- Android 11, Android 13, and the current Android release;
- Solana Seeker plus one non-Solana Android phone;
- at least two compatible Mobile Wallet Adapter wallets;
- a current physical iPhone and a supported macOS machine for view-only behavior;
- compact phone, large phone, and desktop window sizes;
- system text at 100%, 160%, and the platform maximum used by the accessibility test plan.

For each signing-capable combination, exercise discovery, first authorization, returning authorization, cancellation, rejection, claim, batched flip, confirmation, permanent seal, mint authorization, timeout recovery, and explorer links. Never clear wallet-owned data simply to manufacture a first-use state; use a separate test profile or wallet.

## Performance budgets

Profile a release build, never debug mode. Capture Flutter DevTools traces and platform metrics for at least 60 seconds of canvas pan/zoom, coordinate selection, section changes, and live refresh.

- cold launch to first useful frame: p90 below 2 seconds;
- missed-frame/jank rate during interaction: below 1%;
- Flutter build and raster p99: within the device refresh-rate frame budget;
- no continuously growing memory after 20 section-change and zoom cycles;
- no network request or polling burst when the app is backgrounded;
- canvas input remains responsive while a refresh request is slow or failing.

The 2026-09-06 Seeker staging smoke recorded a 515 ms Android cold activity launch and zero janky Android framework frames in its tiny initial sample. Flutter renders through a surface, so that `gfxinfo` sample is preliminary evidence only—not acceptance of the interaction budgets above.

## Deliberate dark appearance

Bitflip intentionally forces `ThemeMode.dark` rather than following the system theme. Validate every native build in bright daylight and a dark room, with platform high-contrast/increased-contrast settings where available. Text and disabled controls must remain distinguishable without relying on color alone. Store screenshots must reflect the same dark appearance; a light theme is not promised by this release.

## Evidence

Attach the exact commit, signed artifact hash, device/OS, wallet/version, screen recording, trace export, result, and defect links to the release record. Automated widget and integration tests are supporting evidence, not a substitute for the physical matrix.
