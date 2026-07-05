# Backpressure Operator States

Date: 2026-07-03
Slice: `LOADSHED-04`
Status: repo-only operator contract

## Purpose

This runbook defines the operator-visible state words for local backpressure and
slow-client handling. The server remains authoritative for every overload
decision. Godot may display these states for diagnostics, but it must not use
them to accept commands, hide authoritative corrections, or decide gameplay
outcomes.

## State Words

| State | Server meaning | Operator meaning | Godot display rule |
| --- | --- | --- | --- |
| `normal` | Client is within current caps. | No overload response is active. | Optional neutral status only. |
| `degraded` | Server is reducing optional snapshot detail or rate. | Client is slow or bandwidth constrained; watch backlog and bandwidth counters. | Show as read-only network quality degradation. |
| `commands_limited` | New commands are being dropped or rejected before simulation mutation. | Input pressure, pending reliable commands, or log volume is above policy limits. | Show as read-only command admission warning; do not retry-spam. |
| `disconnect_pending` | Server has crossed hard resend/backlog safety bounds and is ending the session. | Local evidence should preserve redacted reasons and last policy decision. | Show as server disconnect in progress; wait for reconnect/full-snapshot flow. |
| `disconnected` | Session is no longer active after overload policy. | Resume requires a server-marked full snapshot before delta state is trusted again. | Clear stale client world state only when the reconnect contract says to. |
| `blocked_live_validation` | Live or release-grade overload evidence is requested without fresh Go. | Do not run live Steam, two-machine, public-network, long-soak, or release-candidate validation. | No in-game behavior; this is a runbook/gate state only. |

## Safe Without Fresh Go

The following remain covered by standing Safe Go:

- local JSON overload case catalogs,
- Rust `load_shed` unit tests,
- local smoke scripts that do not open public network state,
- mock/local Godot status wiring needed for diagnostics,
- docs, evidence, and JSON plan handoffs.

## Fresh Go Required

Fresh operator Go is required before any of these actions:

- real Steam auth/session or AppID validation,
- two-machine Steam overload smoke,
- public network or deploy validation,
- long-running soak used as release evidence,
- release-candidate overload signoff,
- final transport, ECS, rendering, gameplay, or balance decisions.

## Evidence To Capture

When a local overload smoke or future bounded run is reported, capture:

- state word,
- action (`accept`, `degrade`, `drop_command`, or `disconnect`),
- reason list,
- snapshot mode,
- command admission,
- optional diagnostics status,
- redacted client/session label,
- command or script that produced the result,
- final status line.

Do not capture secrets, Steam tickets, private account data, raw provider output,
or private network details.

## No-Go Conditions

Mark the path `No-Go` or `Deferred` if a run would:

- require live Steam, a real AppID, public network mutation, deployment, or a
  two-machine smoke without explicit fresh Go,
- persist secrets or raw live provider output,
- let Godot override server overload decisions,
- claim release, scale, live-network, or gameplay readiness from local
  overload smoke alone,
- proceed without a matching evidence record and JSON handoff.

## Related Artifacts

- `docs/protocol/backpressure-rate-limits.md`
- `docs/architecture/backpressure-slow-client-policy.md`
- `docs/evidence/overload-smoke-evidence.md`
- `tests/overload/load-shed-overload-cases.json`
- `scripts/run_overload_smoke.ps1`
