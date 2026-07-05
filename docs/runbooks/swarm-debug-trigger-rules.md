# Swarm Debug Trigger Rules

Status: GSWARM-04 local runbook.

Use this runbook when changing swarm behavior, collision preparation, Godot
swarm readability, or performance-report wiring.

## Required Local Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_swarm_load_smoke.ps1
```

Expected final line:

```text
swarm_load_smoke status=ok zombies=1000 behavior=1000 collision_bodies=1000 admission_checks=16 resolved_checks=4 budget_result=blocked
```

The command must remain `budget_result=blocked` until measured p95 server tick,
render frame, and bandwidth evidence exists.

When changing configured swarm movement-loop behavior or measurement wiring, also
run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_swarm_movement_loop_measurement.ps1
```

Expected final line:

```text
swarm_movement_loop_measurement status=ok samples=3 active=1000 ticks=2 movement_samples=12 budget_result=blocked
```

This harness records local p50/p95/p99 elapsed-time shape for the bounded
configured flow-field/collision loop. It is regression evidence only and does
not unblock the formal swarm budget.

When changing the batch movement tick/config path, also run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_swarm_batch_vs_single_movement_loop_measurement.ps1
```

Expected final line:

```text
swarm_batch_vs_single_movement_loop_measurement status=ok samples=3 active=1000 ticks=2 movement_samples=12 budget_result=blocked
```

This harness compares the same bounded configured loop in single-probe and
batch movement modes and records their local p95 shape plus a batch/single p95
ratio. It is comparison evidence only and does not claim batch is faster or
within budget.

When changing batch-vs-single performance-history rows, thresholds, or budget
promotion wording, also run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_swarm_batch_vs_single_promotion_check.ps1
```

Expected final line:

```text
swarm_batch_vs_single_promotion status=ok decision=blocked min_rows=3 current_rows=1 budget_result=blocked
```

This guard prevents a single local comparison row from being treated as a
budget pass. It requires at least three comparable redacted rows before the
local signal can become a budget-candidate discussion.

When changing batch movement snapshot replication, AOI delta wiring, or local
movement-to-snapshot evidence, also run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_swarm_batch_movement_replication_smoke.ps1
```

Expected final line:

```text
swarm_batch_movement_replication_smoke status=ok zombies=1000 delta_changed_min=1 aggregate_far_min=1 budget_result=blocked
```

This smoke captures a full baseline, runs a configured batch
flow-field/collision movement tick, and verifies that the local AOI delta
contains visible entities whose server positions changed from baseline. It does
not prove live packet transport or client prediction.

When changing collision movement-probe fallback behavior or axis-slide wording,
also run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_collision_axis_slide_smoke.ps1
```

Expected final line:

```text
collision_axis_slide_smoke status=ok direct_blocked=1 slide_min=1 budget_result=blocked
```

This smoke proves the opt-in axis-slide probe can turn a blocked direct movement
candidate into a deterministic single-axis candidate while keeping default
swarm movement, final avoidance, and formal budget claims blocked.

## Debug Trigger Thresholds

Use these thresholds for future debug overlays and local smoke assertions:

| Trigger | Threshold | Action |
| --- | ---: | --- |
| `swarm_active_warning` | active zombies >= 250 | Show gathering/pressure debug marker. |
| `swarm_aggro_direct_warning` | direct aggro trails >= 1 | Show urgent aggro marker. |
| `swarm_aggro_memory_warning` | memory aggro trails >= 1 | Show fading trail marker. |
| `swarm_collision_pressure_debug` | collision contacts > 0 | Show debug-only collision pressure marker. |
| `swarm_collision_admission_reject` | admission rejects > 0 | Show server-only blocked-position diagnostic. |
| `swarm_collision_resolution_plan` | planned corrections > 0 | Show correction-plan diagnostic only. |
| `swarm_collision_physics_step` | applied local corrections > 0 | Show local CollisionWorld physics diagnostic only. |
| `swarm_collision_resolved_admission` | resolved admission checks > 0 | Show local candidate-resolution diagnostic only. |
| `swarm_collision_movement_probe` | movement probe corrected or blocked > 0 | Show local candidate movement decision diagnostic only. |
| `swarm_collision_axis_slide_probe` | accepted axis-slide candidates > 0 | Show opt-in local slide-probe diagnostic only. |
| `swarm_movement_preview_flow_field` | movement preview flow-field queries > 0 | Show local intent-to-flow candidate diagnostic only. |
| `swarm_movement_preview_physics` | movement preview physics corrections > 0 | Show local cloned-world physics diagnostic only. |
| `swarm_movement_preview_blocked` | movement preview blocked deltas > 0 | Show local preview blockage diagnostic only. |
| `swarm_movement_apply_opt_in` | movement apply samples > 0 | Show explicit server-side movement apply diagnostic only. |
| `swarm_batch_movement_apply_opt_in` | batch movement apply samples > 0 | Show explicit simultaneous candidate movement apply diagnostic only. |
| `swarm_movement_tick_opt_in` | movement tick samples > 0 | Show controlled spawn/report plus movement tick diagnostic only. |
| `swarm_batch_movement_tick_opt_in` | batch movement tick samples > 0 | Show controlled spawn/report plus batch movement tick diagnostic only. |
| `swarm_movement_snapshot_full` | movement tick snapshot entities > 0 | Show bounded full-snapshot evidence only. |
| `swarm_movement_snapshot_delta` | movement delta snapshot entities > 0 | Show bounded local AOI delta-snapshot evidence only. |
| `swarm_batch_movement_replication_delta` | changed visible delta entities > 0 | Show local server movement-to-delta materialization evidence only. |
| `swarm_aggregate_lod_debug` | aggregate LOD entities > 0 | Show aggregate LOD debug marker. |
| `swarm_budget_blocked` | missing p95 evidence | Keep claim blocked. |

