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

The claimant is the payer for the new sector PDA and, when BIT rewards are enabled, its lazily created Token-2022 vault. Their launch flow therefore pays:

- Solana rent for the 854-byte bitmap, economy, and policy account;
- current rent for one section-PDA-owned Token-2022 associated account; and
- `claim_price_lamports`, transferred to the configured treasury.

Creation, payment, ownership assignment, and advancing `next_section` are one claim instruction. Vault funding remains a permissionless, one-time instruction; the first-party client submits claim and funding together in one atomic transaction, so a claimant never receives an active but unfunded section. Direct integrations must do the same or keep reward-bearing controls disabled until funding confirms. A failed transaction creates no bitmap or token account, transfers no claim payment or BIT, and does not advance the game. At most one bitmap and one token account are created per active section, so an unused game never allocates the other 255 pairs.

## Play and ownership

Flipping is public while a sector is active. A player can toggle up to 16 unique pixels per transaction. The section controller calculates one posted SOL price for its current five-minute window, and each successfully rewarded pixel transfers one whole zero-decimal BIT from that section's vault. The player signs the window identifier, maximum unit and total charge, and minimum reward. Any mismatch rolls back payment, owner fee accrual, BIT, controller state, and all pixel toggles together. User-owned sections accrue the game's fixed 20% staging share in their own account; section `0` remains protocol-owned and routes 100% to the protocol. The current owner can withdraw accrued fees, a sale settles the seller automatically, and sealing settles before minting. Ownership also controls the scarce lifecycle actions: listing, cancelling a listing, and sealing.

The section now stores its immutable issuance-controller state and separately accounts for base BIT emitted and BIT moved into the protocol reward pool. Anyone can settle elapsed windows, but no instruction can yet distribute BIT or withdraw the pool. Owners can publish a versioned, time-bounded open or eight-colour policy and rules digest; policy terms lock while live, survive a sale, and are signed by every player. Entry charges and rewards remain disabled until owner/sponsor deposits have physically backed custody and refund rules. The economics, custody model, fee-share hypothesis, and implementation gates are specified in [ADR 0003](decisions/0003-section-economy.md).

## Resale

An owner can place a fixed SOL price directly in an active or sealed sector account. This creates no listing account and requires no extra rent. They can replace the price or cancel it at any time.

A purchase transfers the listed lamports from buyer to seller, changes the sector owner, and clears the listing in one transaction. The buyer signs a maximum price, so a front-end or stale RPC response cannot make them pay more than they approved. The seller loses owner authority immediately.

Minted sectors cannot use this listing mechanism. After minting, the compressed NFT is the transferable asset; allowing the bitmap receipt and NFT to be sold independently would create two conflicting owners. A future post-mint sector marketplace would need an atomic Bubblegum transfer and proof flow rather than a simple owner-field update.

## Deployment compatibility

Economy ABI version 6 uses a 237-byte config, 123-byte game, and 854-byte section. It includes the immutable BIT mint/reserve registry, a canonical vault for every funded section, atomic paid issuance, fixed owner fee sharing, shard-local protocol and owner fee ledgers, owner-only withdrawal, and versioned section policies, and rejects initialization beyond the four configured games. Do not point the updated app at accounts from an older layout. For staging and launch, deploy the reviewed program under a fresh program ID and initialize fresh games so every section uses the current layout, controller configuration, custody rules, and policy-version protection.
