# Cross-Side Fixture Validation Plan

Date: 2026-07-03
Slice: `PROTO-03`
Status: repo-only validation plan

## Purpose

Rust and Godot must interpret the same protocol fixture bytes before live
networking expands. This plan defines the local validation matrix for
`protocol_v0` binary fixtures and the evidence required when adding or changing
fixtures.

Canonical binary fixtures remain in:

```text
protocol/fixtures/
```

The machine-readable validation matrix lives in:

```text
tests/fixtures/cross-side-fixture-validation-matrix.json
```

## Required Validation Sides

| Side | Command | Required result |
| --- | --- | --- |
| Rust | `cargo test protocol_v0` | Fixture bytes decode through `PacketHeader::decode` and server assertions. |
| Godot | `powershell -ExecutionPolicy Bypass -File scripts\run_godot_fixture_check.ps1` | `godot_fixture_check status=ok fixtures=3` |
| Plan/docs | `powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1` | Plan remains valid JSON-only source of truth. |

If Godot or Cargo cannot run in the local environment, the slice handoff must
record the missing side as an open check deferral. Do not treat a one-sided
fixture check as cross-side evidence.

## Fixture Matrix

| Fixture | Rust expectations | Godot expectations |
| --- | --- | --- |
| `protocol_v0_server_hello_accept` | message type, payload length, connection/session IDs, server sequence, ack, tick, server tick rate | same decoded header fields plus server hello payload summary |
| `protocol_v0_command_ready_batch_ok` | message type, payload length, connection/session IDs, client sequence, ack, command count, command ID, target tick | same decoded header fields plus command count, command ID, target tick |
| `protocol_v0_snapshot_full_minimal_ok` | message type, payload length, connection/session IDs, server sequence, ack, snapshot ID, baseline, entity count, entity fields | same decoded header fields plus snapshot ID, entity count, first entity summary |

## Add/Change Rule

Adding or changing a protocol fixture requires:

- binary fixture file,
- JSON descriptor file,
- matrix row in `tests/fixtures/cross-side-fixture-validation-matrix.json`,
- Rust fixture assertion or explicit Rust check deferral,
- Godot fixture assertion or explicit Godot check deferral,
- privacy check confirming no secrets, Steam tickets, private account data, raw
  provider output, or real AppID assumptions,
- JSON plan handoff.

## Non-Claims

This plan does not add live networking, Steam auth, public endpoint validation,
gameplay payload semantics, final transport selection, or release-candidate
evidence. It only defines local fixture parity expectations.
