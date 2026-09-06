# ADR 0003: Section economy and owner governance

- Status: proposed
- Date: 2026-09-06
- Owner: @ifiokjr

## Conclusion

Section ownership should confer real, bounded control over a section's game and economy. An owner should be able to choose an approved game mode, publish its rules, set an entry price, add a sponsor budget, and choose from protocol-approved reward policies. It must not let the owner choose recipients, withdraw protocol rewards, or install arbitrary payout logic.

Every section has one 26,214,400 BIT allocation. It begins entirely as base-issuance inventory. After each five-minute window, any shortfall against the immutable target moves irreversibly into that section's reward pool. At the end of the 60-day issuance period, every token still in base inventory moves into the pool. There is no separately locked matching allocation and no free discretionary grant to the section owner.

The pool is an on-chain programme liability, not an owner's wallet. Distribution remains disabled until an independently reviewed, protocol-defined policy can select recipients without relying on the owner or one reward distributor.

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

The selected staging scale gives every section 100 times the historical paid-flip capacity, but does not retain the historical discretionary half. Four games replace eight. Together those decisions reduce the previous staging supply by exactly four:

| Quantity                   |                       Amount |
| -------------------------- | ---------------------------: |
| Total BIT supply           | 26,843,545,600 (`25 × 2^30`) |
| Games/seasons              |                            4 |
| Allocation per game        |                6,710,886,400 |
| Sections per game          |                          256 |
| Allocation per section     |                   26,214,400 |
| Separate locked allocation |                            0 |

This is 100 times more initial reward-bearing flip capacity per section than the historical paid half, not fractional BIT: the mint still has zero decimals and every rewarded pixel earns one whole BIT. A token can be accounted to base issuance or to the reward pool, never both.

The design was incomplete. Production code did not fund newly created section token accounts, the discretionary `reward_tokens` value existed only in the backend database, and there was no enforceable reward policy or anti-self-dealing mechanism. The rebuilt program correctly removed that unfinished faucet.

## Why the discretionary grant is unsafe

At the current defaults, a section costs 0.01 SOL to claim and each pixel costs 0.00001 SOL to flip. Handing the purchaser 26,214,400 discretionary BIT with the section would price that grant at less than one lamport per BIT before rent, while the proposed one-BIT-per-paid-flip path would charge 10,000 lamports. That is a roughly 26,214-fold discount before considering owner revenue.

Restricting the owner to a maximum reward rate does not solve the problem. At a 16 BIT bonus per flip, a purchaser could route the entire grant through fresh wallets in 1,638,400 flips for 16.384 SOL of application fees. Sybil wallets make "the owner cannot reward themselves" unenforceable.

The system must treat owner-controlled rewards as a marketing or prize budget, not as a free asset bundled with ownership. Owners and sponsors can deposit BIT they already hold. Protocol pool funds may be paid only under a separately approved policy with a hard global and per-section cap.

If every claimed section must start with a discretionary budget, the claimant has to buy that budget at no less than the active issuance price. At the current 10,000-lamport flip fee, preloading 26,214,400 BIT would add at least 262.144 SOL to the claim before rent and protocol fees. That conflicts with inexpensive lazy expansion. A much smaller prepaid starter budget can use the same invariant, but a free full allocation cannot.

## Token model

The recommended BIT mint has these properties:

- one Token-2022 mint with zero decimals;
- a fixed maximum supply of `25 × 2^30` BIT (26,843,545,600 whole BIT), minted once into a program-controlled distribution vault;
- mint and freeze authority revoked after the supply and metadata are verified;
- no permanent delegate, transfer fee, interest, rebasing, or additional denomination mints; and
- KiB, MiB, and GiB are display units calculated by clients, not separate assets.

Solana supports revoking mint and freeze roles with `SetAuthority`. Avoiding a permanent delegate is deliberate: that extension can transfer or burn from any holder's token account and holders cannot revoke it. Token-2022 extensions generally have to be selected when the mint is initialized, so the exact mint layout is part of the deployment ceremony:

