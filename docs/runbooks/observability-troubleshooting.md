# Observability Troubleshooting

Date: 2026-07-03
Slice: GOBS-03
Status: troubleshooting runbook for planned debug/observability surfaces

## Purpose

Use this runbook when local multiplayer, snapshots, render state, performance,
Steam mock handoff, or logging evidence looks wrong. It is designed for
repo-only/local checks and does not require live Steam, real AppID, external
playtest, or gameplay implementation.

## First Checks

1. Run `powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1`.
2. Run `powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1`.
3. Run the focused local smoke for the touched system.
4. Inspect structured log/counter fields using the schema in
   `docs/observability/log-event-schema.md`.
5. Confirm no secret, Steam ticket, real AppID assumption, private account data,
   or raw provider payload was persisted.

## Symptom Matrix

| Symptom | First fields to inspect | Likely boundary | Next safe action |
| --- | --- | --- | --- |
| Commands stay pending | `pending_command_count`, `client_seq`, `ack_seq`, `last_command_id` | command reliability | run client adapter and server reliability tests |
| Duplicate command effect suspected | `command_id`, `player_session_id`, `commands_accepted`, `commands_rejected` | server idempotency | run Rust reliability/replay tests |
| Snapshot appears stale | `server_tick`, `snapshot_id`, `snapshot_age_ms`, `buffered_snapshot_count` | snapshot buffer/reconnect | run snapshot render and reconnect checks |
| Entity count mismatch | `authoritative_entity_count`, `visible_entities`, `render_proxy_count` | client world/render adapter | run render batch and snapshot render checks |
| Render load looks high | `render_proxy_count`, `render_batch_count`, `godot_render_update_ms`, `godot_frame_ms` | render/perf | run render stress smoke and perf budget checks |
| Reconnect does not restore | `connection_id`, `player_session_id`, `phase`, `full_snapshot_required` | reconnect/session | run reconnect/loss harness checks |
| Steam mock handoff looks unsafe | `identity_mode`, `endpoint_mode`, `endpoint_epoch`, `ready_epoch`, `redaction_events` | Steam facade | run Steam lobby facade check and verify redaction |
| Logs are too noisy | category, level, `run_id`, repeated event name | observability/loadshedding | lower severity, aggregate, or add bounded counters |
| Potential secret in logs | `redaction_events`, unsafe field class | security/redaction | stop, remove raw value, add redaction test before proceeding |

## Safe Local Commands

Use focused commands first:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_godot_client_adapter_check.ps1
powershell -ExecutionPolicy Bypass -File scripts\run_godot_snapshot_render_smoke.ps1
powershell -ExecutionPolicy Bypass -File scripts\run_godot_render_batch_check.ps1
powershell -ExecutionPolicy Bypass -File scripts\run_godot_lobby_facade_check.ps1
powershell -ExecutionPolicy Bypass -File scripts\run_server_smoke.ps1
powershell -ExecutionPolicy Bypass -File scripts\run_replay_smoke.ps1
```

Use the local smoke orchestrator only when the touched scope spans multiple
systems:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_local_smoke_commands.ps1
```

## Stop Rules

Stop and gate the work if troubleshooting would require:

- live Steam auth, real AppID, or external provider state,
- storing secrets, raw Steam tickets, private account data, or raw provider
  payloads,
- making Godot authoritative for match state,
- implementing gameplay to prove an infrastructure issue,
- destructive Git or broad cleanup,
- claiming scale or release readiness without measured evidence.

## Handoff Template

```text
Observability issue:
- Symptom:
- Fields inspected:
- Focused checks:
- Result: fixed | blocked | deferred | failed
- Redaction status:
- Next safe action:
```
