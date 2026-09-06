# ADR 0003: Section economy and owner governance

- Status: proposed
- Date: 2026-09-06
- Owner: @ifiokjr

## Conclusion

Section ownership should confer real, bounded control over a section's game and economy. An owner should be able to choose an approved game mode, publish its rules, set an entry price, fund a reward budget, and decide how quickly that budget is paid out. The budget follows the section while committed to a live campaign, and a higher reward rate exhausts it sooner.

It is not economically safe to give every new section owner a large, freely distributable token grant. An owner can always use fresh wallets to pay rewards to themselves. The viable beta model therefore has two separate sources:

1. a finite, protocol-priced BIT allocation that players receive for paid pixel flips; and
2. an owner/sponsor-funded campaign budget that the owner can distribute under bounded rules.

The second half of the historical supply allocation remains locked unless a later, independently reviewed matching programme defines how it can be earned. It is not silently granted to section purchasers.

This ADR is a design gate, not a claim that token rewards or campaign governance exist in the rebuilt program today.

## Recovered intent

The pre-rebuild program contained a coherent binary-themed allocation:

| Quantity                          |                 Amount |
| --------------------------------- | ---------------------: |
| Total BIT supply                  | 1,073,741,824 (`2^30`) |
| Games/seasons                     |                      8 |
| Allocation per game               |   134,217,728 (`2^27`) |
| Sections per game                 |                    256 |
| Allocation per section            |       524,288 (`2^19`) |
| Paid-flip allocation per section  |       262,144 (`2^18`) |
| Discretionary rewards per section |       262,144 (`2^18`) |

It also used a zero-decimal Token-2022 BIT mint and attempted to pace paid distribution with a square-root/time price curve. A player paid SOL into the section and received BIT from the section's token account.

The design was incomplete. Production code did not fund newly created section token accounts, the discretionary `reward_tokens` value existed only in the backend database, and there was no enforceable reward policy or anti-self-dealing mechanism. The rebuilt program correctly removed that unfinished faucet.

## Why the discretionary grant is unsafe

At the current defaults, a section costs 0.01 SOL to claim and each pixel costs 0.00001 SOL to flip. Handing the purchaser 262,144 discretionary BIT with the section would price that grant at about 38 lamports per BIT before rent, while the proposed one-BIT-per-paid-flip path would charge 10,000 lamports. That is a roughly 262-fold discount before considering owner revenue.

Restricting the owner to a maximum reward rate does not solve the problem. At a 16 BIT bonus per flip, a purchaser could route the entire grant through fresh wallets in 16,384 flips for 0.16384 SOL of application fees. Sybil wallets make "the owner cannot reward themselves" unenforceable.

The system must treat owner-controlled rewards as a marketing or prize budget, not as a free asset bundled with ownership. Owners and sponsors can deposit BIT they already hold. A future protocol match may add funds, but only under a separately approved policy with a hard global and per-section cap.

If every claimed section must start with a discretionary budget, the claimant has to buy that budget at no less than the active issuance price. At the current 10,000-lamport flip fee, preloading the historical 262,144 BIT would add at least 2.62144 SOL to the claim before rent and protocol fees. That conflicts with inexpensive lazy expansion. A much smaller prepaid starter budget can use the same invariant, but a free full allocation cannot.

## Token model

The recommended BIT mint has these properties:

- one Token-2022 mint with zero decimals;
- a fixed maximum supply of `2^30` BIT, minted once into a program-controlled distribution vault;
- mint and freeze authority revoked after the supply and metadata are verified;
- no permanent delegate, transfer fee, interest, rebasing, or additional denomination mints; and
- KiB, MiB, and GiB are display units calculated by clients, not separate assets.

Solana supports revoking mint and freeze roles with `SetAuthority`. Avoiding a permanent delegate is deliberate: that extension can transfer or burn from any holder's token account and holders cannot revoke it. Token-2022 extensions generally have to be selected when the mint is initialized, so the exact mint layout is part of the deployment ceremony:

- <https://solana.com/docs/tokens/basics>
- <https://solana.com/docs/tokens/extensions>
- <https://solana.com/docs/tokens/extensions/permanent-delegate>

