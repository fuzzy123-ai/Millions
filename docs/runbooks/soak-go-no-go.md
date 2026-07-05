# Soak Go And No-Go

Date: 2026-07-03
Slice: SOAK-04
Status: operator language for soak execution

## Purpose

This runbook defines the decision language for long-running soak and chaos
work. It keeps local contract checks moving while real long sessions, live
Steam, two-machine tests, and release-candidate stability remain explicit
operator decisions.

## Decision States

| State | Meaning | Allowed action |
| --- | --- | --- |
| `Soak Safe Go` | Repo-only or safe-offline preparation is allowed. | Edit scenarios, emitters, scripts, docs, and run short local contract smokes. |
| `Soak Local Go` | The operator explicitly allows a bounded local soak duration. | Run only the named local scenario, duration, machine, and artifact path. |
| `Soak Live Go` | The operator explicitly allows a bounded live/two-machine soak. | Run only the named live scenario with stated Steam/AppID/network/data-retention boundaries. |
| `Soak Deferred` | A scenario is valid but missing a required operator, live, tool, or evidence condition. | Keep safe preparation moving; do not run the deferred scenario. |
| `Soak Blocked` | A required check failed or the run would be unsafe. | Stop that soak path and record evidence. |
| `Soak No-Go` | The requested run must not happen. | Do not run it; preserve the repo state and document why. |

## Safe Without Fresh Go

These actions remain covered by standing Safe Go:

- edit `tests/soak/soak-scenarios.json`,
- run `scripts/run_soak_smoke.ps1`,
- run `scripts/run_soak_smoke.ps1 -SkipGodot`,
- add local/mock metrics and dry-run harness code,
- update redacted local evidence, runbooks, and JSON plan handoffs.

## Requires Fresh Go

These actions require explicit fresh operator Go:

- any 15, 60, or 120 minute soak execution,
- any live Steam or real AppID participation,
- `soak_live_two_machine_60m`,
- public network exposure,
- writing release-candidate stability evidence,
- persisting live session output beyond redacted summaries,
- changing retention rules for logs, packets, tickets, or account data.

## Required Go Record

Use this shape before a bounded soak run:

```text
Soak decision:
- Scenario:
- Duration:
- Machine(s):
- Build ID:
- Mode: local | live
- Steam/AppID boundary:
- Network boundary:
- Artifact path:
- Data retention:
- Stop conditions:
- Operator decision: Soak Local Go | Soak Live Go | Soak Deferred | Soak Blocked | Soak No-Go
- Notes:
```

If any field is missing, the run is `Soak Deferred`.

## Result Record

Use this shape after a soak attempt:

```text
Soak result:
- Scenario:
- Planned duration:
- Completed duration:
- Build ID:
- Machine label:
- Metrics artifact:
- Performance history row:
- Required metric groups: pass | fail | blocked
- Memory trend: pass | fail | blocked
- Queue/backlog trend: pass | fail | blocked
- Reconnect/shutdown: pass | fail | blocked
- Redaction audit: pass | fail
- Budget result: pass | fail | blocked
- Operator decision: accept | rerun | reject | defer
- Notes:
```

Green local contract smokes do not imply a soak result. A result exists only
after the named duration was actually attempted and recorded.

## No-Go Conditions

Stop or refuse the run if:

- the requested scenario is live-gated and lacks `Soak Live Go`,
- secrets, Steam tickets, private account data, raw provider payloads, or raw
  unapproved packet dumps would be persisted,
- public network mutation is not bounded,
- required metric groups are missing,
- memory, queue, resend, or log growth is unbounded,
- shutdown is not graceful,
- release-candidate stability is claimed from local-only or blocked evidence.
