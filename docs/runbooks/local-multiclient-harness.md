# Local Multi-Client Harness

Date: 2026-07-03
Status: slice TEST-01 local harness scaffold

## Purpose

The local harness gives the project one command that exercises the current
server and Godot interface smokes for multiple local client profiles. It is not
a real networked match yet and does not call live Steam.

## Config

`config/local-multiclient-harness.json`

Current profiles:

- `client-1`
- `client-2`

Both use synthetic `local-abc123` lobby identity and run the same local checks.
The local lobby facade derives a redacted session ID in the
`local-session-{lobby}-{player}` shape documented by
`docs/protocol/local-identity-handshake.md`.

## Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_local_multiclient_harness.ps1
```

Expected final line:

```text
local_multiclient_harness status=ok clients=2 mode=local_mock
```

## What It Runs

- server smoke,
- Godot/Rust protocol fixture check,
- Godot client adapter check,
- snapshot-render smoke,
- local/mock lobby facade check.
- restart-client reconnect case: runs the client adapter check before and
  after a simulated local client restart and expects the adapter to reach
  `reconnect=delta_resume`.

Expected restart line:

```text
local_multiclient_harness reconnect_restart status=ok profile=client-1
```

## Not Yet

- no real socket server,
- no live Steam,
- no two-machine smoke,
- no gameplay loop,
- no authoritative match admission beyond local/mock facade shape.

For the ordered local, Steam-gated, reconnect, and packet-loss test matrix, see
`docs/runbooks/local-steam-reconnect-loss-test-plan.md`.

## Stop Rules

Stop and gate the harness if it would require live Steam, real AppID, public
network mutation, secrets, destructive Git, or gameplay implementation.
