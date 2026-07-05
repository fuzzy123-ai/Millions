# Millions Server

Status: foundation backend scaffold

The Rust server is the future authoritative simulation host. Current foundation
scope is intentionally small:

- `src/build_info.rs`: deterministic package/protocol/fixture/replay build
  evidence for local reports.
- `src/collision.rs`: deterministic circle-body collision-prep boundary,
  broadphase overlap probes, admission checks, resolution plans, bounded local
  physics steps, and local collision perf smoke scenarios.
- `src/cover.rs`: first server-owned obstacle, cover, line-of-sight,
  line-of-fire, and cover occupancy data model derived from validated map data.
- `src/protocol.rs`: protocol_v0 header constants and decoding.
- `src/fixtures.rs`: shared fixture inventory for Rust tests.
- `src/game_core.rs`: first local abstract gameplay core for authoritative HQ
  creation, one basic squad per started player, and server-validated move
  intents.
- `src/swarm.rs`: first authoritative zombie swarm timer, gradual spawn, and
  route-pressure baseline.
- `src/transport.rs`: local/mock/UDP transport envelope boundary.
- `src/interest.rs`: deterministic AOI regions and per-client subscription
  state for future snapshot filtering.
- `src/simulation.rs`: deterministic 20 Hz tick-loop scaffold and authority
  identifiers, abstract entity state, movement stub, snapshot builder,
  SoA-friendly entity columns, and deterministic spatial grid baseline.
- `src/simulation_scale.rs`: deterministic 1k/5k/10k simulation-only scenario
  catalog and runner for scale inputs.
- `src/movement_scale.rs`: deterministic movement-scale scenario catalog and
  local flow-field cost map for shared-objective movement pressure.
- `src/metrics.rs`: server tick, snapshot size, and bandwidth report helpers.
- `src/observability.rs`: planned counter names and local counter snapshots for
  logging/debug/evidence surfaces.

SIM-01 keeps the scale foundation deliberately plain-Rust: `EntityColumns` stores
the abstract entity state in column vectors and `SpatialGrid` rebuilds stable
cell membership with `BTreeMap` ordering. This prepares server benchmarks and
interest management without choosing a long-term ECS crate, pathfinding model,
combat rule, economy rule, or gameplay behavior. `G-ECS-CHOICE` remains open for
the later design decision.

SIM-02 adds deterministic scale scenarios that run abstract entities through
movement stubs, spatial-grid rebuilds, and full snapshot byte counting at 1k,
5k, and 10k entity counts. These are scenario inputs only, not wall-clock
performance claims.

NAV-01 documents future movement model options and server authority boundaries
in `docs/architecture/movement-model-options.md`. The current `MovementDelta`
paths remain deterministic stubs for scale evidence; they are not pathfinding,
formation, avoidance, collision, or gameplay movement.

The flow-field follow-up adds a deterministic local `FlowFieldMap` over
`SpatialCell`s for the shared-objective movement scenario. It builds cell costs
from a goal around blocker cells and returns a bounded next-cell movement delta
for each entity query. This is still local scale evidence, not final pathfinding,
map navigation authority, formation behavior, collision integration, or gameplay
movement.

Flow fields are cached by bounds, objective cell, blocker cells, and map
checksum. The cache reports requests, builds, hits, and explicit checksum
invalidations. It has a deterministic 64-entry cap with eviction counters so
later gameplay movement can prove it is reusing shared-objective fields instead
of rebuilding them every tick without allowing hidden memory growth.

The flow-field scale run also creates a bounded local collision-admission sample
for candidate flow steps. It includes sampled moving unit bodies plus static
obstacle bodies derived from blocker cells, then records accepted and rejected
candidate admissions. The same sample now also runs bounded resolved-admission
attempts through the local `CollisionWorld` physics boundary and records
still-overlapping, iteration, correction counts, and total/max correction
distance. It now also applies accepted/corrected sampled flow-field deltas to a
bounded local sample `CollisionWorld`, runs one local physics step, and records
contact pressure, correction distance, and sample position sync counts. This
connects flow-field direction evidence to collision diagnostics without claiming
final pathfinding, avoidance, or authoritative movement.

INT-01 adds server-owned interest management. `AoiRegion` selects deterministic
spatial-grid cells and `InterestManager` stores per-`PlayerSessionId`
subscriptions plus visible/entered/left entity sets. Snapshot payload encoding,
aggregate far state, and bandwidth claims remain later INT slices.

No gameplay rules live here yet. Gameplay slices must wait until the protocol,
authority, Godot interface, determinism, and performance gates remain documented
and testable.

GCORE-01 opens only the local abstract playtest core: HQs, basic squad spawn,
and move intent acceptance are server-owned. It intentionally does not add
pathfinding, combat, economy, final unit roles, live networking, or balance
claims.

GCOV-01 adds only the server-side cover authority data model. It imports
validated mapdata obstacles and cover objects into deterministic rectangular
volumes, supports point lookup, line-of-sight/line-of-fire blocker queries, and
bounded cover occupancy by authoritative entity ID. It does not add attack
commands, damage, cover bonuses, targeting legality, pathfinding, final combat
balance, Godot UI, live networking, or measured performance claims.

