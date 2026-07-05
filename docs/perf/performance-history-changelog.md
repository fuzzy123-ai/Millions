# Performance History Changelog

Date: 2026-07-03
Slice: PHIST-04
Status: human-readable Foundation performance history

## Purpose

This changelog is the human companion to the machine-readable performance
history ledger. It explains what changed, why it changed, which artifacts prove
the change, and which claims remain blocked.

Use this document when the JSON rows are too terse to answer whether a
performance movement is expected, provisional, blocked, or a real regression.

## Update Rule

Add an entry whenever any of these changes:

- a ledger row is added or updated,
- a regression threshold changes,
- a `budget_result`, `status`, `claim_scope`, or `budget_keys` value changes,
- a harness starts measuring a previously null metric,
- a local-only row is promoted toward budget or release evidence.

Every entry must name:

- date,
- slice,
- affected scenarios or families,
- source artifacts,
- why the entry changed,
- claim scope,
- open gates.

If the change is also machine-readable, update
`docs/evidence/performance-history-why-changed-evidence.json`.

## Foundation Timeline

| Date | Slice | Area | Change | Current claim |
| --- | --- | --- | --- | --- |
| 2026-07-02 | BUDGET-01 | Baseline | Defined baseline machine, provisional budgets, scenario matrix, percentiles, and failure semantics. | Budget contract only. |
| 2026-07-03 | BUDGET-02 | Budgets | Copied provisional budget keys into the canonical JSON plan. | Plan-level guardrails, not release promises. |
| 2026-07-03 | BUDGET-03 | Server assertions | Added Rust budget assertion surface for server tick, sim scale, bandwidth, and reconnect reports. | Assertion contract exists. |
| 2026-07-03 | BUDGET-04 | Godot assertions | Added Godot budget assertion surface for decode, snapshot apply, render update, and frame p95. | Assertion contract exists. |
| 2026-07-03 | SIM-02 | Simulation scale | Added deterministic simulation-only 1k, 5k, and 10k scenario catalog. | Scenario definition only. |
| 2026-07-03 | SIM-03 | Simulation scale | Recorded first local informational sim-scale report and thresholds. | Local regression signal only. |
| 2026-07-03 | INT-03 | Interest bandwidth | Recorded local AOI byte-estimate report and thresholds. | Local regression signal only. |
| 2026-07-03 | REND-01 | Godot render | Added 1,000-proxy render stress smoke report without frame/render p95 measurements. | Informational and blocked. |
| 2026-07-03 | PHIST-01 | Ledger | Created performance history ledger row schema and naming rules. | Schema sample only. |
| 2026-07-03 | PHIST-02 | Ledger emitters | Added server and Godot row emitters plus parseable sample rows. | Informational contract only. |
| 2026-07-03 | PHIST-03 | Regression history | Added ledger-wide threshold contract and machine-readable why-changed evidence. | Local threshold contract; Godot render remains blocked until measured. |
| 2026-07-03 | PHIST-04 | Changelog | Started this human-readable history and update rule. | Changelog coverage for Foundation start. |
| 2026-07-03 | SOAK-01 | Soak | Defined local, loss/jitter, reconnect, and live-gated soak scenario catalog. | Scenario definition only. |
| 2026-07-03 | SOAK-02 | Soak | Added server and Godot metric emitters for memory, allocations, queues, connections, dropped snapshots, resends, and shutdown state. | Metric emitter contract only. |
| 2026-07-05 | GSWARM-03 | Swarm/collision | Added a PHIST row family and sample for the 1,000-zombie swarm load smoke with local elapsed-stage counters, now including the configured movement loop stage. | Local regression signal only; p95 server tick, live bandwidth, memory, soak, gameplay, and release gates remain blocked. |
| 2026-07-05 | GSWARM-03 | Swarm/collision thresholds | Added a ledger-wide local threshold row for swarm/collision smoke elapsed time, including the configured movement loop stage, snapshot bytes, and estimated local bandwidth. | Local threshold contract only; formal p95 server tick, memory, soak, live transport, and release gates remain blocked. |

## Current Claim Map

| Family | Current artifacts | Claim scope | Still blocked by |
| --- | --- | --- | --- |
| `simulation_scale` | `tests/perf/sim-scale-local-report.json`, `tests/perf/sim-scale-regression-thresholds.json`, `docs/perf/performance-history-regression-thresholds.json` | `local_regression_signal` | Soak, gameplay workloads, release-candidate evidence, final ECS decision. |
| `swarm_collision` | `tests/perf/swarm-load-smoke-report.json`, `tests/perf/perf-history-swarm-row-sample.json`, `docs/perf/performance-history-regression-thresholds.json` | `local_regression_signal` | Formal p95 server tick budgets, memory rows, live transport bandwidth, unconditional movement loop, final horde AI, gameplay physics, soak, and release evidence. |
| `interest_bandwidth` | `tests/perf/interest-bandwidth-smoke-report.json`, `tests/perf/interest-bandwidth-regression-thresholds.json`, `docs/perf/performance-history-regression-thresholds.json` | `local_regression_signal` | Live transport, packet loss/resend, compression, MTU, gameplay visibility, release evidence. |
| `godot_render` | `tests/perf/render-stress-smoke-report.json`, `tests/perf/perf-history-godot-row-sample.json`, `docs/perf/performance-history-regression-thresholds.json` | `informational_contract_only` | Measured decode/apply/render/frame p95 rows, final render technology, readability checks. |
| `server_tick` | `docs/perf/performance-baseline.md`, `server/src/perf_budget.rs` | Budget assertion contract | Dedicated measured idle tick report and longer stability runs. |
| `reconnect` | `server/src/perf_budget.rs`, reconnect Foundation tests | Budget assertion contract | Measured reconnect full snapshot p95 report and local multiplayer smoke. |
| `soak` | `tests/soak/soak-scenarios.json`, `server/src/soak_metrics.rs`, `client/godot/scripts/perf/SoakMetrics.gd`, `docs/perf/soak-scenarios.md`, `docs/runbooks/soak-scenarios.md` | Metric emitter contract only | SOAK-03 harness commands, SOAK-04 operator language, measured rows, and any live Go for two-machine runs. |

## PHIST-04 Assessment

The Foundation now has both machine-readable and human-readable performance
history starting points:

- schema and row naming,
- server and Godot row emitters,
- local sim/AOI thresholds,
- contract-only Godot render thresholds,
- why-changed evidence rules,
- this changelog.

This does not make scale, soak, release, live Steam, gameplay, final rendering,
or final ECS claims complete. It makes the next performance slices harder to
misread and gives future agents one obvious place to explain performance
movement before claiming a gate.