The historical `2^30` cap and eight-game allocation retain the project's binary identity, but they do not create economic value. BIT must have useful, visible sinks and Bitflip must make no promise of a SOL or fiat redemption price.

## Paid-flip allocation

Each section may distribute at most `2^18` BIT through ordinary paid flips. The starting rule is one BIT per successfully toggled pixel. A 16-pixel transaction therefore earns 16 BIT; batching already reduces the network fee per pixel, so a second hidden batch subsidy is unnecessary.

The player's transaction includes both a maximum SOL charge and a minimum BIT payout. It fails if either side has changed. If the section's paid allocation is exhausted, flipping can continue, but the UI must say that no base BIT remains before signing.

The paid allocation is a finite primary distribution, not risk-free yield. A fixed SOL price cannot track a transferable token's market price without an oracle, and a stale cheap issuance price will be arbitraged. [ADR 0004](0004-section-price-controller.md) proposes an independently paced, time-based price for every section, bounded burst issuance, and a small inventory floor. The first beta must remain on devnet, avoid seeding exchange liquidity, and collect data before those proposed parameters can become a mainnet decision.

## Owner-funded campaign budget

An owner or sponsor can deposit BIT into the section's campaign budget. Once a campaign begins, the committed budget cannot be withdrawn or redirected. A higher configured payout consumes the same finite budget sooner.

For a 262,144 BIT budget, the simple per-pixel case looks like this:

| Bonus per qualifying pixel | Qualifying pixels | Full 64 × 64 board equivalents |
| -------------------------: | ----------------: | -----------------------------: |
|                      1 BIT |           262,144 |                             64 |
|                      2 BIT |           131,072 |                             32 |
|                      4 BIT |            65,536 |                             16 |
|                      8 BIT |            32,768 |                              8 |
|                     16 BIT |            16,384 |                              4 |

The UI should present both "BIT per action" and an estimated number of remaining actions. When a budget reaches zero, the campaign either continues without a bonus or ends according to its published rules; it must never mint more BIT.

Unused owner-funded BIT is refundable only before a campaign starts or after it has ended. A live campaign and its committed budget travel with the section if ownership is sold, so a sale cannot invalidate prizes already advertised.

## Owner governance

For an active section, its owner can configure the next campaign using bounded, protocol-understood fields:

- an allowlisted mode such as open canvas, eight-colour contested canvas, or a future capture-the-flag mode;
- a palette identifier and other mode-specific bounded options;
- a content hash for the human-readable rules;
- start and end times;
- BIT entry price;
- reward formula and maximum reward per wallet; and
- the committed reward budget.

Rules are versioned and immutable while a campaign is live. Players sign the expected policy version, maximum entry/flip cost, and minimum reward, so an owner cannot change terms between preview and execution. Arbitrary executable code, arbitrary payout accounts, and arbitrary token minting are not governance options.

Section ownership also receives a protocol-defined share of that section's SOL flip fees. The recommended staging value is 20%, fixed for the lifetime of a game; the remaining 80% goes to the protocol treasury. The owner cannot set this percentage. At today's 10,000-lamport fee, 20% yields:

| Activity                   | Gross application fees |  Owner share | Protocol share |
| -------------------------- | ---------------------: | -----------: | -------------: |
| One full 4,096-pixel board |            0.04096 SOL | 0.008192 SOL |   0.032768 SOL |
| 262,144 paid flips         |            2.62144 SOL | 0.524288 SOL |   2.097152 SOL |

Ignoring rent and transaction fees, the current 0.01 SOL claim price is recovered after 5,000 paid flips at a 20% share. On 2026-09-06, mainnet reported 0.004933407 SOL as the rent-exempt minimum for the current 651-byte section; including that capital raises the illustrative break-even to 7,467 flips, or just under two full-board equivalents. Adding 64 bytes of policy and ledger state would raise rent by approximately 0.000405312 SOL at the same schedule. Deployment tooling must recalculate rent rather than hard-code either observation.

This is only a staging hypothesis, not an income promise. The final share and claim price require observed retention, bot behaviour, SOL price sensitivity, and programme costs.