## Evidence Checklist

- Rust focused check: `cargo test swarm_load_smoke`.
- Movement-loop measurement check:
  `powershell -ExecutionPolicy Bypass -File scripts\run_swarm_movement_loop_measurement.ps1`.
- Batch-vs-single movement-loop comparison check:
  `powershell -ExecutionPolicy Bypass -File scripts\run_swarm_batch_vs_single_movement_loop_measurement.ps1`.
- Batch-vs-single promotion guard:
  `powershell -ExecutionPolicy Bypass -File scripts\run_swarm_batch_vs_single_promotion_check.ps1`.
- Batch movement replication smoke:
  `powershell -ExecutionPolicy Bypass -File scripts\run_swarm_batch_movement_replication_smoke.ps1`.
- Collision axis-slide smoke:
  `powershell -ExecutionPolicy Bypass -File scripts\run_collision_axis_slide_smoke.ps1`.
- Godot focused check:
  `client/godot/scripts/tests/swarm_readability_stress_check.gd`.
- Report artifact: `tests/perf/swarm-load-smoke-report.json`.
- Collision admission evidence: the swarm load smoke must include admission
  checks and at least one deterministic overlap reject.
- Collision resolution evidence: the swarm load smoke must include resolution
  contacts and planned corrections, but must not claim applied movement.
- Local physics-step evidence: the smoke must include at least one bounded
  CollisionWorld physics iteration, applied corrections, and positive total/max
  applied correction distance, but must not claim authoritative gameplay
  movement.
- Resolved admission evidence: the swarm load smoke and collision physics smoke
  must include bounded candidate admission checks under pressure, but must not
  claim alternate path selection or authoritative movement.
- Movement-probe evidence: the swarm load smoke must include bounded
  movement/collision probe decisions, but must not apply their resolved deltas to
  authoritative state.
- Axis-slide evidence: the dedicated collision axis-slide smoke must include a
  blocked direct movement candidate, at least one accepted deterministic
  single-axis slide candidate, a direct-accepted case with no slide attempts,
  and clamp-policy propagation into slide attempts. This proves only the opt-in
  fallback probe; default swarm movement, full pathfinding, and final avoidance
  remain separate gates.
- Batch movement-probe evidence: the swarm load smoke must include bounded
  simultaneous candidate probes for the movement-preview sample set, with no
  unknown bodies, at least one local resolution iteration, and positive
  correction distance counters. This is pressure evidence for future swarm
  movement batching, not final avoidance or committed path selection.
- Movement-preview evidence: the swarm load smoke must include bounded preview
  samples from swarm intents, flow-field candidate queries with no unreachable
  preview candidates, cloned-world physics pressure/corrections, and at least
  one blocked preview delta under local pressure. Preview physics correction
  distance counters must remain positive when correction count is positive.
- Movement-apply evidence: the swarm load smoke must include bounded opt-in
  `SwarmState` movement apply samples from flow-field candidates and local
  collision physics. It must also include flow-field cache request and hit
  counters for repeated same-target samples, flow-field cache entry and eviction
  counters for the deterministic 32-entry cap, plus position-sync counters
  proving sample final positions are updated after the local physics step.
  Movement-apply physics correction distance counters must remain positive when
  correction count is positive. The default movement apply must remain
  unclamped for both the collision movement probe and the later physics step,
  and a separate clamped movement-apply smoke sample must prove the opt-in
  correction cap is carried through both layers. This proves the explicit apply
  path only; it does not mean snapshots run movement automatically.
