# Log Event Schema

Date: 2026-07-03
Slice: GOBS-01
Status: observability baseline contract

## Purpose

Millions needs structured, redacted logs that help debug large multiplayer
state without leaking secrets or turning logs into gameplay authority. This
contract defines the first log event shape, categories, severities, redaction
rules, and correlation IDs for Rust server, Godot client, local harnesses, and
Steam mock/preparation flows.

This document does not implement counters, debug overlays, evidence export, or
live provider logging.

## Event Shape

Every structured log event should be representable as one JSON object:

```json
{
  "schema": "millions_log_event_v0",
  "ts": "2026-07-03T00:00:00Z",
  "level": "info",
  "category": "network",
  "source": "server|godot|harness|steam_facade",
  "system": "ServerConnection",
  "event": "connection_state_changed",
  "message": "local connection placeholder active",
  "correlation": {
    "run_id": "local-run-id",
    "client_id": "client-0",
    "connection_id": "connection-1",
    "player_session_id": "local-session-redacted",
    "match_id": "local-match-id",
    "trace_id": "optional-trace-id"
  },
  "tick": {
    "server_tick": 20,
    "client_tick": 20,
    "snapshot_tick": 20
  },
  "fields": {
    "command_seq": 1,
    "ack_seq": 1,
    "visible_entities": 128
  }
}
```

Fields may be omitted when unknown, but logs that claim to explain networking,
reconnect, determinism, performance, or release evidence must include the
relevant correlation fields.

## Categories

| Category | Purpose | Example events |
| --- | --- | --- |
| `lifecycle` | process, scene, startup, shutdown | `server_started`, `godot_scene_ready`, `shutdown_complete` |
| `network` | connection, transport, seq/ack, packet loss | `connection_state_changed`, `packet_dropped`, `ack_advanced` |
| `protocol` | decode/encode, fixture, malformed packet | `packet_decoded`, `packet_rejected`, `fixture_validated` |
| `session` | handshake, reconnect, identity handoff | `session_bound`, `reconnect_started`, `full_snapshot_sent` |
| `command` | client intent, server accept/reject, idempotency | `command_queued`, `command_accepted`, `duplicate_ignored` |
| `snapshot` | full/delta snapshots, AOI, visible sets | `snapshot_built`, `snapshot_applied`, `aoi_delta_emitted` |
| `simulation` | server tick and deterministic sim scaffolds | `tick_completed`, `replay_checksum_recorded` |
| `render` | Godot render adapter and proxy updates | `render_batch_updated`, `proxy_count_changed` |
| `performance` | metrics, budgets, reports, regressions | `budget_failed`, `perf_report_written` |
| `steam_facade` | mock/local Steam lobby preparation | `lobby_metadata_ready`, `handoff_redacted` |
| `security` | redaction, malformed input, hardening | `secret_redacted`, `packet_rejected` |

## Severity

| Level | Meaning | Rules |
| --- | --- | --- |
| `trace` | high-volume local development detail | disabled by default and never used for secrets |
| `debug` | local diagnostic detail | bounded volume and redacted |
| `info` | expected state transition or successful check | default for smoke and handoff logs |
| `warn` | recoverable anomaly | packet loss, reconnect fallback, stale command, budget warning |
| `error` | failed operation requiring attention | failed decode, failed smoke, invalid state |
| `fatal` | process cannot continue safely | corrupt invariant, unsafe secret exposure, destructive risk |

Use the lowest level that still lets an operator act. Do not use `error` for
expected budget-blocked states; use `warn` or `info` with `status=blocked`.

## Correlation IDs

| Field | Owner | Notes |
| --- | --- | --- |
| `run_id` | harness/operator | stable for one local smoke, perf run, or evidence export |
| `client_id` | harness/Godot | local client index or profile label; no private account data |
| `connection_id` | transport/server | transport-local and may change on reconnect |
| `player_session_id` | server | stable session identity; use local/mock redacted form in docs |
| `match_id` | server/harness | local or test match label |
| `trace_id` | caller | optional cross-system trace for one scenario |
| `command_id` | command channel | safe numeric/idempotency identifier when not tied to secrets |
| `snapshot_id` | server snapshot builder | safe numeric snapshot identifier |

Correlation IDs are diagnostic keys, not authority. A log entry cannot make a
command accepted, a snapshot authoritative, or a Steam handoff valid.

## Redaction Rules

Never log or persist:

