# ADR 0004: Time-based section price controller

- Status: proposed
- Date: 2026-09-06
- Owner: @ifiokjr

## Conclusion

Every section should have an independent, time-based BIT issuance price. A busy section becomes progressively more expensive; a quiet section becomes cheaper. The controller targets a sustainable rate of rewarded pixel flips, while a small inventory floor increases as that section consumes its finite BIT allocation.

The recommended mechanism is a bounded, additive five-minute target-rate controller informed by EIP-1559, not the historical lifetime-average formula. It is simple enough for deterministic integer arithmetic, gives the wallet a predictable maximum price, and prevents alternating burst/idle traffic from ratcheting the price down.

Strictly, this is congestion-controlled primary issuance rather than a classic automated-market-maker bonding curve. It sells a fixed reserve from the program vault; it does not promise to buy BIT back or maintain a market price.

## Recovered implementation

The pre-rebuild code already contained most of the product idea. For each section it calculated:

```text
current_rate  = emitted_bits / elapsed_seconds
required_rate = remaining_bits / remaining_seconds
pace_ratio    = current_rate / required_rate
static_price  = 100,000 lamports + 512 × sqrt(emitted_bits)
price         = static_price × sqrt(pace_ratio)
```

It floored the result at 10,000 lamports. This made an under-target section cheaper and an over-target section more expensive while attempting to exhaust 262,144 BIT in 60 days.

The same code declared a conservative maximum of 24 section updates per second:

```text
12,000,000 writable-account units / 200,000 units × 40% = 24
```

That constant was never used by the price calculation. It also treated the 200,000-unit default instruction limit as actual usage and omitted block cadence, so it is not a reliable capacity measurement.

## Capacity and emission target

The selected section allocation and historical duration imply an allocation-clearing rate of:

```text
section target = 26,214,400 / 5,184,000 seconds
               = 5.05679 rewarded pixels/second
               = 303.407 rewarded pixels/minute

game target    = 256 × section target
               = 1,294.54 rewarded pixels/second
```

A five-minute allocation-clearing window would therefore target approximately 1,517 rewarded pixels per section. The selected beta target is instead the lower power of two, 1,024 BIT, with a 2,048-BIT burst cap. It is easier to reason about and leaves 32.5% of a section's inventory for games when traffic holds exactly at target for the full 60 days.

At full 16-pixel batches, one section needs 64 transactions per five minutes at target, or 0.213 transactions per second, and 128 transactions per window at the burst cap, or 0.427 per second. If all 256 sections are simultaneously active, that is about 54.6 transactions per second at target and 109.2 at the cap. One-pixel transactions are the adverse case: about 873.8 and 1,747.6 transactions per second across all sections respectively.

The target is fixed for the lifetime of the section and must ultimately be validated against measured devnet capacity with headroom. The 1,024/2,048 staging values are emission parameters, not a claim of measured Solana throughput. The actual Token-2022 transfer path now survives an exact 2,048-BIT real-SBF window and concurrent 1,024-BIT streams across two isolated section shards. That is strong correctness evidence for the initially active section, but it is not a validator or RPC throughput benchmark. Choosing 2,048 as the target would still double full-game pressure without demonstrating a product benefit, so 1,024 remains the safer beta target.

The target is not increased to force the allocation to empty by a deadline. After each window, target shortfall moves irreversibly to the section reward pool, and expiry moves all remaining base inventory. This avoids a final-window attack in which a bot waits for the price floor and then drains an inflated catch-up target.

The economic counter must count rewarded pixels, not transactions. A one-pixel and a 16-pixel transaction consume different amounts of the finite BIT reserve. Capacity testing must separately record transactions, pixel count, and compute units.

Current Solana documentation lists a 60,000,000-unit block limit, a 12,000,000 writable-account-unit limit, a 200,000-unit default per non-builtin instruction, and a 1,400,000-unit transaction maximum. The default is a limit, not the measured cost of `FlipPixels`. Solana also schedules around writable-account contention, which is why separate section accounts provide useful sharding:

- <https://solana.com/docs/core/fees/compute-budget>
- <https://solana.com/developers/courses/program-optimization/program-architecture>

Before selecting a production throughput ceiling, benchmark the actual one-pixel and 16-pixel instructions with the Token-2022 transfer, associated token-account path, owner fee split, and colour event present. Use the p95 measured compute plus headroom rather than a historical network constant.

## Why the historical curve needs replacing

