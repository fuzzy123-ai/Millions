# Performance Baseline

Date: 2026-07-02
Status: slice BUDGET-01 baseline schema

## Purpose

Every scale claim must name the machine, scenario, entity counts, visible
counts, p50/p95/p99 timings, bandwidth, memory, and build/tool versions.

## Baseline Machine

This is the first local development baseline. Future reports must either use
this machine label or define a new machine label with the same fields.

| Field | Value |
| --- | --- |
| Machine label | `local-dev-01` |
| CPU | AMD Ryzen 5 2600X Six-Core Processor, 6 cores / 12 logical processors |
| RAM | 32 GB installed, 31.9 GiB reported usable by Windows query |
| GPU | NVIDIA GeForce RTX 3060, adapter RAM query reports about 4.0 GB |
| OS | Microsoft Windows 10 Pro, version 10.0.19045, build 19045 |
| Rust toolchain | rustc/cargo 1.96.1 via `C:\Users\nkatz\.cargo\bin\` |
| Godot version | 4.7.stable.official.5b4e0cb0f via WinGet package path |
| Script runtime | Windows PowerShell 5.1 |

## Provisional Budgets

| Metric | Initial budget |
| --- | --- |
| Server tick rate | 20 Hz |
| Server tick p99 | under 50 ms |
| 1k sim p95 | under 10 ms |
| 5k sim p95 | under 20 ms |
| 10k sim p95 | under 35 ms |
| Client frame p95 | under 16.7 ms |
| Godot decode p95 | under 2 ms |
| Godot snapshot apply p95 | under 3 ms |
| Godot render update p95 | under 5 ms |
| Normal AOI bandwidth p95 | under 256 KB/s per client |
| 10k stress bandwidth p95 | under 768 KB/s per client unless stress-only |
| Reconnect full snapshot p95 | under 3 seconds |
| Memory | no unbounded growth during soak |

These numbers are initial guardrails, not release promises. A report may fail a
budget and still be useful evidence, but the corresponding scale or stability
claim remains blocked until the failure is fixed or the budget is explicitly
changed with rationale.

## Plan Coupling

The canonical machine-readable budget source is
`docs/plans/millions-plan.json` under `performance_budgets`. This document
explains those values; it does not override them.

| JSON key | Scenario/report field | Blocks claim |
| --- | --- | --- |
| `server_tick_hz` | server tick rate | Any authoritative server timing claim |
| `server_tick_p99_ms_max` | `server_tick_ms.p99` | Server tick stability, 5k/10k, RC |
| `sim_1k_p95_ms_max` | simulation-only p95 | 1k simulation scale |
| `sim_5k_p95_ms_max` | simulation-only p95 | 5k simulation scale |
| `sim_10k_p95_ms_max` | simulation-only p95 | 10k simulation scale |
| `client_frame_p95_ms_max` | Godot frame p95 | Client render/playtest readability |
| `godot_decode_p95_ms_max` | `godot_ms.decode_p95` | Protocol decode/render pipeline |
| `godot_snapshot_apply_p95_ms_max` | `godot_ms.snapshot_apply_p95` | Snapshot apply/reconnect client claim |
| `godot_render_update_p95_ms_max` | `godot_ms.render_update_p95` | Render proxy claim |
| `normal_aoi_bandwidth_kb_s_p95_max` | bandwidth p95 per client | Normal multiplayer/AOI claim |
| `stress_10k_bandwidth_kb_s_p95_max` | bandwidth p95 per client | 10k stress claim |
| `reconnect_full_snapshot_p95_s_max` | reconnect full snapshot p95 | Reconnect claim |
| `memory_rule` | memory start/peak/end trend | Soak or RC stability claim |

`BUDGET-02` sets these as provisional gates. `BUDGET-03` wires server/network
assertions. `BUDGET-04` wires Godot decode/apply/render assertions. Until those
assertion slices exist, reports must still copy the budget keys they intend to
claim against and mark missing assertion coverage as `blocked`.

Changing a budget requires:

- updating `performance_budgets` in the JSON plan,
- updating this table or its rationale,
- recording the changed slice handoff,
- running `scripts\validate_plans.ps1` and `scripts\check_foundation.ps1`.

## Scenario Matrix

Scenario IDs are stable names used by reports, fixtures, and future CI/local
matrix entries.

| Scenario ID | Purpose | Entity count | Clients | Required before claim |
| --- | --- | ---: | ---: | --- |
| `server_idle_20hz` | Validate empty authoritative tick overhead. | 0 | 0 | Any server tick claim |
| `protocol_fixture_decode` | Validate fixed protocol decode cost and fixture compatibility. | 0 | 0 | MV-G1 protocol fixture claim |
| `sim_1k_single_client` | First simulation-scale sanity check. | 1,000 | 1 | 1k sim claim |
| `sim_5k_single_client` | Mid-scale server/AOI pressure. | 5,000 | 1 | 5k sim claim |
| `sim_10k_single_client` | Stress server, AOI, snapshot, and bandwidth budgets. | 10,000 | 1 | 10k sim claim |
| `int_aoi_delta_steady_128_visible` | Estimate steady visible-delta bandwidth. | 128 visible | 1 | Normal AOI bandwidth regression gate |
| `int_aoi_delta_churn_256_visible_32_removed` | Estimate churn with visible records and removed IDs. | 288 | 1 | Normal AOI bandwidth regression gate |
| `int_aoi_10k_aggregate_far_state` | Estimate 10k AOI bandwidth with aggregate far-state cells. | 10,000 | 1 | 10k stress AOI bandwidth regression gate |
| `godot_render_1k_visible` | Client render/decode/apply budget baseline. | 1,000 visible | 1 | Render proxy claim |
| `local_two_client_loop` | Same-workstation server plus two client smoke. | TBD by slice | 2 | Local multiplayer claim |
| `reconnect_full_snapshot_1k` | Reconnect full snapshot timing. | 1,000 | 1 reconnecting | Reconnect claim |
| `loss_jitter_command_ack` | Command reliability under simulated loss/jitter. | TBD by slice | 1+ | Reliability claim |
| `soak_60m_baseline` | Memory/log/queue stability over time. | TBD by slice | TBD by slice | Soak claim |

`TBD by slice` means the scenario name is reserved here, but the exact load
profile is owned by the later roadmap slice that implements the harness.

## Percentile Conventions

- `p50`, `p95`, and `p99` are calculated over the same warmed-up sample window.
- Reports must state warmup duration, sample duration, and sample count.
- Startup, fixture generation, and teardown are excluded unless the scenario is
  specifically measuring startup or reconnect.
- Frame and tick times are reported in milliseconds.
- Bandwidth is reported as KB/s per client plus aggregate KB/s where available.
- Snapshot size is reported in bytes before any compression gate.
- Memory is reported as RSS/working-set MB at start, peak, and end.
- Each report records the exact binary/tool versions used.

## Failure Semantics

A scenario is `pass` only when all required metrics are present and each required
metric is within budget.

A scenario is `fail` when any required metric exceeds budget, a required metric
is missing, the run crashes, the harness cannot complete, or the report omits
machine/tool identity.

A scenario is `blocked` when a prerequisite gate is not available yet, such as
Godot headless execution, fixture implementation, local multi-client launcher,
or live Steam permission.

A scenario is `informational` only when the slice explicitly states that no
budget claim is being made. Informational rows cannot close a scale, stability,
or release gate.

## Required Report Fields

- date
- commit/build id
- machine label
- scenario id
- entity count
- visible entity count per client
- client count
- faction count
- server tick p50/p95/p99
- snapshot size p50/p95/p99
- bandwidth per client p50/p95/p99
- Godot decode/apply/render/frame p50/p95/p99
- server RSS
- Godot RSS
- queue depth
- dropped snapshot count
- reconnect full snapshot time
- pass/fail against budget
- notes explaining any intentional budget change

## Minimal Machine-Readable Row

Future perf rows should be representable with this shape before any richer
formatting is added:

```json
{
  "schema_version": 1,
  "date": "YYYY-MM-DD",
  "machine_label": "local-dev-01",
  "build_id": "git-or-local-build-id",
  "scenario_id": "sim_1k_single_client",
  "status": "pass|fail|blocked|informational",
  "entity_count": 1000,
  "visible_entities_per_client": 1000,
  "client_count": 1,
  "server_tick_ms": {"p50": 0.0, "p95": 0.0, "p99": 0.0},
  "snapshot_bytes": {"p50": 0, "p95": 0, "p99": 0},
  "bandwidth_kb_s_per_client": {"p50": 0.0, "p95": 0.0, "p99": 0.0},
  "godot_ms": {
    "decode_p95": 0.0,
    "snapshot_apply_p95": 0.0,
    "render_update_p95": 0.0,
    "frame_p95": 0.0
  },
  "memory_mb": {"server_peak": 0.0, "godot_peak": 0.0},
  "reconnect_full_snapshot_s": {"p95": null},
  "budget_result": "pass|fail|blocked",
  "notes": ""
}
```
