# Sector lifecycle and cost model

Bitflip grows the canvas lazily. The protocol pays for one public starting sector; players fund every later sector only when demand reaches it.

## Launch

`InitializeGame` creates two program accounts in one transaction:

1. the game account; and
2. sector `0`, an empty 64 × 64 bitmap whose logical owner is the game PDA.

The configured authority pays rent for those two accounts. The game starts with `next_section = 1`, so no other bitmap account exists yet. Anyone can pay to flip pixels in sector `0`, but no wallet can list or seal it because its owner is the program-derived game address. It remains the permanent public starting arena.

## Unlock and first purchase

Only `next_section` can be claimed. A later sector unlocks when either:

- the immediately preceding sector reaches `early_unlock_flips`; or
- its deterministic schedule reaches `starts_at + section_index ×
  unlock_interval_seconds`.

The defaults are 1,024 paid flips or one hour per sector. Both values are configuration, so the launch ceremony must record the intended policy.

The claimant is the payer for the new sector PDA. Their transaction therefore pays:

- Solana rent for the 651-byte bitmap account; and
- `claim_price_lamports`, transferred to the configured treasury.

Creation, payment, ownership assignment, and advancing `next_section` are one atomic instruction. A failed claim creates no account and transfers no funds. At most one new bitmap account is created for each successful claim, so an unused game never allocates the other 255 sectors.

## Play and ownership

Flipping is public while a sector is active. A player can toggle up to 16 unique pixels per transaction and pays the game fee for every flip. Ownership controls the scarce lifecycle actions: listing, cancelling a listing, and sealing.

## Resale

An owner can place a fixed SOL price directly in an active or sealed sector account. This creates no listing account and requires no extra rent. They can replace the price or cancel it at any time.

A purchase transfers the listed lamports from buyer to seller, changes the sector owner, and clears the listing in one transaction. The buyer signs a maximum price, so a front-end or stale RPC response cannot make them pay more than they approved. The seller loses owner authority immediately.

Minted sectors cannot use this listing mechanism. After minting, the compressed NFT is the transferable asset; allowing the bitmap receipt and NFT to be sold independently would create two conflicting owners. A future post-mint sector marketplace would need an atomic Bubblegum transfer and proof flow rather than a simple owner-field update.

## Deployment compatibility

This model changes the sector account from 643 to 651 bytes and changes the `InitializeGame` instruction accounts and data. Do not point the updated app at a game containing old-layout sectors. For staging and launch, deploy the reviewed program and initialize a fresh game so every sector uses the current layout.