- <https://solana.com/docs/tokens/basics>
- <https://solana.com/docs/tokens/extensions>
- <https://solana.com/docs/tokens/extensions/permanent-delegate>

The historical binary allocation inspired the scaled four-game cap, but the number itself creates no economic value. BIT must have useful, visible sinks and Bitflip must make no promise of a SOL or fiat redemption price.

## Shared section allocation

Each section starts with `100 × 2^18` BIT (26,214,400 whole BIT). The starting base rule is one BIT per successfully toggled pixel. A 16-pixel transaction therefore earns 16 BIT; batching already reduces the network fee per pixel, so a second hidden batch subsidy is unnecessary.

The five-minute base target is 1,024 BIT and the burst cap is 2,048 BIT. At rollover the programme calculates `max(window_target - rewarded, 0)` and moves that amount from base inventory to the reward pool. It settles arbitrarily many empty windows in constant time and caps the transfer at remaining inventory. Expiry moves the remainder. Consequently:

```text
base BIT emitted + reward-pool BIT + unallocated base BIT
  = 26,214,400 BIT per section
```

At exactly the target for every window over 60 days, 17,694,720 BIT is emitted and 8,519,680 BIT, or 32.5%, enters the pool at expiry. Sustained burst traffic can emit more and leave less for games. This is deliberate: unused base issuance funds future play instead of becoming a late cheap giveaway.

The player's transaction includes both a maximum SOL charge and a minimum BIT payout. It fails if either side has changed. If the section's base inventory is exhausted, flipping can continue, but the UI must say that no base BIT remains before signing.

Base issuance is a finite primary distribution, not risk-free yield. A fixed SOL price cannot track a transferable token's market price without an oracle, and a stale cheap issuance price will be arbitraged. [ADR 0004](0004-section-price-controller.md) proposes an independently paced, time-based price for every section, bounded burst issuance, and a small inventory floor. The first beta must remain on devnet, avoid seeding exchange liquidity, and collect data before those proposed parameters can become a mainnet decision.

## Reward-pool distribution and owner-funded budgets

There is no trusted reward-distributor role in the beta design. A section owner cannot sign an arbitrary payment, upload a final payout list, name a recipient account, or withdraw protocol pool BIT. The first implementation should accrue and expose the pool balance while keeping protocol-pool payout disabled.

The smallest defensible later policy is a precommitted, fixed-length reward epoch:

- the protocol allowlists the formula before the epoch starts;
- qualifying actions and weights are derived from paid, confirmed on-chain actions;
- the owner cannot edit the formula, entrants, weights, or pool once the epoch begins;
- claims are pull-based, capped by the epoch allocation, replay-protected, and independently reconstructible; and
- an owner receives no fee share for self-funded or reward-eligible actions, so ownership does not create a cheaper farming route.

Wallet caps alone are not Sybil resistance. Proportional rewards per paid qualifying action make splitting wallets irrelevant, but remain economically farmable whenever the BIT payout is worth more than the non-refundable SOL cost. A contest whose winner depends on an off-chain judgement still requires either a disclosed trusted attestor or an optimistic result root with a challenge period. Neither should ship with mainnet funds without separate adversarial review.

An owner or sponsor can deposit BIT into the section's campaign budget. Once a campaign begins, the committed budget cannot be withdrawn or redirected. A higher configured payout consumes the same finite budget sooner.

For a 26,214,400 BIT budget, the simple per-pixel case looks like this:

| Bonus per qualifying pixel | Qualifying pixels | Full 64 × 64 board equivalents |
| -------------------------: | ----------------: | -----------------------------: |
|                      1 BIT |        26,214,400 |                          6,400 |
|                      2 BIT |        13,107,200 |                          3,200 |
|                      4 BIT |         6,553,600 |                          1,600 |
|                      8 BIT |         3,276,800 |                            800 |
|                     16 BIT |         1,638,400 |                            400 |

The UI should present both "BIT per action" and an estimated number of remaining actions. When a budget reaches zero, the campaign either continues without a bonus or ends according to its published rules; it must never mint more BIT.

