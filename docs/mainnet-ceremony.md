# Mainnet deployment ceremony

This ceremony requires two people: an operator and an independent witness. Do not accept real funds until every item is complete and the signed record is stored outside the repository.

## Before the ceremony

1. Freeze the reviewed commit and record its full Git SHA.
2. Resolve or explicitly accept every independent program/operator audit finding.
3. Run `audit:security`, `lint:all`, `test:all`, and all release builds from a clean checkout.
4. Build the SBF artifact twice in isolated clean environments with `build:program`. Confirm the recorded feature set contains `bpf-entrypoint` and does **not** contain `sbf-test-authority`; that feature embeds a public test key and must never appear in a deployable artifact. Record both SHA-256 values and stop if they differ.
5. Verify the deployed program against the reviewed source with `solana-verify` and preserve its output.

## Keys and authorities

1. Generate fresh deployment, bootstrap authority, collection authority, operator, and tree keys on an offline machine or managed HSM/KMS.
2. Back them up using the documented recovery policy. Never copy private bytes into chat, CI logs, shell history, or git.
3. Create a private Bubblegum V1 tree and delegate it only to the operator.
4. Initialize Bitflip with the intended treasury, prices, collection authority, game index, and unlock policy. Independently read every account back.
5. Create BIT as a Token-2022 mint with no extensions and zero decimals. Mint exactly 26,843,545,600 BIT into the canonical config-PDA associated account, then revoke mint authority. Use no freeze authority. Independently verify mint supply, decimals, extensions, authorities, reserve owner, reserve mint, and reserve balance before registering custody.
6. Register the mint and launch reserve once with `ConfigureBitCustody`. Fund only the initial section vault; later section claimants pay their own vault rent. Record every transaction signature and independently reconcile the reserve reduction.
7. Transfer or revoke the program upgrade authority according to the policy below. Use Bitflip's propose/accept flow to rotate temporary bootstrap authority to the production authority.

## Upgrade-authority decision

Choose and record exactly one policy before deployment:

- Immutable: revoke the program upgrade authority after the canary period.
- Governed: place it behind a disclosed multisig with a review delay, signer roster, quorum, incident process, and public upgrade announcement policy.

A single hot-wallet upgrade authority is not an acceptable production policy.

## Canary and opening

1. Deploy Serverpod with validated production configuration and restricted Insights access.
2. Confirm health, readiness, structured logs, metrics, alerts, RPC budgets, and database backup/restore.
3. Fund canary wallets only with the bounded amount needed for the test.
4. Perform one claim, multi-pixel flip, permanent seal, and compressed mint.
5. Confirm the transaction signatures, on-chain section receipt, tree leaf, metadata, image, and marketplace display.
6. Exercise the reconciliation procedure as a dry run.
7. The operator and witness sign the release record before public access opens.

## Release record

Record at minimum:

- UTC date and participants
- Git commit and independent audit reference
- SBF SHA-256 values and reproducibility result
- deployed program ID and deployment signature
- upgrade-authority policy and address, or revocation signature
- config, treasury, collection authority, BIT mint, launch reserve, initial section vault, game, and tree addresses
- Serverpod image digest and database migration version
- canary wallet, transaction signatures, asset ID, and outcome
- rollback decision owner and incident contacts