GCOV-02 adds a range-first targeting query that classifies clear, target-in-cover,
blocked-by-obstacle, and out-of-range cases. It still does not add attack command
admission, damage, hit/miss rolls, cover bonuses, cooldowns, weapon stats, final
balance, protocol events, or Godot UI behavior.

GCOV-04 adds a local dense targeting smoke for 64 attacker positions and 96
target positions. The smoke verifies result-bucket coverage for clear, in-cover,
blocked, and out-of-range classifications, but remains informational and blocked
until measured p95 query-cost evidence exists.

GSWARM-01 adds only the server-owned zombie swarm scheduling baseline.
`SwarmState` owns start tick, spawn interval, batch size, active cap,
deterministic spawn-point rotation, and aggregate route-pressure samples by
target position. It does not add movement, pathfinding, aggro, AI LOD, attack
commands, damage, economy pressure, protocol payloads, Godot UI, final balance,
or performance budget claims.

GSWARM-02 adds deterministic aggro trail splitting and an AI LOD behavior report.
Fresh stimuli produce direct aggro intent, aged stimuli become memory trail
intent, and expired stimuli fall back to route-pressure targets. `collision.rs`
and `SwarmEntity::collision_body` prepare circle-body broadphase data for later
collision work, but do not resolve movement, obstacle blocking, unit separation,
pushback, damage, Godot physics, or final gameplay collision.

GSWARM-03 adds a local 1,000-zombie load smoke. It records active swarm count,
route-pressure buckets, direct and memory aggro trails, AI LOD counts,
collision-prep bodies, broadphase contacts, estimated snapshot bytes, and an
informational blocked budget result. The collision section now includes
target-position admission, bounded resolved-admission, resolution-plan, and
local physics-step counters. It is not measured p95 performance, final horde AI,
gameplay-applied collision resolution, live networking, or release readiness.

The collision follow-up adds target-position admission to `collision.rs` and the
swarm load smoke. The server can now say whether a candidate body position is
accepted, rejected by overlap, or unknown. It still does not move entities,
choose alternate positions, push bodies apart, resolve obstacle contact,
pathfind, or expose collision results over the protocol.

The next collision follow-up adds deterministic resolution plans. The server can
now convert overlap contacts into per-entity correction vectors for evidence and
future movement integration. Those plans are not applied to authoritative
positions, snapshots, Godot physics, or pathfinding.

The local physics-step follow-up applies those correction vectors inside an
isolated `CollisionWorld` for bounded iterations and rebuilds collision cells.
It also reports total and max applied correction distance so local physics
pressure is visible beyond correction counts. A separate clamped local step can
cap each applied correction for stability diagnostics and reports how many
corrections were clamped.
This is a server-side physics boundary for tests and future integration, not
gameplay movement, snapshot truth, pathfinding, or Godot physics.

The collision physics smoke follow-up adds deterministic 1,000-body and
5,000-body clustered `CollisionWorld` scenarios with local elapsed-time
recording. It is a regression guard for collision pressure only: the report is
informational and blocked until measured p95/p99 tick evidence and formal
collision budgets exist.

The resolved-admission follow-up clones a candidate `CollisionWorld`, moves one
body to a requested target, runs bounded local overlap resolution, and reports
whether the candidate is clear, clear after resolution, still overlapping, or
unknown. It also carries the initial blocking IDs, so movement and swarm smokes
can derive target-position admission and resolved-admission evidence from the
same probe. This is still only an admission diagnostic for future movement code,
not authoritative movement, alternate path selection, sliding, or gameplay
pushback.

The movement-probe follow-up wraps resolved admission into a reusable
`CollisionMovementProbe` with requested and resolved deltas plus an
`Accepted`/`Corrected`/`Blocked`/`UnknownBody` decision. Flow-field and swarm
smokes use those decisions for local candidate evidence only; no authoritative
positions or snapshots are mutated by the probe.
`CollisionWorld::probe_batch_movements_after_resolution` now applies the same
diagnostic shape to a bounded set of simultaneous movement candidates, recording
accepted/corrected/blocked/unknown counts and aggregate correction distances for
future swarm batching evidence.

The flow-field smoke now lets the bounded sampled candidates apply the
movement-probe decision locally: accepted and corrected candidates use the
probe's resolved delta, while blocked or unknown candidates receive a zero
delta. This is still limited to smoke-sample movement evidence and does not make
flow fields the authoritative gameplay movement system.

The swarm load smoke now also creates a bounded non-authoritative movement
preview from swarm behavior intents. Preview samples build local flow fields for
their selected route or aggro target, query short candidate deltas from those
fields, run the candidates through `CollisionMovementProbe`, and record whether
the local candidate delta would be applied or blocked. Accepted or corrected
preview candidates are also applied to a cloned `CollisionWorld` for one bounded
local physics step, so the smoke can see batch contact pressure and correction
cost without mutating swarm positions or snapshots. The same sampled candidates
also run through the batch movement probe so simultaneous candidate pressure is
visible. This connects swarm intent, flow-field candidate selection, collision
movement probes, and local physics preview evidence while keeping gameplay
authority unchanged.

