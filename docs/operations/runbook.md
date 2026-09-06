# Operations runbook

This runbook covers a Bitflip web/server deployment. It does not authorize a Solana program upgrade or mainnet key ceremony; use [mainnet-ceremony.md](../mainnet-ceremony.md) for those operations.

## Release record

For every deployment, record:

- release owner and UTC window;
- git commit and immutable container digest;
- environment, game index, program address, RPC origin, metadata origin, and Merkle tree public key;
- migration identifier and backup checksum;
- mobile/web build numbers;
- canary wallet public key, transaction signatures, and asset ID;
- rollback decision and incident channel.

Never record database passwords, signing keys, seed phrases, or the operator private key.

## Before deployment

1. Require a green CI run for the exact commit. Verify generated code is clean and all release-mode builds completed.
2. Build the container once. Promote that digest from staging to production; do not rebuild between environments.
3. Validate every release variable listed in [release-readiness.md](../release-readiness.md). Resolve secrets from the deployment secret manager, not a checked-in file or shell history.
4. Create a custom-format PostgreSQL backup with `setup/scripts/postgres_backup.sh`. Copy the dump and checksum to encrypted storage with retention and access controls.
5. Restore that exact dump into an isolated database with `setup/scripts/postgres_restore_verify.sh`. A backup is not considered valid until this succeeds.
6. Rehearse pending migrations against the restored copy. Serverpod must start, `/startupz` and `/readyz` must pass, and the application integration suite must complete before touching the live database.
7. Confirm ingress has a distributed source/global challenge rate limit, request-size limit, TLS, and no route to the Insights port.

## Staging rollout

1. Deploy the immutable image with one replica and zero public traffic. Keep exactly one mint-capable process for the entire beta; the current operator gate is process-local.
2. Wait for `/startupz`; then require `/livez` and `/readyz`. Readiness includes PostgreSQL and a bounded Solana `getSlot` request.
3. Run the `staging verification` GitHub workflow from outside the hosting network. For a sealed canary section, provide both optional indices so its metadata and SVG are checked too.
4. Exercise wallet cancellation, insufficient funds, stale revision, and RPC failure. Confirm each path preserves queued work and does not report success.
5. Run a bounded devnet canary: connect, claim, flip, confirm, seal, authorize mint, and verify both the cNFT and Bitflip receipt on chain. Record signatures and the asset ID.
6. Hold staging for the soak period required by the Serverpod ADR. Review errors, latency, RPC usage, database connections, and operator saturation.

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
