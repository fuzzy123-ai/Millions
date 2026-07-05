# Simulation Scale Regressions

Date: 2026-07-03
Status: slice SIM-03 local Foundation evidence

## Purpose

SIM-03 records the first local tick-time, memory, and regression-threshold
evidence for the deterministic simulation-only scenarios introduced by SIM-02.
The scenarios are:

- `sim_1k_single_client`
- `sim_5k_single_client`
- `sim_10k_single_client`

This evidence is deliberately narrow. It covers abstract server-side entity
columns, movement stubs, spatial-grid rebuilds, and full snapshot byte counting.
It does not include gameplay rules, pathfinding, combat, economy, AI, live
networking, real Steam, AOI filtering, client rendering, or a long-term ECS crate
choice.

## Evidence Files

- `tests/perf/sim-scale-local-report.json`
- `tests/perf/sim-scale-regression-thresholds.json`
- `tests/perf/sim-scale-scenarios.json`

The local report is `informational` even when its budget result is `pass`.
Informational rows do not close release-candidate, soak, bandwidth, live Steam,
or gameplay performance claims.

## Local Results

| Scenario | Entities | Tick p95 | Tick p99 | Peak memory | Snapshot bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| `sim_1k_single_client` | 1,000 | 1.133525 ms | 1.133525 ms | 3.449 MB | 36,024 |
| `sim_5k_single_client` | 5,000 | 6.57575 ms | 6.57575 ms | 4.184 MB | 180,024 |
| `sim_10k_single_client` | 10,000 | 9.8306 ms | 9.8306 ms | 5.188 MB | 360,024 |

Each row used seven local samples on `local-dev-01`. Tick timing is wall-clock
time per simulated server tick for the current simulation-only runner, including
movement-stub application, spatial-grid rebuilds, and snapshot preparation.

## Regression Thresholds

Thresholds mirror the provisional budget keys in
`docs/plans/millions-plan.json`:

- 1k p95 must remain at or below 10 ms.
- 5k p95 must remain at or below 20 ms.
- 10k p95 must remain at or below 35 ms.
- Server tick p99 must remain at or below 50 ms.
- Peak server memory for these local samples must remain below 64 MB.
- End memory must not exceed start memory by more than 16 MB.

If a later report exceeds any threshold, the relevant scale claim is a
regression until either the implementation is fixed or the budget is explicitly
changed in the JSON plan with rationale.

## Open Limits

- No soak duration has been measured.
- No bandwidth or AOI filtering claim is made.
- No render, decode, apply, or client-frame claim is made.
- No release-candidate claim is made.
- `G-ECS-CHOICE` remains open.
