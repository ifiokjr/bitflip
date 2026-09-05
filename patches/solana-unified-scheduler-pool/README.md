# Solana 2.1 Rust compatibility patch

This is `solana-unified-scheduler-pool` 2.1.16 with the two `Vec::extract_if`
calls updated for Rust 1.87 and newer. The standard library stabilized that
method with an explicit range argument after Solana 2.1 shipped its own
polyfill.

Remove this vendored patch when the Steel dependency no longer pins the
workspace to Solana 2.1.