- Steam auth/session tickets,
- provider tokens,
- secrets, passwords, API keys, or private credentials,
- raw private account data,
- private provider payloads,
- large raw packet dumps by default,
- real non-local endpoint details in public evidence.

Allowed with care:

- local/mock `player_session_id`,
- synthetic local identity labels,
- numeric command IDs and snapshot IDs,
- redacted endpoint mode such as `local_direct_server`,
- aggregate counts and timings,
- local-only file paths when they are already repo-relative.

When uncertain, store a redacted classification and a hash or stable synthetic
label instead of raw content.

## Required Fields By Flow

| Flow | Required fields |
| --- | --- |
| Command queue | `category=command`, `client_id`, `player_session_id`, `command_id`, `command_type`, `client_seq`, `target_tick`, `status` |
| Server command result | `category=command`, `connection_id`, `player_session_id`, `command_id`, `status`, `reason`, `server_tick` |
| Snapshot build/apply | `category=snapshot`, `player_session_id`, `snapshot_id`, `server_seq`, `server_tick`, `entity_count`, `removed_count`, `visible_entities` |
| Reconnect | `category=session`, `old_connection_id`, `new_connection_id`, `player_session_id`, `phase`, `full_snapshot_required` |
| Perf/budget | `category=performance`, `run_id`, `scenario_id`, `machine_label`, `budget_result`, metric names, p50/p95/p99 where relevant |
| Steam facade | `category=steam_facade`, `identity_mode`, `endpoint_mode`, `endpoint_epoch`, `ready_epoch`, redaction status |
| Hardening/security | `category=security`, `reason`, redacted packet class, reject stage, no raw secret |

Hardening/security logging must also follow
`docs/observability/security-privacy-logging-checklist.md`. Rejected, duplicate,
or disconnected hostile inputs must record `mutate_authoritative_state=false`
and `redacted_diagnostics_only=true`.

## Implementation Notes

- Godot `ClientLog.gd` currently emits JSON-like records with `level`,
  `context`, `system`, `message`, and `fields`; GOBS-02 may extend it toward
  `millions_log_event_v0`.
- Server smoke currently emits a stable text line; GOBS-02 may add structured
  helpers without breaking the existing smoke.
- Perf reports remain separate artifacts, but log events should reference their
  `scenario_id`, `run_id`, and `budget_result`.
- Planned debug overlay fields live in
  `docs/observability/debug-overlay-fields.md`; troubleshooting steps live in
  `docs/runbooks/observability-troubleshooting.md`.
- Observability evidence export rules live in
  `docs/observability/evidence-export-plan.md`.
- Security and privacy logging checklist rules live in
  `docs/observability/security-privacy-logging-checklist.md`.

## Planned Counter Catalog

GOBS-02 introduces a small planned-counter surface for server and Godot code.
The first counter names are:

| Counter | Primary owner | Purpose |
| --- | --- | --- |
| `connections_active` | server/client bridge | Current local/mock connection count. |
| `commands_pending` | Godot command queue/server reliability | Pending reliable command count. |
| `commands_accepted` | server command path | Accepted command count. |
| `commands_rejected` | server command path | Rejected command count. |
| `snapshots_built` | server snapshot path | Built full/delta snapshot count. |
| `snapshots_dropped` | server/network path | Dropped or skipped snapshot count. |
| `visible_entities` | server AOI/Godot world mirror | Visible entity count for a client. |
| `render_proxy_count` | Godot render adapter | Current render proxy count. |
| `backpressure_events` | load shedding | Degrade/drop/disconnect pressure events. |
| `redaction_events` | logging/security | Redaction actions or blocked unsafe fields. |

Implementation surfaces:

- Rust: `server/src/observability.rs` exposes `ObservabilityCounter` and
  `ObservabilityCounters`.
- Godot: `client/godot/scripts/autoload/PerfLedger.gd` exposes
  `set_metric`, `increment_counter`, `counter_value`, `has_counter`,
  `snapshot`, and `reset`.

These counters are local diagnostic state. They do not close performance,
release, gameplay, Steam, or security claims without matching evidence.

## Done Rules For Logging Changes

A logging change is not done until:

- it uses a known category and severity,
- it includes needed correlation IDs,
- it is redacted by default,
- it has bounded volume or a gate for high-volume debug,
- it does not claim gameplay authority,
- it has a focused test, smoke, or docs-only reason.