The recovered curve had the right direction but several unsafe properties:

- it used the lifetime average, so a burst late in a 60-day section barely moved the price;
- its response was extremely sensitive near launch and near the deadline;
- `remaining_bits = allocation - emitted_bits` could underflow after exhaustion;
- a zero or nearly zero denominator produced discontinuities and fallback prices;
- the square-root fixed-point operations made boundary behaviour harder to audit; and
- the unused throughput constant gave a false impression that capacity was enforced.

The historical snapshots demonstrate the sensitivity: five flips cost about 100,575 lamports after 100 seconds, 71,116 after 200 seconds, 50,286 after 400 seconds, and 31,801 after 1,000 seconds. Nothing new happened between those quotes; the price fell solely because the same five flips were averaged over a longer lifetime.

## Recommended controller

Each section has its own emission clock, beginning when the section is launched or claimed. A sale does not reset the clock, inventory, control window, or price. Using one global game deadline would make late-created sections dump BIT at their minimum price to catch up.

The on-chain transition must read time from Solana's `Clock` sysvar. A player or section owner must never supply the controller timestamp.

The game snapshots these parameters for every section it creates:

- emission duration;
- control-window duration;
- starting, minimum, and maximum price;
- ending inventory-floor price;
- maximum additive price step denominator;
- rewarded-token target per window;
- target allocation; and
- burst elasticity.

The proposed staging defaults are a 60-day emission duration, five-minute windows, and a maximum change denominator of eight. Price configuration remains explicit per game; the current 10,000-lamport fee is not silently treated as a mainnet price.

For a completed window, the maximum additive step is fixed from the section's launch price:

```text
A = rewarded pixels in the completed window
T = target rewarded pixels for that window
U = clamp(A / T, 0, 2)
S = starting_price / 8

next_controller_price =
  clamp(min_price, max_price, current_price + S × (U - 1))
```

This retains the useful target-utilisation shape of EIP-1559: usage at target leaves the price unchanged, usage above target raises it, and usage below target lowers it. It deliberately does not multiply the current price. The multiplicative rule allowed equal 200%/0% windows to reduce price by about 1.56% per pair, giving a player a timing discount while average utilisation remained exactly on target. The additive rule makes equal positive and negative pressure cancel. Research on EIP-1559 also shows why the step must remain bounded to avoid unstable or chaotic behaviour:

- <https://eips.ethereum.org/EIPS/eip-1559>
- <https://arxiv.org/abs/2102.10567>

With a 10,000-lamport controller price and a target of 1,024 integer flip units for an illustrative window:

| Rewarded pixels | Target utilisation | Next controller price |
| --------------: | -----------------: | --------------------: |
|               0 |                 0% |        8,750 lamports |
|             512 |                50% |        9,375 lamports |
|           1,024 |               100% |       10,000 lamports |
|           1,536 |               150% |       10,625 lamports |
|   2,048 or more |       200% or more |       11,250 lamports |

Six consecutive windows at or above twice the target raise the staging controller from 10,000 to 17,500 lamports. Four empty windows take it from 10,000 to the configured 5,000-lamport minimum. Transactions in one window use one posted price, so transaction ordering cannot change that price. Ordering can still decide who receives the last available rewards near the window cap; `minimum_reward_tokens` makes that race fail safely instead of silently changing the payout.

The target does not catch up after missed windows. Each missed target moves from base inventory to the reward pool. Once time or inventory is exhausted, base BIT distribution stops cleanly and expiry transfers the final unallocated base balance to the pool. A section launch, ownership sale, or long idle period cannot reset its clock, price, inventory, pool, or target.

Use integers and `u128` intermediates. Missed empty windows apply their total additive decay in one constant-time operation. The on-chain implementation must not use floating point, unchecked subtraction, or a loop that advances one window at a time.

## Inventory floor

Rate control alone keeps the price flat when demand exactly matches the target. A small deterministic floor retains the useful part of a conventional bonding curve and rewards earlier participation:

```text
inventory_floor =
  start_floor
  + (end_floor - start_floor) × emitted_bits / allocation

quoted_price = max(inventory_floor, next_controller_price)
```

A linear floor is preferred over the historical square root because it is easy to explain, quote, and audit. Its start and end values are game configuration chosen before launch. They cannot be changed by a section owner or during an active game.

## Burst handling

