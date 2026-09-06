# Observability and alert contract

Production logs are emitted as JSON to stdout and persisted by Serverpod. The hosting platform must ship stdout to access-controlled, encrypted log storage before public traffic is enabled. Retain security and mint-operation logs according to the approved privacy policy; never log authorization signatures, private keys, database URLs, or seed material.

## Required signals

Every mint log includes `operation`, `gameIndex`, `sectionIndex`, `outcome`, and `durationMs`. Infrastructure telemetry must additionally expose:

- request count, status, and p50/p95/p99 duration by route;
- challenge rate-limit rejections by source class, without raw IP labels;
- operator in-flight/rejected work and mint outcomes;
- Solana RPC request count, latency, timeout, retry, and error class;
- PostgreSQL connections, saturation, transaction duration, storage, and backup age;
- container restarts, CPU, memory, disk, and probe status;
- on-chain operator SOL balance and private Merkle tree capacity.

Wallet addresses, signatures, asset IDs, and IP addresses must not be metric labels.

## Paging alerts

Page the release owner when any condition holds:

- readiness fails for 2 consecutive minutes or more than one replica is unready;
- confirmed mint failure rate exceeds 2% over 10 minutes with at least 10 attempts;
- p95 mint duration exceeds 75 seconds for 10 minutes;
- operator busy rejections exceed 10% over 5 minutes;
- Solana RPC timeout rate exceeds 5% over 5 minutes;
- database connections exceed 80% of the pool for 10 minutes;
- no successful production backup exists in 25 hours, or the latest restore drill is older than 30 days;
- operator balance falls below the documented two-day fee budget;
- remaining Merkle tree capacity falls below the larger of 5% or seven days of forecast mints.

Create a ticket, rather than a page, for rising client errors, metadata latency, or isolated wallet incompatibility that does not block transactions.

## RPC budget

- readiness: one cached `getSlot` per replica every five seconds at most, three-second deadline;
- chain reads: ten-second endpoint deadline and twelve-second service default;
- mint submission and confirmation: ninety-second endpoint deadline, fresh-state retries capped by `BITFLIP_MINT_MAX_ATTEMPTS`;
- application refresh: one revision-aware poll every twelve seconds while visible;
- ingress and provider quotas must reserve at least 25% headroom for canary and recovery traffic.

Review budgets after every load test and incident. Raising timeouts is not a substitute for reducing saturation or failing over.
