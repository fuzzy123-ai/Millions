# Performance Tests

Perf tests and reports will validate server tick, client frame, bandwidth,
memory, reconnect time, AOI, rendering, and movement budgets.

No scale claim is complete without a corresponding report row.

## Pipeline Rule For New Features

Every feature slice that plausibly changes runtime cost must run the relevant
focused tests and the closest available performance/readability smoke before the
slice is marked done. Runtime-cost changes include simulation work, entity
count, snapshots, network bandwidth, rendering, AI, movement, collision, combat,
selection density, or other scale-sensitive behavior.

Docs-only, contract-only, planning-only, and small non-runtime refactors may
skip performance smokes when the handoff states why the change cannot affect
runtime cost. They still need the normal focused checks for the touched files.

When a feature creates a new scale-sensitive path, add or update a report under
`tests/perf/` and document whether it is measured, informational, blocked, or a
regression guard. Do not optimize away an existing performance smoke, threshold,
or report row unless the replacement provides equal or stronger coverage for the
same risk.

If measured p95/p99 evidence is missing, keep `budget_result: "blocked"` and
record the missing evidence rather than claiming pass. This applies even when
functional tests are green.

The baseline schema, scenario IDs, percentile conventions, and failure semantics
live in `docs/perf/performance-baseline.md`. Perf harnesses should emit rows that
can be compared against that schema before adding richer report formats.

The machine-readable provisional budgets live in
`docs/plans/millions-plan.json` under `performance_budgets`. A perf report that
omits the relevant budget key or cannot compare against it must report
`budget_result: "blocked"` rather than closing a scale, reconnect, soak, or
release claim.

## Server Budget Assertions

`server/src/perf_budget.rs` is the current BUDGET-03 assertion surface for
server, network, reconnect, and AOI-adjacent reports. It evaluates:

- `server_idle_20hz` against server tick p99,
- `sim_1k_single_client`, `sim_5k_single_client`, and
  `sim_10k_single_client` against tick, sim p95, and bandwidth p95,
- `reconnect_full_snapshot_1k` against tick and reconnect restore p95.

Result rules:

- `Pass`: every required metric is present and within budget,
- `Fail`: at least one required metric exceeds budget,
- `Blocked`: at least one required metric is missing.

This is not a real perf harness yet. It is the assertion contract future harness
rows must satisfy before they close server/network/reconnect/AOI claims.

## Godot Budget Assertions

`client/godot/scripts/perf/PerfBudget.gd` is the current BUDGET-04 assertion
surface for Godot reports. It evaluates:

- `decode_p95` against `godot_decode_p95_ms_max`,
- `snapshot_apply_p95` against `godot_snapshot_apply_p95_ms_max`,
- `render_update_p95` against `godot_render_update_p95_ms_max`,
- `frame_p95` against `client_frame_p95_ms_max`.

`client/godot/scripts/tests/perf_budget_check.gd` verifies `pass`, `fail`, and
`blocked` results. Missing required Godot metrics are blocked, not passed.

## Report Schema

PERF-01 report artifacts:

- `tests/perf/perf-report.schema.json`
- `tests/perf/sample-perf-report-row.json`
- `docs/perf/perf-report-format.md`

The sample row is intentionally informational/blocked. It proves the shape, not
runtime performance.

## Performance History Ledger

PHIST-01 artifacts:

- `docs/perf/performance-history-ledger.md`
- `tests/perf/performance-history-ledger.schema.json`
- `tests/perf/performance-history-ledger-sample.json`

The ledger schema records comparable performance rows over time with stable
scenario families, deterministic ledger IDs, source artifacts, why-changed
notes, claim scope, and redaction status. The first sample row is
`informational` and `budget_result: "blocked"`; it is not a measured result.

PHIST-02 emitter artifacts:

- `server/src/perf_history.rs`
- `client/godot/scripts/perf/PerfHistoryRow.gd`
- `client/godot/scripts/tests/perf_history_row_check.gd`
- `tests/perf/perf-history-server-row-sample.json`
- `tests/perf/perf-history-swarm-row-sample.json`
- `tests/perf/perf-history-swarm-batch-vs-single-row-sample.json`
- `tests/perf/perf-history-godot-row-sample.json`

