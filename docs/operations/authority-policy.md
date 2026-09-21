# Authority, key custody, and incident policy

This document answers the questions the [mainnet ceremony](../mainnet-ceremony.md) depends on but does not itself specify: who holds each key, where it lives, how it rotates, and what happens when one is suspected compromised. It is a prerequisite for mainnet, not a description of the current deployment.

The 2026-09-05 internal audit and the 2026-09-21 independent review both found that most of this protocol's residual risk sits here rather than in the program. The incident record agrees: Drift lost ~$285M in April 2026 with no code bug, entirely through signing-ceremony and authority-custody failures.

## Authority inventory

| Authority                     | Held by                          | Controls                                                                          | Rotation                                                 |
| ----------------------------- | -------------------------------- | --------------------------------------------------------------------------------- | -------------------------------------------------------- |
| Program upgrade               | Multisig (default: Squads V4)    | Deploying new bytecode: every program rule                                        | Multisig membership change                               |
| Config `authority`            | Ops signer, multisig-recommended | `UpdateConfig`, `ProposeAuthority`, `WithdrawProtocolFees`, `ConfigureBitCustody` | `ProposeAuthority` + `AcceptAuthority`                   |
| Config `treasury`             | Plain address (no signing)       | Receives claim price and swept protocol fees                                      | `UpdateConfig`                                           |
| Config `collection_authority` | Operator key (hot, server)       | `RecordSectionMint` — attests cNFT receipts                                       | `UpdateConfig`                                           |
| BIT mint / freeze             | **Revoked, permanently**         | Nothing                                                                           | Not rotatable; enforced on-chain at custody registration |
| Bubblegum tree delegate       | Operator key (hot, server)       | Minting leaves in the private tree                                                | Re-delegate via Bubblegum                                |

### The bootstrap-authority window is the sharpest edge

`InitializeConfig` sets `authority`, `treasury`, and `collection_authority` to the compiled-in `BOOTSTRAP_AUTHORITY` (`B8yibwGRtrnp55T8uRyt19J6KTTRAZMTD9DgEgjQqVNi`, `bitflip_program/src/lib.rs`) in a single instruction. Between deployment and rotation, **one private key can redirect the treasury, take over the config, and forge mint attestations**. Nothing on-chain prevents this; only custody discipline does.

Requirements:

1. Generate the bootstrap keypair on the offline machine or HSM named in the ceremony. It must never exist on a networked host, in CI, or in a chat client.
2. Treat the ceremony's rotation step as the _first_ production action after deployment, not a later cleanup.
3. Immediately after rotation, use `UpdateConfig` to move `treasury` and `collection_authority` off the config authority so that no single key ever holds two roles again.
4. Once rotated, the bootstrap key has no purpose. Revoke or destroy it and record that in the release record.

### Sign-off (required before mainnet)

The following are defaults, not decisions. The named release owner must confirm or replace each one and record the result in the release record:

- [ ] Upgrade authority behind a multisig — platform: `__________` — threshold: `______` of `______`
- [ ] Execution timelock on upgrades: `__________` (recommend 24h minimum; the Drift council had none)
- [ ] Signer roster disclosed, with hardware-wallet or HSM-backed keys for every signer: `__________`
- [ ] Emergency-pause path pre-approved as a template so it exists before it is needed: `__________`
- [ ] Bootstrap key custody documented (who, where, backup): `__________`
- [ ] Post-rotation `UpdateConfig` splitting treasury/collection off the config authority: `__________`

A 2-of-5 council with no timelock is one phished signer away from a total loss and does not satisfy this policy.

## Key custody rules

- Production keys are generated on an offline machine or managed HSM/KMS and are never reused from development or repository material. `SECURITY.md` records that a legacy `setup/keypairs/admin.json` key was committed and must be treated as permanently compromised.
- Private bytes never enter chat, CI logs, shell history, issue trackers, or git. `gitleaks` enforces the mechanical part of this in the pre-commit hook and in `audit:security`.
- Backup and recovery: the ceremony references a "documented recovery policy" that had not been written. Until one exists, treat every key as unrecoverable and prefer keys that can be re-derived or re-issued:

  1. Write down, for each authority in the inventory above, where the backup lives and who can retrieve it.
  2. Store backups in two access-controlled locations, at least one offline, sealed, and inventoried.
  3. Test one restore per key generation cycle and record the date. An untested backup is not a backup.
- Offboarding is revocation. When a person with signing power leaves, every key they could reach is rotated before their last day. Pump.fun's ~$1.9M loss (May 2024) was a former employee with retained withdrawal authority.

## Operator key (collection authority and tree delegate)

The operator key is the one production key that is deliberately hot: the server holds it to mint compressed NFTs. It is therefore the most realistic compromise target, and its blast radius is the authenticity of the collection rather than the treasury.

Controls:

- The key lives only in the deployment secret manager, never in a checked-in file.
- The server zeroes the decoded key bytes after each mint and never logs key material.
- Public mint ingress is rate limited at the edge as well as in-process.
- Alerts fire on operator balance and tree capacity, which are the observable failure modes.

Rotation procedure, to be exercised at least once before mainnet:

1. Generate the replacement key on the offline machine or HSM.
2. `UpdateConfig` with the new `collection_authority`.
3. Re-delegate the Bubblegum tree to the new operator.
4. Deploy the new `BITFLIP_OPERATOR_PRIVATE_KEY` through the secret manager and restart the server.
5. Confirm one live mint succeeds, then destroy the old key and record the rotation.

Compromise detection matters as much as prevention: because a compromised operator can forge receipts that look identical to legitimate ones, alert on `RecordSectionMint` for any asset or leaf that does not correspond to a server-issued challenge.

## Incident response

`docs/operations/runbook.md` covers rollback. This section covers the broader playbook, because every one of these steps depends on preparation done before the incident.

1. **Detect.** Alert on ProgramData and authority changes (Yellowstone gRPC or an equivalent account subscription) — authority movement is the loudest signal in this ecosystem and almost nothing watches it by default. Also alert on anomalous `WithdrawProtocolFees` and `RecordSectionMint` activity.
2. **Pause.** Disable transaction entry points at the app and ingress layers, and use the pre-approved emergency-pause upgrade if the program itself must stop. Never attempt to roll back chain history.
3. **Contain.** The BIT mint has no freeze authority and no permanent delegate by design, so the protocol cannot freeze or confiscate BIT. Do not assume validator intervention: Solana has no precedent for it.
4. **Rotate.** Any key that may be touched rotates immediately — config authority first (via propose/accept), then operator, then upgrade authority.
5. **Trace and engage.** Record signatures and timeline, engage tracing and law enforcement, and consider an on-chain bounty message. Bridging out is the point of no return for recovery.
6. **Disclose.** Publish a post-mortem, update `SECURITY.md`, and re-audit the changed paths. Euler and Curve both shipped losses in code paths added after clean audits; an incident is not over when the bleeding stops.

Record for every incident: detection time, first affected signature, decisions and their owners, and the follow-up audit. Contacts and the rollback decision owner are recorded per deployment in the release record.
