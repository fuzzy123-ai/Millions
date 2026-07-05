# Soak Scenarios

Date: 2026-07-03
Slice: SOAK-01
Status: scenario definitions only

## Purpose

SOAK-01 defines the first long-run stability scenarios and the evidence each
future run must produce. It does not execute a soak run.

The scenario catalog lives at:

```text
tests/soak/soak-scenarios.json
```

The lightweight schema lives at:

```text
tests/soak/soak-scenarios.schema.json
```

## Scenario Order

| Scenario | Class | Duration | Clients | Load | Current status |
| --- | --- | ---: | ---: | --- | --- |
| `soak_15m_foundation_local` | `safe_offline` | 15 min | 2 | 1k abstract local/mock | planned |
| `soak_60m_baseline` | `safe_offline` | 60 min | 2 | 5k abstract local loopback/mock | planned |
| `soak_120m_loss_jitter_reconnect` | `safe_offline` | 120 min | 4 | 10k abstract with simulated loss/jitter/reconnect churn | blocked until SOAK-02/SOAK-03 emitters and harness exist |
| `soak_live_two_machine_60m` | `needs_live_go` | 60 min | 2 | live two-machine path | blocked until explicit fresh operator Go |

## Required Evidence

Every soak report must include these groups before it can move beyond
`scenario_definition_only`:

- memory start, peak, and end for server and Godot where applicable,
- queue depth p50/p95/p99 or max,
- connection count over time,
- dropped snapshot count,
- resend count and resend backlog,
- structured log event counts by severity/category,
- graceful shutdown result.

Missing required groups make the row `blocked`, not `pass`.

## Budget Coupling

The first local scenarios use existing provisional budget keys:

- `memory_rule`
- `server_tick_p99_ms_max`
- `normal_aoi_bandwidth_kb_s_p95_max`
- `stress_10k_bandwidth_kb_s_p95_max`
- `reconnect_full_snapshot_p95_s_max`

SOAK-02 owns metrics emission. SOAK-03 owns harness commands. SOAK-04 owns
operator Go/No-Go wording for long-run execution.

SOAK-02 emitter surfaces:

- `server/src/soak_metrics.rs`
- `client/godot/scripts/perf/SoakMetrics.gd`
- `client/godot/scripts/tests/soak_metrics_check.gd`

The emitters provide stable local snapshot keys for memory, allocation, queue
depth, connection count, dropped snapshots, resend counts, and shutdown state.
They do not run long sessions.

## Claim Limits

These scenarios do not close release, live Steam, gameplay, final render, final
ECS, or large-scale stability claims. They only define the queue of soak work.

The live two-machine scenario is intentionally present as a blocked
`needs_live_go` entry so future agents do not silently convert it into a local
or mock run.