Unused owner-funded BIT is refundable only before a campaign starts or after it has ended. A live campaign and its committed budget travel with the section if ownership is sold, so a sale cannot invalidate prizes already advertised.

## Owner governance

For an active section, its owner can configure the next campaign using bounded, protocol-understood fields:

- an allowlisted mode such as open canvas, eight-colour contested canvas, or a future capture-the-flag mode;
- a palette identifier and other mode-specific bounded options;
- a content hash for the human-readable rules;
- start and end times;
- BIT entry price;
- one allowlisted reward policy and its bounded parameters; and
- the committed reward budget.

Rules are versioned and immutable while a campaign is live. Players sign the expected policy version, maximum entry/flip cost, and minimum reward, so an owner cannot change terms between preview and execution. Arbitrary executable code, arbitrary payout accounts, owner-authored payout lists, and arbitrary token minting are not governance options.

Section ownership also receives a protocol-defined share of that section's SOL flip fees. The recommended staging value is 20%, fixed for the lifetime of a game; the remaining 80% goes to the protocol treasury. The owner cannot set this percentage. At today's 10,000-lamport fee, 20% yields:

| Activity                   | Gross application fees |  Owner share | Protocol share |
| -------------------------- | ---------------------: | -----------: | -------------: |
| One full 4,096-pixel board |            0.04096 SOL | 0.008192 SOL |   0.032768 SOL |
| 26,214,400 paid flips      |            262.144 SOL |  52.4288 SOL |   209.7152 SOL |

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

Creating a Token-2022 account for all 256 sections up front would undermine the lazy allocation model. Use one program-controlled BIT distribution vault and store each created section's unallocated base balance, reward-pool balance, and campaign balance as on-chain ledger fields. For every section, emitted plus pool plus unallocated base must never exceed 26,214,400 BIT. Across the game, the sum of every section liability must never exceed the vault balance.

This adds a small amount of rent only when the bitmap section itself is created. It does not create 256 token accounts. A player needs one associated BIT token account the first time they receive BIT; the transaction must disclose who pays that rent. For scale, mainnet reported 0.001855569 SOL for a 165-byte base token account on 2026-09-06; the exact Token-2022 account size and current rent must be calculated from the selected extensions during deployment.

The vault PDA can sign only transfers from the vault. With mint/freeze authority revoked and no permanent delegate, the program cannot seize BIT that has reached a player's wallet.

## Required implementation order

1. Simulate the [time-based section price controller](0004-section-price-controller.md), owner share, claim break-even, and likely bot strategies using staging activity assumptions.
2. Implement and independently review the fixed mint, global-vault invariant, per-section base ledger, and deterministic shortfall-to-pool transition on a fresh devnet program ID.
3. Add base paid-flip distribution with SOL/BIT slippage protection and real-SBF tests for exhaustion, batching, custody, rollover, and arithmetic boundaries. Accrue the pool but do not pay it yet.
4. Remove the global counter and treasury from the flip hot path, then benchmark the actual one-pixel and 16-pixel Token-2022 paths across many sections.
5. Add owner fee sharing and versioned section policies. Policies cannot change during a live campaign and must survive a section sale.
6. Add owner/sponsor deposits, entry payments, budget exhaustion, cancellation, and refund rules.
7. Add colour events and the indexed eight-colour canvas without attaching token payouts.
8. Independently review and implement one allowlisted pool distribution policy, then test Sybil splitting, owner farming, replay, equivocation, expiry, sale, seal, and indexer recovery.
9. Run the economy on devnet before any token or reward code is eligible for mainnet.

## Mainnet gates

Mainnet remains blocked until all of the following are recorded:

- exact Token-2022 extension set, mint address, `decimals = 0`, supply, vault balance, revoked authorities, and reproducible transaction signatures;
- an independent program and token-economics review;
- evidence that section allocations always reconcile to the vault;
- an explicit decision on transferability, exchange/liquidity support, legal presentation, and tax/accounting treatment;
- measured staging data supporting the issuance curve, claim price, and owner fee share;
- at least one useful BIT sink operating end to end; and
- product copy that calls rewards finite game credits and never guaranteed profit, yield, or redeemable value.
