# ADR 0004: Time-based section price controller

- Status: proposed
- Date: 2026-09-06
- Owner: @ifiokjr

## Conclusion

Every section should have an independent, time-based BIT issuance price. A busy section becomes progressively more expensive; a quiet section becomes cheaper. The controller targets a sustainable rate of rewarded pixel flips, while a small inventory floor increases as that section consumes its finite BIT allocation.

The recommended mechanism is a bounded, five-minute target-rate controller modelled on EIP-1559, not the historical lifetime-average formula. It is simple enough for deterministic integer arithmetic, gives the wallet a predictable maximum price, and cannot jump more than 12.5% at one update.

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

A five-minute control window therefore targets approximately 15.17 rewarded pixels per section. One full 16-pixel batch roughly fills a window. If users submit smaller batches, the same economic target can require more transactions; if they submit full batches, transaction pressure falls substantially.

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

The game snapshots these parameters for every section it creates:

- emission duration;
- control-window duration;
- starting, minimum, and maximum price;
- ending inventory-floor price;
- maximum price change denominator;
- target allocation; and
- burst elasticity.

The proposed staging defaults are a 60-day emission duration, five-minute windows, and a maximum change denominator of eight. Price configuration remains explicit per game; the current 10,000-lamport fee is not silently treated as a mainnet price.

For a completed window:

```text
A = rewarded pixels in the completed window
T = target rewarded pixels for that window
U = clamp(A / T, 0, 2)

next_controller_price =
  clamp(min_price, max_price, current_price × (1 + (U - 1) / 8))
```

This is the same useful shape as EIP-1559: usage at target leaves the price unchanged; usage above target raises it; usage below target lowers it; and the elasticity plus denominator bound each step. EIP-1559 uses a maximum 12.5% base-fee change, allowing wallets to set reliable fee ceilings. Research on the mechanism also shows why the step size must remain bounded to avoid unstable or chaotic behaviour:

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

Six consecutive windows at or above twice the target approximately double the controller price. Six empty windows approximately halve it; the configured minimum stops further decay. Transactions in one window use one posted price, so transaction ordering cannot change that price. Ordering can still decide who receives the last available rewards near the window cap; `minimum_reward_tokens` makes that race fail safely instead of silently changing the payout.

The target for the next window is recalculated from the section's remaining BIT and remaining emission time. This preserves the original catch-up intent:

- if distribution is behind schedule, the next target rises and downward price pressure becomes more likely;
- if distribution is ahead, the target falls and upward pressure becomes more likely; and
- once time or inventory is exhausted, base BIT distribution stops cleanly.

Use fixed-point integers and `u128` intermediates. The on-chain implementation must not use floating point, unchecked subtraction, or an unbounded loop to advance missed windows.

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

This cap prevents a bot from draining a stale-price reserve and keeps maximum reward throughput related to the modelled capacity. A continuous leaky-bucket controller could remove the window boundary later, but it introduces more fixed-point state and has less precedent. Start with the auditable windowed mechanism.

## Manipulation resistance

Only successful, paid, rewarded pixel changes affect utilisation. Duplicate coordinates remain invalid. The owner cannot set the base curve, target, inventory floor, or window length.

An owner can use another wallet to create artificial demand, but doing so pays the protocol share of every flip and consumes the section's finite issuance. The completed-window delay means a manipulator cannot raise the price charged to later Bitflip players within the same control window. The 12.5% bound limits the effect of any one window.

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
| Full PID controller            | Can correct both recent and cumulative error | More state, tuning risk, oscillation, and harder consensus review                   |
| Continuous leaky bucket        | Immediate response without window cliffs     | More custom fixed-point arithmetic and less proven behaviour                        |
| Batch auction                  | Strong price discovery                       | Slow, complex, and wrong for direct manipulation of a live canvas                   |
| Oracle-linked market price     | Can reduce external arbitrage                | Adds oracle latency, failure, and manipulation risk without solving emission pacing |

## Required tests and evidence

Implementation is blocked on all of the following:

1. A deterministic simulator covers idle, target, burst, oscillating, late launch, sale, seal, exhaustion, and timestamp-jump scenarios.
2. Property tests prove price bounds, maximum step size, no arithmetic overflow, no reward beyond inventory or window capacity, and monotonic inventory floor.
3. Real-SBF tests measure compute for one and 16 pixels, first-time associated token-account creation, and existing token-account transfers.
4. The application displays the current price, next update, remaining window reward capacity, and signed SOL/BIT bounds.
5. Devnet telemetry records per-section transaction rate, rewarded-pixel rate, batch size, quote failures, price, reward exhaustion, and compute units.
6. Mainnet parameters are selected from that evidence and receive an independent program and economic review.
