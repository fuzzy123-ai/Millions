# Fuzz And Hardening Tests

Fuzz and hardening tests will cover malformed packets, oversized payloads,
version mismatches, stale commands, replayed commands, and abusive rates.

`protocol-hardening-corpus.json` records the deterministic seed catalog for
`HARDEN-02`. The Rust tests in `server/src/hardening.rs` consume the same case
families without requiring a network listener, Steam, downloads, or a fuzzing
framework.