- Batch movement-apply evidence: the swarm load smoke must include bounded
  opt-in `SwarmState::apply_flow_field_batch_movement_step` samples with
  flow-field cache requests/hits, no cache evictions, at least one applied
  sampled delta, bounded physics candidates, physics iterations, and position
  sync counters. The default batch apply must remain unclamped. This proves
  simultaneous candidate apply evidence only; it is not default tick-loop
  movement or final avoidance.
- Batch movement-tick/config evidence: the swarm load smoke must include bounded
  `SwarmState::tick_with_batch_flow_field_movement` samples, full-snapshot bytes,
  AOI delta-snapshot bytes, and a configured
  `SwarmConfig::BatchFlowFieldCollision` tick through `tick_with_focus` with the
  batch apply claim scope. This proves explicit configured batch movement only;
  spawn-only smoke config, live transport, and final avoidance remain separate
  gates.
- Movement-tick evidence: the swarm load smoke must include bounded
  `SwarmState::tick_with_flow_field_movement` samples. This proves only the
  opt-in spawn/report plus movement tick. It must also include bounded
  configured `SwarmConfig::movement_mode` tick evidence, including a configured
  clamped movement tick that proves the optional correction-limit policy flows
  through `tick_with_focus` into movement probes and local physics, plus a
  two-tick configured movement loop with cache
  requests/hits, physics iterations, and moved-entity evidence. Default
  spawn-only smoke config and snapshot movement replication remain separate
  gates.
- Static-obstacle movement evidence: the swarm load smoke must include bounded
  static obstacle count, flow-field blocker-cell count, and movement samples
  that run against a collision world containing immovable static obstacles from
  a validated local map-data import. The blocker-cell count must prove obstacle
  half-extents plus swarm-radius clearance expand beyond one center cell. This
  proves local map-data bridge integration only; live/exported map-data obstacle
  evidence remains a separate gate.
- Movement snapshot evidence: the swarm load smoke must include bounded full
  snapshot entity count, bytes, and bandwidth after the opt-in movement tick.
  It must also include bounded AOI-visible delta snapshot entity count, bytes,
  and bandwidth through the shared interest delta builder. A second smaller AOI
  movement delta must include visible moved entities plus aggregate far-state for
  distant moved zombies. This proves local snapshot materialization only; live
  transport replication and default tick-loop emission remain separate gates.
- Batch movement replication evidence: the dedicated batch movement replication
  smoke must include a full baseline, configured batch movement tick, full
  movement snapshot, all-visible AOI delta with at least one changed visible
  entity, and a smaller AOI delta with both changed visible entities and
  aggregate far-state. This proves local server movement-to-snapshot
  materialization only; live transport, client prediction, and public-network
  behavior remain separate gates.
- Local elapsed-time evidence: the swarm load smoke must include positive
  local microsecond counters for spawn ticks, behavior, preview, opt-in movement
  tick, configured movement tick, configured movement loop, static-obstacle
  movement, snapshots, collision diagnostics, and total smoke time. These
  counters are regression evidence only and do not satisfy measured p95
  server-tick or release-budget claims.
- Movement-loop measurement evidence: when the configured movement loop or its
  performance wiring changes, the dedicated measurement harness must keep at
  least three local samples, positive p50/p95/p99 elapsed microsecond evidence,
  cache hits, physics iterations, and moved-entity evidence while remaining
  `budget_result=blocked`.
- Contract docs:
  `docs/gameplay/swarm-behavior-contract.md` and
  `docs/gameplay/swarm-warning-feedback.md`.

## No-Go Conditions

Record No-Go rather than expanding scope when:

- Godot decides swarm behavior or collision authority,
- collision contacts are treated as movement resolution,
- collision admission rejects are treated as pushback or alternate path
  selection without an explicit movement/collision slice,
- collision resolution plans are applied to authoritative movement without a new
  movement/collision slice,
- local CollisionWorld physics steps are treated as gameplay movement,
  snapshot truth, or Godot physics,
- resolved-admission candidate positions are treated as selected paths,
  pushback, or authoritative snapshot truth,
- opt-in axis-slide probe evidence is treated as default swarm movement, full
  pathfinding, final avoidance, live transport, or release readiness,
- movement-apply or configured movement-tick evidence is treated as
  unconditional tick-loop movement, snapshot truth, or final horde pathfinding,
- movement-preview deltas are treated as committed horde movement, pathfinding,
  or gameplay physics,
- aggro markers are generated without server/adapter evidence,
- 1,000-proxy readability is presented as measured frame-rate evidence,
- live Steam, public networking, release, or balance claims are implied.

## Handoff

After a passing local smoke, the next safe work is collision-resolution
admission design or a measured performance harness. Both require a new slice
because GSWARM-04 only defines warning language and debug trigger rules.
