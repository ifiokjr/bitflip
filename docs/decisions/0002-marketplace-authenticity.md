# ADR 0002: Marketplace authenticity

- Status: accepted
- Date: 2026-09-06
- Owner: @ifiokjr

## Decision

Bitflip's canonical authenticity signal is the mint receipt stored by the Bitflip Solana program: game, section, asset ID, Merkle tree, and leaf index are recorded atomically with the Bubblegum mint. The beta will not claim membership in a verified marketplace collection. Its Bubblegum metadata has only an unverified operator creator.

UI, store copy, and support responses must direct users to the Bitflip program receipt and must not use “verified collection,” “official collection,” or equivalent marketplace language.

## Mainnet gate

Before mainnet, the release owner must make and record one of two explicit choices:

1. keep program receipts as the sole authenticity mechanism and obtain a product/security review of how users and marketplaces can verify them; or
2. create a sized Metaplex collection, change minting to the collection-verifying Bubblegum instruction, validate collection authority during startup, and expand the real-SBF/cNFT suite.

Silently presenting an unverified creator as marketplace verification is not an option.