The swarm movement-apply follow-up adds an explicit
`SwarmState::apply_flow_field_movement_step` path. It uses the same
server-owned behavior intents, local flow-field candidate steps,
`CollisionMovementProbe` decisions, and bounded `CollisionWorld` physics, then
syncs the resulting body positions back into `SwarmState`. The apply path uses
a bounded state-local flow-field cache keyed by bounds, target cell, blocker
cells, and cell size so repeated same-target samples do not rebuild identical
fields in one movement step. The cache has a deterministic 32-entry cap and
reports evictions plus current entry count for memory-growth regression
evidence. A separate
`SwarmState::apply_flow_field_movement_step_with_correction_limit` path runs the
same apply logic with an explicit local correction cap for stability evidence.
That cap applies to the collision movement probe and the later local physics
step; the default apply path remains unclamped. The swarm load smoke
executes this through `SwarmState::tick_with_flow_field_movement` on a cloned
swarm state for bounded opt-in tick evidence. `SwarmConfig::movement_mode` can
also enable the same flow-field/collision apply path from the regular tick via
`tick_with_focus`; `SwarmConfig::local_scale_smoke()` still defaults to
spawn-only. `SwarmConfig::movement_correction_limit_abs_mm` can additionally
enable the same clamped movement-probe and local-physics policy in that
configured tick path, while the default remains unset and unclamped. The swarm
load smoke also runs a
bounded two-tick configured movement loop to prove repeated configured ticks
continue to move sampled zombies, hit the local flow-field cache, and execute
bounded local physics without making default tick-loop movement automatic. No
final pathfinding/avoidance behavior is claimed.
`SwarmState::apply_flow_field_batch_movement_step` adds a separate opt-in
apply path for simultaneous sampled candidates. It resolves the bounded
flow-field candidate set through the batch movement probe, applies accepted or
corrected resolved positions, and syncs the later local physics result back to
active swarm entities. Its correction-limit variant carries the same explicit
cap through the batch probe and physics step. The default configured movement
path still uses the single-probe apply path unless this batch API is called
directly.
`SwarmState::tick_with_batch_flow_field_movement` adds the matching explicit
spawn/report plus batch movement tick wrapper, and
`SwarmConfig::with_batch_flow_field_collision_movement` can route
`tick_with_focus` through the same batch apply path. The swarm load smoke records
bounded batch tick counters, configured batch tick counters, and full/AOI delta
snapshot bytes after batch movement. Spawn-only remains the default local smoke
mode, and live transport replication is still a separate gate.
GSWARM-13 adds `run_swarm_batch_movement_replication_smoke`, a focused local
1,000-zombie check that captures a baseline snapshot, runs configured batch
movement, and verifies that AOI deltas contain visible entities whose positions
changed from the baseline. This is local server movement-to-snapshot evidence
only; packet transport, client prediction, public-network replication, and
release readiness remain separate gates.
`SwarmState::set_static_obstacles` and
`SwarmState::set_static_obstacles_from_map_data` let the local movement path
include `CollisionBodyKind::StaticObstacle` bodies as both flow-field blocker
cells and immovable collision bodies; the swarm load smoke records the
validated local map-data obstacle bridge separately from the base 1,000-zombie
counts. Map-data obstacle half-extents expand to bounded local flow-field
blocker cells with swarm-radius clearance rather than only blocking the
obstacle center cell.

`SwarmState::snapshot_entities` and `SwarmState::build_full_snapshot` now expose
the opt-in movement tick state through the shared server snapshot model. The
swarm load smoke records full-snapshot entity count, bytes, and bandwidth after
movement. `SwarmState::build_interest_delta_snapshot` uses the shared interest
delta builder for local AOI-visible movement deltas and records delta bytes and
bandwidth in the smoke. The same smoke records positive local elapsed-time
evidence for spawn ticks, behavior, preview, opt-in movement apply, configured
movement tick, configured movement loop, static-obstacle movement, snapshots,
collision diagnostics, and total smoke time without claiming p95 server-tick
performance. Live transport replication and unconditional tick-loop snapshot
emission remain future gates.

GSWARM-05 adds a separate local measurement harness for that configured movement
loop. It runs cloned samples from the same 1,000-zombie base state, records
p50/p95/p99 elapsed microsecond shape, and keeps cache, physics, and
moved-entity counters visible without claiming a formal swarm p95 budget.
GSWARM-10 adds a paired batch-vs-single movement-loop comparison harness for the
same base state and bounded two-tick loop. It records both single-probe and
batch-probe p50/p95/p99 shape plus a batch/single p95 ratio in basis points, but
it remains local regression evidence only and does not claim batch is faster or
within a formal server tick budget.

Static obstacle resolution now treats `CollisionBodyKind::StaticObstacle` as
immovable inside local plans and physics steps. Dynamic bodies can be corrected
away from static obstacles, while static-static contacts stay unresolved for
future map/nav validation rather than moving authored blockers.

Local smoke:

```powershell
C:\Users\nkatz\.cargo\bin\cargo.exe test
```
