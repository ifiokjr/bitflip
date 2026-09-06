# Economics simulation and adversarial findings

This is reproducible engineering evidence for [ADR 0004](decisions/0004-section-price-controller.md), not a mainnet parameter recommendation. The controller is not connected to token custody or the `FlipPixels` account ABI yet.

## Run it

```sh
devenv shell -- simulate:economics
devenv shell -- test:rust
devenv shell -- test:surfpool
```

`simulate:economics` executes the same `no_std` integer controller exported by `bitflip_program`. The property suite generates 1,024 cases per property and covers arbitrary traffic delays and batches, price and inventory bounds, ordering, Sybil splitting, quote staleness, timestamp reversal, missed windows, final partial-window exhaustion, and representable `u64` extremes.

## Staging scenario output

The staging model uses a 1,600-BIT target, 3,200-BIT window cap, five-minute windows, 10,000-lamport starting price, 5,000-lamport minimum, 100,000-lamport ending inventory floor, and a 20% owner fee share. BIT has zero decimals: one rewarded pixel always means one whole BIT.

| Scenario                     | Rewarded BIT | Paid lamports | Owner recovery | Protocol revenue | Next price |
| ---------------------------- | -----------: | ------------: | -------------: | ---------------: | ---------: |
| 12 idle windows              |            0 |             0 |              0 |                0 |      5,000 |
| 12 windows at target         |       19,200 |   192,000,000 |     38,400,000 |      153,600,000 |     10,000 |
| 12 saturated windows         |       38,400 |   648,000,000 |    129,600,000 |      518,400,000 |     25,000 |
| 6 alternating burst/idle     |       19,200 |   192,000,000 |     38,400,000 |      153,600,000 |     10,000 |
| 59-day idle then 102,400 ask |        3,200 |    16,000,000 |      3,200,000 |       12,800,000 |      5,000 |

Amounts exclude Solana transaction fees and token-account rent. Owner recovery is money returned to the owner, so an owner using Sybil wallets to create artificial demand has a net SOL cost equal to the protocol-revenue column while acquiring the finite BIT payout.

## Attacks attempted

### Deadline catch-up drain: fixed

The original proposal recalculated each target from remaining inventory divided by remaining time. After a 59-day idle period, the final target approached the entire 26,214,400-BIT reserve while the price had reached its minimum. A bot could wait and drain the reserve during the cheapest window.

The target is now immutable. A final-window request for 102,400 BIT receives only the 3,200-BIT cap. Unissued BIT stays locked after expiry.

### Multiplicative burst/idle ratchet: fixed

The first EIP-1559-style implementation multiplied the current price by 1.125 after a saturated window and by 0.875 after an idle window. Those factors multiply to 0.984375, so traffic averaging exactly 100% utilisation reduced the price by roughly 1.56% every pair. Six pairs reduced 10,000 lamports to 9,095.

The controller now adds or subtracts a fixed utilisation-weighted step based on the launch price. Equal positive and negative pressure returns exactly to 10,000 lamports in the simulation.

### Sybil splitting: bounded

Two hundred 16-BIT batches and 3,200 one-BIT wallets produce identical state, rewards, and issuance charges. Wallet identity is not an input. An owner can still manufacture demand, but the proposed 20% owner share leaves 80% of every charge with the protocol and the attacker consumes finite inventory.

This does not prevent ordinary market arbitrage. If transferable BIT is worth more than the posted issuance price, buyers will rationally consume all available capacity. Devnet data and an independent economic review must set the minimum and starting prices.

That arbitrage is especially important because BIT is fungible while section prices differ: rational bots will route to the cheapest section. This can help equalise congestion, but the controller only limits the rate and size of extraction; it cannot guarantee that a SOL-denominated price is economically sufficient. Mainnet needs either conservative empirically selected floors or a separately reviewed oracle policy. Section owners must not be allowed to lower immutable price, target, duration, or inventory parameters after launch.

### Window-boundary double capacity: accepted for devnet

An attacker can receive 3,200 BIT immediately before a boundary and another 3,200 immediately after it. The second batch costs 12.5% more, total exposure is 6,400 BIT, and that is approximately 0.0244% of one section's paid allocation. A rolling token bucket would remove the edge but adds state and custom arithmetic. Devnet telemetry should measure boundary concentration before adding that complexity.

### Stale quotes and capacity races: fixed

Every execution binds the window identifier, maximum unit price, maximum total price, and minimum BIT reward. A rollover or competing transaction that consumes the final capacity fails atomically instead of accepting worse terms.

## Real-SBF Surfpool stress

The Surfpool test deploys the actual SBF artifact and submits 128 signed maximum-size flip transactions concurrently. All 2,048 pixel changes, 128 section revisions, the game counter, and 20,480,000 lamports of application fees reconcile exactly after contention.

An initial attempt to fund 128 separate accounts through 128 setup transactions caused the isolated Surfpool RPC to become unreachable after approximately 103 seconds. Reusing Surfpool's pre-funded payer removed that test-harness bottleneck; the 128-transaction burst then completed successfully. Sybil equivalence remains covered by the controller property tests rather than expensive account setup.

This is a correctness stress test, not a throughput benchmark. It exposed two shared writable accounts in every current `FlipPixels` transaction:

- `GameState`, because every section increments one global `total_flips`; and
- the global treasury, because every section transfers fees directly to it.

Those accounts serialize otherwise independent sections. Before measuring sharded capacity, remove the global counter from the hot path, emit/index aggregate activity off-chain, and accrue the protocol fee in each section PDA for later sweeping. Then repeat the real-SBF stress test across multiple sections with transaction profiling enabled.

Token-2022 transfer and first-time associated-token-account compute are intentionally not claimed here because the rebuilt program does not implement BIT custody or rewards yet. Those paths require a later real-SBF suite after the vault invariant is implemented.
