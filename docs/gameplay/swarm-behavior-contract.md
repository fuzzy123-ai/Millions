# Swarm Behavior Contract

Status: GSWARM-02 repo-only server preparation.

`server/src/swarm.rs` owns the current zombie swarm behavior model. The model is
deterministic and server-authoritative, but intentionally narrow:

- the server schedules swarm spawn timing and entity IDs,
- active swarm entities keep a route-pressure target,
- aggro stimuli are stored as short-lived trail samples,
- AI LOD is evaluated from a server-side focus point,
- swarm entities can export simple collision bodies for later collision work.

## Aggro Trail Split

Aggro uses two lanes:

- `Direct`: the stimulus is fresh enough to drive immediate swarm intent.
- `Memory`: the stimulus has aged out of direct aggro but remains a trail target.

Expired stimuli are ignored by behavior reports. If no direct or memory stimulus
is active, swarm entities fall back to their route-pressure target.

When multiple stimuli exist, the server chooses the deterministic highest
priority sample by lane, strength, age, and source entity ID. This is a behavior
input, not final combat AI or perception.

## AI LOD Model

`SwarmState::evaluate_behavior` assigns each active swarm entity one of:

- `Full`: close enough to the focus point for per-entity behavior updates.
- `Reduced`: mid-distance entities keep cheaper behavior.
- `Aggregate`: far entities are candidates for aggregate updates.

The current report is a model boundary and test surface. It does not yet perform
movement, pathfinding, steering, animation, replication, or render culling.

## Collision Preparation

`server/src/collision.rs` owns the first collision-prep boundary:

- circle bodies with authoritative entity IDs,
- deterministic spatial-cell broadphase,
- overlap contacts sorted by entity ID,
- circle probes for later movement/collision admission checks,
- target-position admission that accepts clear positions and rejects overlaps,
- deterministic resolution plans that describe correction vectors without
  applying movement,
- bounded local physics steps that apply those corrections only inside a
  `CollisionWorld`,
- immovable `StaticObstacle` bodies for local resolution diagnostics,
- resolved-admission probes that test a candidate target in a cloned
  `CollisionWorld` after bounded local resolution,
- movement/collision probes that expose requested and resolved deltas plus a
  local candidate decision without mutating authoritative positions.

`SwarmEntity::collision_body` exposes a simple swarm body using the configured
swarm collision radius. This prepares collision integration without claiming
final physics, unit separation, obstacle resolution, navmesh blocking,
pushback, damage, or Godot collision behavior.

Collision admission is a yes/no server boundary for a candidate target position.
It ignores the moving body itself, returns deterministic blocking entity IDs, and
reports unknown bodies without panicking. It does not yet choose an alternate
position, slide along obstacles, separate units, or update authoritative
movement state.

Collision resolution planning turns detected overlaps into deterministic
per-entity correction vectors. The plan is evidence and future input only: it
does not mutate `CollisionWorld`, does not update authoritative snapshots, and
does not choose gameplay movement outcomes.

Static obstacles are immovable during local resolution. Dynamic bodies may be
corrected away from them, but static-static contacts remain unresolved
diagnostics for future map and navigation validation.

The local collision physics step applies a resolution plan to `CollisionWorld`
body positions and rebuilds collision cells for a bounded number of iterations.
It reports applied correction count, total applied correction distance, and max
single correction distance as local physics diagnostics only. It also exposes a
separate clamped correction step for local stability evidence when large
corrections should be bounded.
This is the first apply-stage physics boundary, but it is still not connected to
authoritative gameplay movement, snapshot emission, pathfinding, Godot physics,
or balance.

Resolved admission asks whether a requested target position is already clear,
can become clear after bounded local resolution, remains overlapping, or refers
to an unknown body. It reports the candidate resolved position, the initial
blocking IDs, and pressure counters for future movement code. Flow-field and
swarm load smokes use this resolved-admission surface through
`CollisionMovementProbe` as the shared movement/collision probe, while keeping
authoritative entity positions unchanged. It does not choose alternate paths,
slide against obstacles, move authoritative entities, emit snapshots, or apply
gameplay pushback.

`CollisionWorld::probe_batch_movements_after_resolution` extends that diagnostic
to a bounded set of simultaneous movement candidates. It clones the local
collision world, places all known candidate bodies at their requested positions,
runs bounded overlap resolution, and reports accepted/corrected/blocked/unknown
sample decisions plus aggregate correction distance counters. This is batch
admission evidence for future swarm movement only; it is not final avoidance,
path selection, or committed gameplay movement by itself.

Flow-field scale evidence may apply the probe decision to its bounded local
sample by using resolved deltas for accepted/corrected candidates and zero deltas
for blocked candidates. That sample application is regression evidence only; it
is not an authoritative movement loop or final horde pathing.

Swarm load evidence may also build bounded movement-preview samples from
server-owned swarm intents. The preview builds local flow fields for the sampled
route or aggro targets, queries short candidate deltas, runs those candidates
through `CollisionMovementProbe`, and records whether they would be applied or
blocked. Accepted or corrected preview candidates are then applied to a cloned
`CollisionWorld` for one bounded local physics step, producing contact-pressure
and correction counters for regression evidence. The load smoke also runs the
sampled preview candidates through the batch movement probe so simultaneous
candidate pressure is visible. The preview is not committed to `SwarmState` and
does not emit authoritative snapshots.