A posted price that updates only every five minutes must not sell the entire reserve during one cheap window. Base BIT rewards in a window are therefore capped at the configured elasticity multiplied by that window's target. With an elasticity of two, exactly 2,048 whole BIT can be distributed before the next price update.

ABI version 5 stops flips when reward capacity is used, even if a custom client signs a zero minimum. This keeps every base-game pixel equal to one whole BIT and prevents free flips from advancing activity-based section unlocks. The next window restores capacity at its new price. A future unrewarded mode would need a separate instruction whose activity semantics are explicit. Owner-funded campaign rewards are a separate budget and do not bypass the base-emission cap.

This cap prevents a bot from draining a stale-price reserve and keeps maximum reward throughput related to the modelled capacity. Transactions immediately before and after a boundary can access two caps—4,096 staging BIT—within seconds, but the second cap is posted at the higher price and the exposure remains 0.015625% of one section allocation. A continuous leaky-bucket controller could remove that edge later, but it introduces more state and has less precedent. Start with the auditable windowed mechanism and monitor boundary concentration on devnet.

## Manipulation resistance

Only successful, paid, rewarded pixel changes affect utilisation. Duplicate coordinates remain invalid. The owner cannot set the base curve, target, inventory floor, or window length.

An owner can use another wallet to create artificial demand, but doing so pays the protocol share of every flip and consumes the section's finite issuance. With the proposed 20% owner share, self-generated staging demand still loses 80% of its SOL charges to the protocol. The completed-window delay means a manipulator cannot raise the price charged to later Bitflip players within the same control window. Splitting one batch across Sybil wallets does not change rewards, capacity, or price.

Every flip quote binds:

- section and policy version;
- posted per-BIT price;
- maximum total SOL fee;
- minimum total BIT reward; and
- reward-window identifier.

If a rollover, sale, or competing transaction changes the quote, the transaction fails rather than accepting worse terms.

## Alternatives considered

| Mechanism                      | Advantage                                    | Main problem                                                                        |
| ------------------------------ | -------------------------------------------- | ----------------------------------------------------------------------------------- |
| Historical lifetime pace ratio | Closest to the original code                 | Poor recent-demand response and difficult boundary arithmetic                       |
| Supply-only bonding curve      | Simple and predictable                       | A quiet and a congested section with equal sales have the same price                |
| Multiplicative EIP-1559 step   | Familiar percentage response                 | Alternating equal pressure ratchets the issuance price downward                     |
| Full PID controller            | Can correct both recent and cumulative error | More state, tuning risk, oscillation, and harder consensus review                   |
| Continuous leaky bucket        | Immediate response without window cliffs     | More custom fixed-point arithmetic and less proven behaviour                        |
| Batch auction                  | Strong price discovery                       | Slow, complex, and wrong for direct manipulation of a live canvas                   |
| Oracle-linked market price     | Can reduce external arbitrage                | Adds oracle latency, failure, and manipulation risk without solving emission pacing |

## Required tests and evidence

The deterministic controller, game snapshot, section state, permissionless settlement, fixed custody, atomic paid-flip path, and fixed owner fee share are implemented in economy ABI version 5. A mainnet reward-bearing release remains blocked on all of the following:

1. The deterministic [economics simulator](../economics-simulation.md) continues to cover idle, target, burst, oscillating, deadline catch-up, and owner wash-trading cost scenarios as the on-chain ABI is implemented.
2. Property and unit tests continue to prove price bounds, maximum step size, no arithmetic overflow, no reward beyond inventory or window capacity, final partial-window exhaustion, monotonic inventory floor, ordering independence, and atomic quote failure.
3. On-chain lifecycle tests cover late launch, ownership sale, sealing, exhaustion, and timestamp jumps without resetting the controller. The controller is embedded in the section account; real-SBF tests cover independent launch, sale, sealing, exact window exhaustion, settlement, and sharded paid issuance. Broader timestamp-jump coverage remains required.
4. Real-SBF tests record compute for one and 16 pixels, first-time associated token-account creation, and existing token-account transfers. Existing-account correctness is covered; durable compute measurements still need to be captured from the staged artifact.
5. The application displays the current signed total SOL price and whole-BIT reward before submission. Next-window time and remaining capacity still need first-class presentation before mainnet.
6. Devnet telemetry records per-section transaction rate, rewarded-pixel rate, batch size, quote failures, price, reward exhaustion, and compute units.
7. Mainnet parameters are selected from that evidence and receive an independent program and economic review.
