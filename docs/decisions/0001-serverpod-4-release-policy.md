# ADR 0001: Serverpod 4 release policy

- Status: accepted for beta; blocked for mainnet
- Date: 2026-09-06
- Owner: @ifiokjr

## Context

Bitflip targets Serverpod 4's generated protocol, health checks, embedded PostgreSQL test tooling, and application layout. The repository currently pins `4.0.0-rc.2` exactly. Serverpod 3.4.13 is the latest stable line, while Serverpod 4 remains a release candidate at the time of this decision. Moving this codebase back to 3.x would be a migration rather than a safe dependency downgrade.

## Decision

The pinned release candidate is accepted only for the web/devnet and native/devnet beta milestones. Mainnet is blocked until one of these conditions is recorded in a new ADR:

1. Serverpod 4 reaches a stable release, Bitflip upgrades to it, and the complete release suite passes; or
2. the release owner accepts the RC risk after a minimum 24-hour staging soak, migration and restore rehearsal, load test at twice the expected peak request rate, and review of every newer Serverpod 4 release note.

Dependency constraints stay exact. Automated dependency updates may propose a change, but cannot merge without generated-code, integration, migration, backup/restore, and staging smoke evidence.

## Consequences

This keeps the beta moving without disguising prerelease framework risk as production readiness. It also makes the Serverpod decision an explicit mainnet gate instead of an undocumented dependency accident.