`SwarmState::apply_flow_field_movement_step` is the first opt-in swarm movement
apply path. It uses the same server-owned intents, flow-field candidates, and
collision movement probes as preview, applies accepted or corrected positions to
`SwarmState`, runs bounded local `CollisionWorld` physics, and syncs corrected
body positions back to active swarm entities. This is server-side authoritative
state mutation when explicitly called. The apply path reuses a bounded local
flow-field cache for repeated same-target fields with the same blocker set. The
cache has a deterministic 32-entry cap and reports evictions and current entry
count so movement evidence can catch hidden cache growth.
`SwarmState::apply_flow_field_movement_step_with_correction_limit` exposes the
same apply path with an explicit local correction cap for stability evidence.
The cap now flows through both the collision movement probe that decides
accepted/corrected/blocked candidate movement and the later bounded
`CollisionWorld` physics step. The default apply path remains unclamped.
Movement-apply samples report final positions after the local physics sync so
their sample deltas match the active swarm state.
`SwarmState::apply_flow_field_batch_movement_step` is a separate opt-in apply
path for bounded simultaneous candidate movement. It builds the same flow-field
candidate set, resolves those candidates through
`CollisionWorld::probe_batch_movements_after_resolution`, applies only accepted
or corrected resolved positions, then runs the same bounded local physics sync.
The default tick path does not use this batch apply path automatically.
`SwarmState::apply_flow_field_batch_movement_step_with_correction_limit` carries
the same explicit correction cap through the batch probe and the later local
physics step for stability evidence.
`SwarmState::tick_with_batch_flow_field_movement` wraps spawn/report plus the
batch apply path into an explicit opt-in movement tick. `SwarmConfig` can select
`BatchFlowFieldCollision` through
`with_batch_flow_field_collision_movement`, causing `tick_with_focus` to use the
batch apply path while preserving the spawn-only default. Batch movement tick
snapshots use the same full and AOI delta snapshot builders as the single-probe
movement tick and are local evidence only, not live transport replication.
`SwarmState::tick_with_flow_field_movement`
wraps the normal spawn/report tick and this apply path into a controlled opt-in
movement tick. `SwarmConfig::movement_mode` can enable the same flow-field and
collision apply path from the regular tick through `tick_with_focus`, while the
local smoke config remains spawn-only by default. `SwarmConfig` can also carry
an optional movement correction limit so the regular configured tick path can
run the same clamped probe and local physics policy; the default remains `None`
and unclamped. The load smoke also runs a bounded two-tick configured movement loop
to prove repeated tick calls can continue applying movement, reusing the local
flow-field cache, and mutating positions without changing the default spawn-only
config. `SwarmState::set_static_obstacles`
adds local `StaticObstacle` bodies to movement collision worlds and flow-field
blocker cells; `SwarmState::set_static_obstacles_from_map_data` builds those
bodies from validated local map-data obstacles without adding them to swarm
snapshots or active zombie counts. Map-data obstacle half-extents expand to
flow-field blocker cells with swarm-radius clearance, clipped to the local
movement bounds.
`SwarmState::snapshot_entities` and `SwarmState::build_full_snapshot` expose the
resulting active swarm positions through the shared server snapshot model for
bounded full-snapshot evidence.
`SwarmState::build_interest_delta_snapshot` maps the same moved positions
through the shared interest/AOI delta snapshot builder for local replication
evidence. The local swarm load smoke covers both an all-visible movement delta
and a smaller AOI movement delta where distant moved zombies are represented by
aggregate far-state. Unconditional swarm tick-loop movement, live transport
replication, final pathfinding, and final avoidance are still not automatic.
`run_swarm_batch_movement_replication_smoke` adds a focused 1,000-zombie local
replication check for the batch movement path. It captures a full baseline
snapshot, runs a configured batch flow-field/collision tick, builds a new full
snapshot plus AOI deltas, and counts visible entities whose positions changed
from the baseline. This proves local server snapshot materialization after
batch movement only; live delta transport, client prediction, final avoidance,
and release readiness remain separate gates.

The batch-vs-single movement-loop measurement harness uses the same seeded
1,000-zombie base state to compare configured single-probe and batch-probe
movement ticks. It records local timing shape and movement/physics/cache
counters for regression tracking only. It is not a balance decision, formal
performance budget, or final pathfinding/avoidance claim.

The swarm load smoke records local elapsed-time microsecond counters for the
spawn, behavior, preview, opt-in movement, configured movement loop,
static-obstacle movement, snapshot, collision-diagnostic, and total smoke
stages. These counters are regression
evidence for performance-sensitive swarm/collision changes only; they are not
formal p95 server-tick, render-frame, live-bandwidth, or release-budget claims.

## Non-Claims

GSWARM-02 and the collision follow-up do not add final horde AI, pathfinding,
avoidance, unconditional tick-loop movement, live delta transport, attacks,
damage, capture pressure, economy pressure, balance, live networking, protocol
payloads, Godot presentation, or release readiness.

Future slices must add measured 1,000-zombie load evidence, aggregate far-state
replication rules, player-facing warning language, debug triggers, and
live/exported map-data obstacle evidence before broader gameplay claims are
made.
