# Godot/Rust Fixture Contract

Date: 2026-07-03
Status: slice GNET-02 fixture contract

## Purpose

Godot and Rust must decode the same protocol_v0 fixture bytes into the same
header fields before live networking expands. Binary files in
`protocol/fixtures/` are canonical. JSON descriptor files explain the expected
fields and privacy constraints.

`docs/protocol/cross-side-fixture-validation-plan.md` and
`tests/fixtures/cross-side-fixture-validation-matrix.json` define the PROTO-03
matrix for keeping the Rust and Godot expectations aligned.

## Current Fixtures

| Fixture | Message | Required checks |
| --- | --- | --- |
| `protocol_v0_server_hello_accept.bin` | `server_hello` | type, length, connection/session IDs, server sequence, tick |
| `protocol_v0_command_ready_batch_ok.bin` | `client_command_batch` | type, length, client sequence, ack sequence, target tick |
| `protocol_v0_snapshot_full_minimal_ok.bin` | `server_full_snapshot` | type, length, server sequence, ack sequence, snapshot tick |

## Rust Check

```powershell
C:\Users\nkatz\.cargo\bin\cargo.exe test
```

Rust validates fixtures through `server/src/protocol.rs` and the integration
tests in `server/src/lib.rs`.

## Godot Check

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_godot_fixture_check.ps1
```

Godot validates fixtures through `client/godot/scripts/net/ProtocolCodec.gd`
and `client/godot/scripts/tests/protocol_fixture_check.gd`.

Expected terminal line:

```text
godot_fixture_check status=ok fixtures=3
```

## Rules

- Fixture bytes must remain deterministic and local.
- Fixture descriptors must not contain secrets, Steam tickets, account data, or
  raw provider/session tokens.
- Godot may decode packet facts into dictionaries, but it must not instantiate
  scenes, Resources, or Nodes from packet bytes.
- Adding a fixture requires both Rust and Godot checks, or an explicit plan
  handoff that marks the missing side as an open gate.