The server and Godot emitters produce ledger-shaped rows for harnesses while
remaining conservative: rows are informational and blocked until measured
runtime data and threshold evidence exist. The Godot check loads the ledger
schema and verifies that the emitted row contains all required fields.
The swarm/collision sample uses local elapsed-stage counters from the swarm load
smoke as regression signal only; it does not close p95 server-tick, live
bandwidth, memory, soak, gameplay, or release gates.
The swarm batch-vs-single sample records local single-loop p95, batch-loop p95,
and batch/single p95 ratio as regression signal only; it does not claim batch is
faster or within a formal budget.

## Local Perf Smoke

PERF-03 command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_perf_smoke.ps1
```

Expected final line:

```text
perf_smoke status=ok schema=ok rust_metrics=ok rust_budget=ok godot_budget=ok
```

This smoke validates schema parsing and the current Rust/Godot budget assertion
surfaces. It does not measure runtime performance or close scale claims.

## Simulation Scale Scenarios

SIM-02 artifacts:

- `server/src/simulation_scale.rs`
- `tests/perf/sim-scale-scenarios.json`
- `tests/perf/sim-scale-local-report.json`
- `tests/perf/sim-scale-regression-thresholds.json`
- `docs/perf/simulation-scale-regressions.md`

The server exposes deterministic simulation-only workloads for:

- `sim_1k_single_client`
- `sim_5k_single_client`
- `sim_10k_single_client`

Each workload builds abstract entity columns, applies deterministic movement
stubs for four server ticks, rebuilds the spatial grid, and creates a full
snapshot for byte counting. This proves that the Foundation can generate the
required 1k/5k/10k scale inputs without gameplay behavior, live networking, real
Steam, pathfinding, combat, economy, AI, or a long-term ECS crate choice.

These rows remain `scenario_definition_only`. They do not close performance,
scale, soak, release-candidate, or bandwidth claims until SIM-03 records measured
tick time, memory, and regression thresholds as report rows.

SIM-03 records the first local informational report and provisional regression
thresholds for the same 1k/5k/10k scenarios. Those rows can catch local
simulation regressions, but they still do not close soak, bandwidth, release, or
gameplay performance claims.

## Movement Scale Scenario Catalog

NAV-02 artifacts:

- `server/src/movement_scale.rs`
- `tests/perf/movement-scale-scenarios.json`

The movement scale catalog turns the NAV-01 option families into deterministic
server-side scenario inputs for direct steering, grid corridor pathing,
flow-field objective movement, formation anchors, and local avoidance pressure.
Each run records query pressure, correction counts, occupied cells, blocker cell
counts, flow-field build/query/visited/unreachable counts for the shared
objective scenario, and the mapdata fixture checksum. These are scenario
definitions and local deterministic counters only. They do not implement final
pathfinding, formations, avoidance, collision, gameplay movement, balance, live
networking, or release-candidate evidence.

Flow-field smoke artifacts:

- `scripts/run_flow_field_smoke.ps1`
- `tests/perf/flow-field-smoke-report.json`

The flow-field smoke validates that the 10,000-entity shared-objective movement
scenario builds a deterministic local flow field, reuses the cached field after
the first tick, keeps the flow-field cache under a deterministic 64-entry cap
without evictions in the bounded smoke, queries it once per entity per tick,
checks a bounded sample of candidate steps through the shared
resolved-admission collision probe with
static obstacle bodies, records both initial admission and bounded
resolved-admission counters plus total/max correction distance from that probe,
records local movement-probe candidate decisions, applies those decisions to the
bounded sampled candidate deltas, runs a bounded local apply-physics step for
the sampled movement world, records contact/correction distance and sample
position sync counters, records visited and blocked cells, and keeps unreachable
queries at zero. The report remains informational and blocked until measured p95
tick evidence and formal movement/pathfinding budgets exist.

NAV-03 artifacts:

- `client/godot/scripts/render/MovementReadabilityStress.gd`
- `client/godot/scripts/tests/movement_readability_stress_check.gd`
- `tests/perf/movement-readability-stress-report.json`

The Godot movement readability stress creates 1,000 deterministic movement
proxies across 20 groups and 10 lanes, then records group coverage, occupied
readability cells, density, and bounds through a headless check. The report is
informational and blocked because it does not measure frame/render p95, choose a
render technology, validate real movement, or sign off player readability.

## Selection Readability Stress

GCTRL-03 artifacts:

- `client/godot/scripts/render/SelectionReadabilityStress.gd`
- `client/godot/scripts/tests/selection_readability_stress_check.gd`
- `tests/perf/selection-readability-stress-report.json`

The Godot selection readability stress creates 1,000 deterministic selectable
proxies and 128 local selection overlays, then verifies selection feedback,
command-context-ready feedback, occupied readability cells, density, and bounds
through a headless check. The report is informational and blocked because it
does not measure frame/render p95, choose final UI or render technology, validate
gameplay authority, or sign off player readability.

## Cover Combat Smoke

GCOV-04 artifacts:

- `server/src/cover.rs`
- `scripts/run_cover_combat_perf_smoke.ps1`
- `tests/perf/cover-combat-smoke-report.json`

The cover combat smoke runs deterministic dense targeting classifications across
64 attacker positions and 96 target positions. It requires all four result
buckets: clear, target in cover, blocked by obstacle, and out of range. The
report is informational and blocked because it does not measure p95 query cost,
apply damage, define final cover bonuses, choose balance, or close release
readiness.

## Swarm Load Smoke

GSWARM-03 artifacts:

- `server/src/swarm.rs`
- `server/src/collision.rs`
- `client/godot/scripts/render/SwarmReadabilityStress.gd`
- `client/godot/scripts/tests/swarm_readability_stress_check.gd`
- `scripts/run_swarm_load_smoke.ps1`
- `tests/perf/swarm-load-smoke-report.json`

The swarm load smoke covers 1,000 active zombie swarm entities with route
pressure, direct and memory aggro trails, AI LOD buckets, collision-prep bodies,
collision-prep contacts, target-position collision admission checks, collision
resolved-admission checks, collision resolution-plan corrections, bounded local
CollisionWorld physics steps with total/max applied correction distance, bounded
movement/collision probe decisions, bounded batch movement-probe evidence for
simultaneous sampled candidates,
bounded non-authoritative swarm movement-preview flow-field queries and deltas,
cloned-world preview physics counters and correction distances, bounded opt-in
`SwarmState` movement apply counters plus correction distances, bounded
clamped movement-apply stability evidence that covers both collision movement
probes and the later local physics step,
bounded opt-in batch movement-apply counters for simultaneous sampled
candidates,
bounded opt-in batch movement-tick counters, configured batch movement-mode
counters, and batch movement full/delta snapshot bytes,
movement-apply flow-field cache hit counters, bounded
movement-apply flow-field cache entry and eviction counters, bounded
post-physics sample position sync counters, bounded opt-in movement-tick
counters, movement-tick full snapshot bytes, configured movement-mode tick
counters, configured clamped movement-mode probe/physics correction-limit
counters, bounded
two-tick configured movement-loop cache/physics/position-change counters,
static-obstacle flow-field blocker and movement counters sourced from
validated local map-data obstacles, including obstacle half-extent and
swarm-radius clearance expansion beyond center cells, local AOI-visible movement
delta snapshot bytes, local aggregate far-state movement delta evidence, local
elapsed-time smoke-stage evidence, and a Godot headless 1,000-proxy readability
check. The report is informational and blocked because it does not measure p95
server tick, p95 render frame time, live bandwidth, unconditional tick-loop
movement, live delta transport, final horde
AI, or release readiness.

GSWARM-13 adds a focused local batch movement replication smoke:

- `scripts/run_swarm_batch_movement_replication_smoke.ps1`
- `tests/perf/swarm-batch-movement-replication-smoke-report.json`

The smoke builds the 1,000-zombie movement base, captures a full baseline
snapshot, runs one configured batch flow-field/collision tick, then verifies
that the all-visible AOI delta contains visible entities whose positions changed
from baseline. It also checks a smaller AOI delta with aggregate far-state. The
report remains informational and blocked because it is not live transport,
client prediction, public-network replication, a formal p95 budget, or release
evidence.

GSWARM-05 adds a focused local measurement harness for the configured
flow-field/collision movement loop:

- `scripts/run_swarm_movement_loop_measurement.ps1`
- `tests/perf/swarm-movement-loop-measurement-report.json`

The harness builds the same 1,000-zombie base state once, clones it per sample,
and measures a bounded two-tick configured movement loop with two movement
samples per tick. It records local p50/p95/p99 elapsed microsecond shape plus
cache, physics, and moved-entity counters. The report remains informational and
blocked because it is not a formal p95 server-tick budget, render-frame budget,
live bandwidth measurement, release claim, or final horde pathfinding proof.

GSWARM-10 adds a paired batch-vs-single comparison harness:

- `scripts/run_swarm_batch_vs_single_movement_loop_measurement.ps1`
- `tests/perf/swarm-batch-vs-single-movement-loop-report.json`
- `scripts/run_swarm_batch_vs_single_promotion_check.ps1`
- `tests/perf/swarm-batch-vs-single-promotion-report.json`

The comparison harness reuses the same 1,000-zombie base state and bounded
two-tick configured loop, then measures both the single movement-probe mode and
the batch movement-probe mode with matching sample counts. It records local
p50/p95/p99 shape for both paths, a batch/single p95 ratio in basis points,
cache hits, cache evictions, physics iterations, applied deltas, and moved
entity counters. The report remains informational and blocked because it does
not prove batch is faster, set a formal budget, measure live server ticks, or
close final horde pathfinding.

GSWARM-12 adds the promotion check for those rows. The check keeps a single
local comparison row blocked and requires at least three comparable redacted
rows under the local ratio and elapsed-time thresholds before the signal can be
treated as budget-candidate evidence.

## Collision Physics Smoke

Collision physics artifacts:

- `server/src/collision.rs`
- `scripts/run_collision_physics_perf_smoke.ps1`
- `tests/perf/collision-physics-smoke-report.json`

The collision physics smoke runs deterministic clustered 1,000-body and
5,000-body `CollisionWorld` scenarios through broadphase contact detection and
bounded local overlap-resolution iterations. It records contact pressure,
resolved-admission pressure, iteration count, applied correction count, total
and max applied correction distance, a clamped stability check that caps single
corrections at 50 mm, elapsed local microseconds, and the same conservative
`local_smoke_only` claim scope used by the current physics boundary. This smoke
is required when changes
plausibly affect collision broadphase, resolved admission, resolution planning,
local physics stepping, static-obstacle correction rules, or swarm collision
pressure.

The report remains informational and blocked because it does not measure p95 or
p99 server tick time, compare against a formal collision budget, integrate
gameplay-authoritative movement, validate Godot physics, or close release
readiness.

## Faction Scale Scenario Catalog

GLOAD-02 artifacts:

- `server/src/faction_scale.rs`
- `tests/perf/faction-scale-scenarios.json`
- `docs/gameplay/faction-scale-scenarios.md`
- `docs/perf/faction-scale-reporting.md`

The faction scale catalog translates the provisional GLOAD Stage A-D matrix into
server-addressable scenario IDs, entity counts, role-mix counts, visible entity
targets, aggregate far-state requirements, and conservative 1k/5k/10k budget
reference families. It does not run gameplay, choose the final faction count,
measure performance, or close `G-FACTION-COUNT`.

## Interest Bandwidth Smoke

INT-03 artifacts:

- `tests/perf/interest-bandwidth-smoke-report.json`
- `tests/perf/interest-bandwidth-regression-thresholds.json`
- `docs/perf/interest-bandwidth-regressions.md`

The interest bandwidth smoke estimates protocol v0 snapshot bytes per tick and
KB/s per client for visible entity deltas, removed IDs, and aggregate far-state
cells. The gates catch local AOI bandwidth regressions against the provisional
normal and 10k stress budgets. They do not measure socket transport, packet
loss, resend, compression, fragmentation, MTU, live Steam, or gameplay
visibility.

## Godot Render Stress Smoke

REND-01 artifacts:

- `client/godot/scenes/dev/render_stress_smoke.tscn`
- `client/godot/scripts/render/RenderStressSmoke.gd`
- `client/godot/scripts/tests/render_stress_smoke_check.gd`
- `tests/perf/render-stress-smoke-report.json`

The render stress smoke creates 1,000 simple `Node2D` placeholder proxies under
the scene-owned `RenderProxyHost` and validates the proxy count and layout bounds
in a headless Godot check. The report remains `informational` and
`budget_result: "blocked"` because no p95 frame/render metrics are measured yet.
This keeps REND-01 useful for infrastructure wiring without choosing final
render technology, final art, gameplay authority, or release performance claims.