Protocol-owned section `0` routes both shares to the protocol. Later owners receive their share atomically on each flip. The protocol share accrues as backed lamports in the already-writable section PDA and is swept separately; sending every flip directly to one global treasury would serialize all section shards. This requires a bounded protocol-fee ledger field but no additional rent-bearing revenue account.

## Game loop and token sinks

BIT needs a use beyond being issued. The smallest coherent loop is:

1. a player pays SOL to flip pixels and receives finite base BIT;
2. a section owner chooses a campaign and seeds its prize budget;
3. players spend BIT to enter that campaign;
4. entry BIT refills the section's committed prize budget; and
5. winners claim bounded payouts from that budget.

This lets successful sections recycle activity into prizes instead of depending on perpetual inflation. A campaign may also award small owner-funded bonuses for qualifying colour flips. Owners should be warned that unconditional per-flip bonuses are bot-farmable; contest outcomes and per-wallet caps are safer uses of the budget.

## Off-chain game integrity

Colour does not need to occupy the permanent bitmap. The flip instruction can validate an eight-colour index and emit the colour, player, coordinates, and policy version. The backend indexer reconstructs the contested colour canvas from confirmed transactions while the on-chain bitmap remains one bit per pixel.

An emitted colour is objectively attributable to a paid on-chain action. The winner of an off-chain drawing or capture-the-flag game is not. The beta may use a configured Bitflip game attestor to issue single-use, expiring claim vouchers that are bounded by the campaign budget. The UI and rules must describe that trust explicitly. A later optimistic Merkle settlement with a challenge window can reduce that trust, but should not be built before a real game needs it.

Section owners select an approved game and its parameters; they do not become a mint authority or gain access to protocol custody. Protocol-funded rewards must never depend solely on an owner's signature.

## Low-rent custody design

Creating a Token-2022 account for all 256 sections up front would undermine the lazy allocation model. Use one program-controlled BIT distribution vault and store each created section's paid and campaign balances as on-chain ledger fields. The sum of section liabilities must never exceed the vault balance.

This adds a small amount of rent only when the bitmap section itself is created. It does not create 256 token accounts. A player needs one associated BIT token account the first time they receive BIT; the transaction must disclose who pays that rent. For scale, mainnet reported 0.001855569 SOL for a 165-byte base token account on 2026-09-06; the exact Token-2022 account size and current rent must be calculated from the selected extensions during deployment.

The vault PDA can sign only transfers from the vault. With mint/freeze authority revoked and no permanent delegate, the program cannot seize BIT that has reached a player's wallet.

## Required implementation order

1. Simulate the [time-based section price controller](0004-section-price-controller.md), owner share, claim break-even, and likely bot strategies using staging activity assumptions.
2. Implement and independently review the fixed mint plus global-vault invariants on a fresh devnet program ID.
3. Add base paid-flip distribution with SOL/BIT slippage protection and real-SBF tests for exhaustion, batching, custody, and arithmetic boundaries.
4. Add owner fee sharing and versioned section policies. Policies cannot change during a live campaign and must survive a section sale.
5. Add owner/sponsor deposits, entry payments, budget exhaustion, cancellation, and refund rules.
6. Add colour events and the indexed eight-colour canvas without attaching token payouts.
7. Add a single allowlisted off-chain contest and bounded voucher settlement, then test replay, equivocation, expiry, sale, seal, and indexer recovery.
8. Run the economy on devnet before any token or reward code is eligible for mainnet.

## Mainnet gates

Mainnet remains blocked until all of the following are recorded:

- exact Token-2022 extension set, mint address, supply, vault balance, revoked authorities, and reproducible transaction signatures;
- an independent program and token-economics review;
- evidence that section allocations always reconcile to the vault;
- an explicit decision on transferability, exchange/liquidity support, legal presentation, and tax/accounting treatment;
- measured staging data supporting the issuance curve, claim price, and owner fee share;
- at least one useful BIT sink operating end to end; and
- product copy that calls rewards finite game credits and never guaranteed profit, yield, or redeemable value.
