# Bitflip Flutter app

The responsive Bitflip client targets Android, iOS, macOS, and Flutter web from one codebase. It reads Pina accounts directly from Solana, signs mobile transactions with Mobile Wallet Adapter, signs browser transactions through Wallet Standard, and calls the generated Serverpod client for compressed-NFT mint authorization.

Run it from the repository root through the pinned devenv toolchain:

```bash
devenv shell
flutter:app run
```

For the complete local stack, use `server:start`; Serverpod launches this app automatically. See the repository [readme](../readme.md) for configuration, testing, and deployment commands.
