# Native store release packet

Native releases are separate from the web beta. Do not submit a store build until every item below has an owner and evidence in the release record.

## Public URLs

- Privacy policy: `https://bitflip.xyz/privacy`
- Terms: `https://bitflip.xyz/terms`
- Support: `https://bitflip.xyz/support`
- Marketing site: `https://bitflip.xyz/`

The URLs must be live without authentication, render outside the app, and match the approved in-app copy before submission.

## Disclosures

- The app uses a public Solana wallet address and public on-chain activity.
- Mobile Bitflip creates a device-bound private key stored using Android Keystore-backed encryption or iOS Keychain. It is never sent to the Bitflip service. Web keys remain in the selected third-party wallet.
- Claims and flips pay network/program fees; the exact amount is shown before signing.
- Flips are public and confirmed transactions cannot be reversed.
- Sealing is permanent and enables compressed-NFT minting.
- Android and iOS sign with the embedded spending wallet; Android can fund it through MWA. Web uses third-party Wallet Standard wallets. macOS remains view-only.
- Pixels are stored on chain; HTTPS metadata availability is an operated service and must not be described as immutable.
- The product is experimental software, not an investment or promise of token value.

Privacy labels and age/content ratings must be completed from observed production behavior, SDK inventories, and counsel-approved wording—not copied from this checklist.

## Signing and distribution

### Android

- Provision an upload keystore outside the repository and back it up in two access-controlled locations.
- Supply the four `ANDROID_KEY*` environment variables documented in release readiness.
- Build the AAB from the reviewed commit, verify its certificate fingerprint, upload to internal testing, and repeat the physical-device MWA matrix from the delivered track.
- Enable Play App Signing and record the upload and app-signing certificate fingerprints.

### iOS

- Provision the App Store Connect API key, distribution certificate, bundle ID, and provisioning profile in the protected release environment.
- Archive and export the exact reviewed commit. Confirm Keychain restoration, third-party funding, embedded transaction/message signing, and the device-loss warning on a physical iPhone.
- Upload to TestFlight, complete export-compliance/privacy answers, and test on a physical iPhone before review submission.

### macOS

- Sign with Developer ID/Application Distribution as appropriate, enable the hardened runtime, notarize, staple, and validate with `spctl`.
- Verify outbound networking in the signed sandboxed artifact and confirm transaction controls remain view-only.

CI intentionally performs unsigned Apple compilation. Store credentials cannot be generated or validated in the repository; provisioning them is an external release gate.

## Metadata and review evidence

- name, subtitle, description, keywords, category, copyright, and release notes;
- current screenshots for each required device class, captured from the submitted build;
- embedded-wallet creation/funding, web-wallet connection, macOS view-only behavior, irreversible action confirmation, and support screenshots;
- accessibility statement and keyboard/screen-reader test evidence;
- reviewer notes explaining wallet requirements and a devnet test path;
- support escalation owner and response target.
