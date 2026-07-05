# Local, Steam, Reconnect, And Packet Loss Test Plan

Date: 2026-07-03
Status: slice TEST-03 runbook

## Purpose

This runbook keeps the foundation test order clear while backend, Godot
interface, and Steam-preparation slices continue without fresh operator Go. It
defines what local automation can do now, what future reconnect and packet-loss
slices must prove, and where live Steam remains gated.

## Execution Order

1. Validate plans and foundation gates.
2. Run the local server smoke.
3. Run Godot/Rust protocol fixture checks.
4. Run Godot client adapter and snapshot-render smokes.
5. Run the local/mock lobby facade check.
6. Run the same-workstation multi-client harness.
7. Add reconnect tests only after `RECON-01` defines session rebind state.
8. Add packet-loss and jitter tests only after `LOSS-01` defines ack/resend and
   idempotency behavior.
9. Run live Steam checks only after explicit fresh Go.

## Current Local Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_local_multiclient_harness.ps1
```

Expected final line:

```text
local_multiclient_harness status=ok clients=2 mode=local_mock
```

This command is safe for repo-only work. It runs same-workstation checks and
does not call live Steam, open a public server, or start gameplay.

## Parallel Work Rules

The JSON plan remains the source of truth. Parallel workers may proceed only
when their write scopes do not overlap.

| Lane | Safe next work | Paths | Blocks |
| --- | --- | --- | --- |
| Backend protocol | `RECON-01`, then `LOSS-01` | `server/`, `protocol/` | Real Steam auth, gameplay authority |
| Godot interface | Reconnect/loss adapter checks after backend contract exists | `client/godot/`, `scripts/` | Final UI/design, live Steam |
| Steam preparation | `STEAM-01` after reconnect contract evidence | `docs/steam/`, `docs/runbooks/` | Real AppID, tickets, two-machine smoke |
| Evidence and build | `BUILD-02`, `BUDGET-02`, perf/report wiring | `.github/`, `scripts/`, `docs/evidence/`, `docs/perf/` | Release-candidate claims |

If two workers need the same file, one waits or the coordinator narrows the
slice. No worker may mark a slice done without implementation or explicit
docs-only completion, checks, documentation, and plan handoff.

## Reconnect Test Shape

Reconnect is local/mock until live gates open.

Required evidence for future reconnect slices:

- stable `player_session_id` survives connection rebind,
- old `connection_id` can be replaced during a grace period,
- stale commands from the old connection are rejected or ignored,
- reconnect receives a full snapshot before delta resume,
- Godot adapter exposes reconnect started/restored status without becoming
  authoritative.

`RECON-01` must define the data model before any harness restart case is called
done. `RECON-03` may then add a restart-client harness case.

## Packet Loss Test Shape

Packet loss tests stay synthetic and local.

Required evidence for future loss slices:

- command sequence numbers are monotonic per session,
- duplicate reliable commands are idempotent,
- ack/resend state is bounded,
- stale snapshots do not roll the client world backwards,
- operator-visible logs are redacted and low-volume.

`LOSS-01` must define command ack/resend semantics before `LOSS-02` introduces
loss or jitter simulation.

## Steam Test Shape

Safe without fresh Go:

- lobby metadata shape,
- endpoint advertisement model,
- local/mock identity handoff,
- Spacewar-safe configuration placeholders,
- redacted operator wording.

Requires explicit fresh Go:

- real Steam API calls,
- real auth/session ticket validation,
- real AppID release/playtest assumptions,
- two-machine live Steam smoke,
- release-candidate confidence claims.

## Go / No-Go Language

For the canonical operator language across all Foundation slices, see
`docs/runbooks/operator-go-no-go.md`.

`Go`: local/mock checks pass, no live Steam action is required, no gameplay
authority is added, and documentation/evidence is updated.

`Deferred`: a live Steam, real AppID, two-machine, design, or release-candidate
claim is needed.

`No-Go`: a test would persist secrets, raw Steam tickets, private account data,
public-network state, or gameplay authority outside the Rust server.

## Handoff Fields

Each future reconnect, packet-loss, or Steam-preparation slice records:

- slice ID and class,
- commands run and final status line,
- touched paths,
- whether local/mock, reconnect, loss, or Steam-gated evidence changed,
- open gates,
- next safe slice.
