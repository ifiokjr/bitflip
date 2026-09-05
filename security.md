# Security

Report suspected vulnerabilities privately to `security@ifiokjr.com`. Do not open a public issue for an exploitable finding.

## Compromised development key

The development key with public address `4z5X2suocz9szaQnSshj2AW8tuLgUVmYUxiW9hhPaRHs` was previously committed to this repository. It is permanently compromised and must never control a deployed program, configuration, treasury, mint, or funded account.

Rewriting or force-pushing Git history does not make exposed key material private again. Production and staging keys must be newly generated and stored outside Git in managed secret storage.
