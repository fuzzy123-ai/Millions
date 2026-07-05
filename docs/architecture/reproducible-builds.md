# Reproducible Build Evidence

Date: 2026-07-03
Status: slice BUILD-03 deterministic evidence contract

## Purpose

Fixture, replay, and performance evidence needs a stable build identity before
release packaging exists. This contract defines the first repo-only build
evidence shape for the Rust server and protocol fixtures.

## Server Evidence

`server/src/build_info.rs` exposes `BuildEvidence::foundation()` with:

- Cargo package name,
- Cargo package version,
- protocol version,
- fixture descriptor count,
- replay format version,
- deterministic `build_id`.

The current build ID shape is:

```text
millions-server-0.1.0-protocol_v0-fixtures_3-replay_v1
```

## Forbidden Inputs

Build evidence used by fixtures, replay, perf, or release preparation must not
include:

- wall-clock timestamps,
- private machine paths,
- user names,
- process IDs,
- random values,
- live Steam/provider data,
- secrets or tokens.

## Handoff

Later build/release slices may add artifact hashes and package metadata, but
they must preserve this rule: evidence that compares fixtures or replay output
uses stable IDs unless the slice explicitly documents why a value is expected to
vary.
