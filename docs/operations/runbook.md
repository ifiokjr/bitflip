# Operations runbook

This runbook covers a Bitflip web/server deployment. It does not authorize a Solana program upgrade or mainnet key ceremony; use [mainnet-ceremony.md](../mainnet-ceremony.md) for those operations.

## Release record

For every deployment, record:

- release owner and UTC window;
- git commit and immutable container digest;
- environment, game index, program address, RPC origin, metadata origin, and Merkle tree public key;
- colour-indexer launch signature and the latest durable completed sweep;
- migration identifier and backup checksum;
- mobile/web build numbers;
- canary wallet public key, transaction signatures, and asset ID;
- rollback decision and incident channel.

Never record database passwords, signing keys, seed phrases, or the operator private key.

## Before deployment

1. Require a green CI run for the exact commit. Verify generated code is clean and all release-mode builds completed.
2. Build the container once. Promote that digest from staging to production; do not rebuild between environments.
3. Validate every release variable listed in [release-readiness.md](../release-readiness.md). Resolve secrets from the deployment secret manager, not a checked-in file or shell history.
4. Create a custom-format PostgreSQL backup with `setup/scripts/postgres_backup.sh`. Copy the dump and checksum to encrypted storage with retention and access controls. Record the SHA-256 digest in a signed release record kept separately from backup storage.
5. Verify that release record's signature with the release owner's trusted key. Set `BITFLIP_BACKUP_SHA256` to its 64-digit digest and `BITFLIP_BACKUP_FILE` to the selected dump, then run `setup/scripts/postgres_restore_verify.sh`. Never derive the expected digest from the downloaded dump or its adjacent `.sha256` file: an attacker could replace both. The script hashes a private local copy before any database command and restores only that authenticated copy; allow local temporary space for one full dump. A backup is not considered valid until this succeeds.
6. Rehearse pending migrations against the restored copy. Serverpod must start, `/startupz` and `/readyz` must pass, and the application integration suite must complete before touching the live database.
7. Confirm ingress has a distributed source/global challenge rate limit, request-size limit, TLS, and no route to the Insights port.
8. Confirm both public artwork routes use the distributed ingress/CDN limits described below, with no direct origin bypass.

## Public artwork RPC budget

`/metadata` and `/art` share one `SealedSectionArtwork` instance. Each process admits at most four concurrent section loads and 32 new loads in any rolling 30-second window, including failed loads. Excess cold requests return `503`, `Retry-After: 30`, and `Cache-Control: no-store` without queueing or calling RPC. Cached responses and requests sharing an existing load do not spend admission capacity.

The process retains at most 256 rendered sections and 256 negative entries. Missing or invalid on-chain accounts and unsealed sections are remembered for 30 seconds; transport failures remain errors. Rendered entries are evicted in least-recently-used order, and negative entries expire or are evicted oldest first. Successful responses keep their existing immutable CDN headers.

These limits reset at process restart and multiply with replicas. Before public rollout, the release owner must approve the cold-request budget against expected marketplace traffic and the provider quota reserved for minting, indexing, and readiness. Enforce an aggregate limit for both routes at ingress/CDN across all replicas, prevent origin bypass, and verify the deployed policy with a bounded load test. Source tests establish the process bounds; they do not establish a deployed edge policy or provider capacity.

Run restore verification on a disposable PostgreSQL instance isolated from production databases and credentials, with outbound network access blocked. Use a dedicated restore role with only the privileges needed to create and own the disposable database, without superuser, role administration, or membership in production roles. A database name ending in `_restore_verify` is an accidental-deletion guard, not an isolation boundary. The script does not verify release signatures, instance isolation, or role grants; the release owner must verify those prerequisites before supplying the trusted digest. Destroy the instance after rehearsal.

## Staging rollout

1. Deploy the immutable image with one replica and zero public traffic. Keep exactly one mint-capable process for the entire beta; the current operator gate is process-local.
2. Wait for `/startupz`; then require `/livez` and `/readyz`. Readiness includes PostgreSQL and a bounded Solana `getSlot` request.
3. Run the `staging verification` GitHub workflow from outside the hosting network. For a sealed canary section, provide both optional indices so its metadata and SVG are checked too.
4. Exercise wallet cancellation, insufficient funds, stale revision, and RPC failure. Confirm each path preserves queued work and does not report success.
5. Make confirmed colour flips and verify the recurring indexer catches up from its durable cursor. Public `colourCanvas.recordSignature` calls only read the latest indexed canvas; they never fetch or apply transactions. After idle backoff, a new flip can wait up to 30 seconds before the next sweep, plus RPC and catch-up time. Monitor sweep age and indexing failures. Restart the server during a multi-page sweep and prove replay neither loses nor overwrites a newer pixel revision or rewrites an unchanged canvas row.
6. Run a bounded devnet canary: connect, claim, flip, confirm, seal, authorize mint, and verify both the cNFT and Bitflip receipt on chain. Record signatures and the asset ID.
7. Hold staging for the soak period required by the Serverpod ADR. Review errors, latency, RPC usage, database connections, indexer sweep age, and operator saturation.

## Production rollout

1. Freeze program upgrades and configuration changes for the release window.
2. Apply the rehearsed database migration. Database changes must be backward compatible with the previous server image for at least one deployment window.
3. Start one canary server replica. Do not send public traffic until all probes pass.
4. Send only release-owner traffic to the canary. Read existing chain state, load metadata/art, and complete one bounded mainnet journey using the ceremony wallet.
5. Increase traffic to the single mint-capable process in 10%, 50%, and 100% steps. Hold each step for at least ten minutes and check the alerts in [observability.md](observability.md). Do not add replicas until durable single-consumer mint work exists.

## Rollback

- Application or server regression: route traffic back to the previous immutable image and app version. Keep the database at the forward-compatible schema.
- Bad migration: stop rollout. Restore only into a new database, verify it, then cut over; never overwrite the only production database in place.
- RPC provider failure: remove the deployment from readiness or fail over to a pre-approved mainnet RPC endpoint with equivalent data and rate limits.
- Operator failure after submission: inspect the section account and transaction signature before retrying. The on-chain record is authoritative. `mintSection` is idempotent after the section reaches minted state.
- Program regression: disable transaction entry points at the app/ingress layer and follow the upgrade-authority policy. Never attempt to roll back chain history.

## Mint reconciliation

The current beta uses bounded synchronous submission plus idempotent re-entry. After any timeout or process loss:

1. Load the section account at confirmed commitment.
2. If status is minted, compare its asset ID, tree, and leaf index with Bubblegum tree state and return the existing result.
3. If status is sealed and no submitted signature is known, require a fresh wallet authorization before retrying.
4. If a signature exists, resolve it to confirmed, failed, or expired before a new submission. Never submit a second transaction while the first is indeterminate.
5. Alert and quarantine mismatches; do not repair records by writing directly to PostgreSQL.

This is sufficient for a bounded beta, where the user remains present for retry. A durable queue that survives process loss is a mainnet gate. It requires a persisted mint-job model, a single-concurrency worker, submission-signature storage before confirmation, exponential retry, and an operator view. That schema change must be generated and migrated with the Serverpod MCP workflow before mainnet.
