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

The historical allocation and duration imply:

```text
section target = 262,144 / 5,184,000 seconds
               = 0.0505679 rewarded pixels/second
               = 3.03407 rewarded pixels/minute

game target    = 256 × section target
               = 12.9454 rewarded pixels/second
```

A five-minute control window therefore targets approximately 15.17 rewarded pixels per section. The simulator rounds the staging target to one full 16-pixel batch. If users submit smaller batches, the same economic target can require more transactions; if they submit full batches, transaction pressure falls substantially.

The target is fixed for the lifetime of the section and must ultimately come from measured real-SBF capacity with headroom. It is not increased to force the allocation to empty by a deadline. If the section is quiet, undistributed BIT remains locked. This avoids a final-window attack in which a bot waits for the price floor and then drains an inflated catch-up target.

The economic counter must count rewarded pixels, not transactions. A one-pixel and a 16-pixel transaction consume different amounts of the finite BIT reserve. Capacity testing must separately record transactions, pixel count, and compute units.

Current Solana documentation lists a 60,000,000-unit block limit, a 12,000,000 writable-account-unit limit, a 200,000-unit default per non-builtin instruction, and a 1,400,000-unit transaction maximum. The default is a limit, not the measured cost of `FlipPixels`. Solana also schedules around writable-account contention, which is why separate section accounts provide useful sharding:

- <https://solana.com/docs/core/fees/compute-budget>
- <https://solana.com/developers/guides/advanced/how-to-use-priority-fees>

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

With a 10,000-lamport controller price and a target of 16 integer flip units for an illustrative window:

| Rewarded pixels | Target utilisation | Next controller price |
| --------------: | -----------------: | --------------------: |
|               0 |                 0% |        8,750 lamports |
|               8 |                50% |        9,375 lamports |
|              16 |               100% |       10,000 lamports |
|              24 |               150% |       10,625 lamports |
|      32 or more |       200% or more |       11,250 lamports |

Six consecutive windows at or above twice the target raise the staging controller from 10,000 to 17,500 lamports. Four empty windows take it from 10,000 to the configured 5,000-lamport minimum. Transactions in one window use one posted price, so transaction ordering cannot change that price. Ordering can still decide who receives the last available rewards near the window cap; `minimum_reward_tokens` makes that race fail safely instead of silently changing the payout.

The target does not catch up after missed windows. Once time or inventory is exhausted, base BIT distribution stops cleanly. A section launch, ownership sale, or long idle period cannot reset its clock, price, inventory, or target.

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

A posted price that updates only every five minutes must not sell the entire reserve during one cheap window. Base BIT rewards in a window are therefore capped at the configured elasticity multiplied by that window's target. With an elasticity of two, approximately 30–32 BIT can be distributed before the next price update.

Players may continue flipping after the reward capacity is used, but the quote must show zero base BIT and their signed `minimum_reward_tokens` must protect them from an unexpected zero payout. The next window restores capacity at its new price. Owner-funded campaign rewards are a separate budget and do not bypass the base-emission cap.

This cap prevents a bot from draining a stale-price reserve and keeps maximum reward throughput related to the modelled capacity. A transaction immediately before and after a boundary can access two caps—64 staging BIT—within seconds, but the second cap is posted at the higher price and the exposure remains 0.0244% of one section allocation. A continuous leaky-bucket controller could remove that edge later, but it introduces more state and has less precedent. Start with the auditable windowed mechanism and monitor boundary concentration on devnet.

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

Implementation is blocked on all of the following:

1. The deterministic [economics simulator](../economics-simulation.md) continues to cover idle, target, burst, oscillating, deadline catch-up, and owner wash-trading cost scenarios as the on-chain ABI is implemented.
2. Property and unit tests continue to prove price bounds, maximum step size, no arithmetic overflow, no reward beyond inventory or window capacity, final partial-window exhaustion, monotonic inventory floor, ordering independence, and atomic quote failure.
3. On-chain lifecycle tests cover late launch, ownership sale, sealing, exhaustion, and timestamp jumps without resetting the controller. Ownership and lifecycle status deliberately remain outside the pure pricing state, so those invariants cannot be proven until the controller is embedded in the section account.
4. Real-SBF tests measure compute for one and 16 pixels, first-time associated token-account creation, and existing token-account transfers.
5. The application displays the current price, next update, remaining window reward capacity, and signed SOL/BIT bounds.
6. Devnet telemetry records per-section transaction rate, rewarded-pixel rate, batch size, quote failures, price, reward exhaustion, and compute units.
7. Mainnet parameters are selected from that evidence and receive an independent program and economic review.
