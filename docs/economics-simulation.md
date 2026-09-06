# Economics simulation and adversarial findings

This is reproducible engineering evidence for [ADR 0004](decisions/0004-section-price-controller.md), not a mainnet parameter recommendation. The controller configuration is snapshotted in each game, and every section stores an independently launched controller and custody ledger. `SettleSectionEconomy` permissionlessly moves completed-window shortfalls into the pool. ABI version 4 validates the immutable zero-decimal Token-2022 mint, funds one sharded vault per active section, and atomically exchanges the signed SOL charge for the signed minimum BIT reward while toggling pixels.

## Run it

```sh
devenv shell -- simulate:economics
devenv shell -- test:rust
devenv shell -- test:surfpool
```

`simulate:economics` executes the same `no_std` integer controller exported by `bitflip_program`. The property suite generates 1,024 cases per property and covers arbitrary traffic delays and batches, price and inventory bounds, ordering, Sybil splitting, quote staleness, timestamp reversal, missed windows, final partial-window exhaustion, and representable `u64` extremes.

## Staging scenario output

The staging model uses a 1,024-BIT target, 2,048-BIT window cap, five-minute windows, 10,000-lamport starting price, 5,000-lamport minimum, 100,000-lamport ending inventory floor, and a 20% owner fee share. BIT has zero decimals: one rewarded pixel always means one whole BIT. Target shortfall moves into the section reward pool when a window settles.

| Scenario                     | Rewarded BIT | Paid lamports | Owner recovery | Protocol revenue | Next price | Reward pool BIT |
| ---------------------------- | -----------: | ------------: | -------------: | ---------------: | ---------: | --------------: |
| 12 idle windows              |            0 |             0 |              0 |                0 |      5,000 |          12,288 |
| 12 windows at target         |       12,288 |   122,880,000 |     24,576,000 |       98,304,000 |     10,000 |               0 |
| 12 saturated windows         |       24,576 |   414,720,000 |     82,944,000 |      331,776,000 |     25,000 |               0 |
| 6 alternating burst/idle     |       12,288 |   122,880,000 |     24,576,000 |       98,304,000 |     10,000 |           6,144 |
| 59-day idle then 102,400 ask |        2,048 |    10,240,000 |      2,048,000 |        8,192,000 |      5,000 |      26,212,352 |

Amounts exclude Solana transaction fees and token-account rent. Owner recovery is money returned to the owner, so an owner using Sybil wallets to create artificial demand has a net SOL cost equal to the protocol-revenue column while acquiring the finite BIT payout.

## Attacks attempted

### Deadline catch-up drain: fixed

The original proposal recalculated each target from remaining inventory divided by remaining time. After a 59-day idle period, the final target approached the entire 26,214,400-BIT reserve while the price had reached its minimum. A bot could wait and drain the reserve during the cheapest window.

The target is now immutable. A final-window request for 102,400 BIT receives only the 2,048-BIT cap. Unissued base BIT moves to the programme-controlled reward pool at expiry; the late caller cannot claim it.

### Multiplicative burst/idle ratchet: fixed

The first EIP-1559-style implementation multiplied the current price by 1.125 after a saturated window and by 0.875 after an idle window. Those factors multiply to 0.984375, so traffic averaging exactly 100% utilisation reduced the price by roughly 1.56% every pair. Six pairs reduced 10,000 lamports to 9,095.

The controller now adds or subtracts a fixed utilisation-weighted step based on the launch price. Equal positive and negative pressure returns exactly to 10,000 lamports in the simulation.

### Sybil splitting: bounded

One hundred twenty-eight 16-BIT batches and 2,048 one-BIT wallets produce identical state, rewards, and issuance charges. Wallet identity is not an input. An owner can still manufacture demand, but the proposed 20% owner share leaves 80% of every charge with the protocol and the attacker consumes finite inventory. Protocol reward-pool payouts therefore must not be controlled by the owner, and reward-eligible owner activity must not receive the owner fee share.

This does not prevent ordinary market arbitrage. If transferable BIT is worth more than the posted issuance price, buyers will rationally consume all available capacity. Devnet data and an independent economic review must set the minimum and starting prices.

That arbitrage is especially important because BIT is fungible while section prices differ: rational bots will route to the cheapest section. This can help equalise congestion, but the controller only limits the rate and size of extraction; it cannot guarantee that a SOL-denominated price is economically sufficient. Mainnet needs either conservative empirically selected floors or a separately reviewed oracle policy. Section owners must not be allowed to lower immutable price, target, duration, or inventory parameters after launch.

### Window-boundary double capacity: accepted for devnet

An attacker can receive 2,048 BIT immediately before a boundary and another 2,048 immediately after it. The second batch costs 12.5% more, total exposure is 4,096 BIT, and that is exactly 0.015625% of one section's allocation. A rolling token bucket would remove the edge but adds state and custom arithmetic. Devnet telemetry should measure boundary concentration before adding that complexity.

### Malicious reward distributor: disabled by design

The current controller accrues reward-pool liabilities but exposes no payout authority. A section owner cannot provide a payout list or withdraw the pool. Before payouts are enabled, one protocol-defined epoch policy must make its inputs and weights independently reconstructible from paid, confirmed actions; claims must be capped, pull-based, and replay-protected. Off-chain judged games still require a disclosed attestor or a challengeable result root and a separate adversarial review.

### Stale quotes and capacity races: fixed

Every execution binds the window identifier, maximum unit price, maximum total price, and minimum BIT reward. A rollover or competing transaction that consumes the final capacity fails atomically instead of accepting worse terms.

## Real-SBF Surfpool stress

The Surfpool test deploys the actual SBF artifact and submits 128 signed maximum-size flip transactions concurrently. All 2,048 pixel changes, 128 section revisions, 2,048 BIT debited from the section vault and credited to the player, and 20,480,000 lamports of application fees reconcile exactly after contention. A 2,049th reward-bearing flip is rejected without changing pixels, tokens, or the fee ledger.

An initial attempt to fund 128 separate accounts through 128 setup transactions caused the isolated Surfpool RPC to become unreachable after approximately 103 seconds. Reusing Surfpool's pre-funded payer removed that test-harness bottleneck; the 128-transaction burst then completed successfully. Sybil equivalence remains covered by the controller property tests rather than expensive account setup.

This remains a correctness stress test, not a validator-throughput benchmark. ABI version 4 removes the shared writable `GameState` counter and global-treasury payment from `FlipPixels`; the game and configuration are read-only, while pixels, controller state, BIT custody, and gross application fees are isolated in the section shard. Aggregate flip telemetry must therefore be indexed from confirmed section transactions rather than trusted from `GameState.total_flips`.

A second real-SBF scenario concurrently submits 64 maximum-size transactions to each of two independently owned and funded sections. Their players, vaults, bitmaps, controllers, and fee accounts are disjoint, and both 1,024-BIT streams reconcile after contention. Wider devnet profiling is still required because Surfpool timing does not establish production validator capacity or RPC quality.

The suite also creates a zero-decimal Token-2022 mint and config-owned reserve, rejects custody while mint authority remains live, revokes it, registers the exact fixed supply, creates section ATAs through CPI, transfers exactly one allocation to each active shard, and proves duplicate funding rolls back without moving BIT. Stale windows, underpriced quotes, excessive minimum rewards, duplicates, capacity exhaustion, arithmetic extremes, and Sybil splitting all fail or reconcile without partial state.
