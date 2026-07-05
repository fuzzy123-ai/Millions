# Movement Model Options

Date: 2026-07-03
Slice: `NAV-01`
Status: repo-only movement option contract

## Purpose

This document defines safe movement model options for later large-army
pathfinding work. It is an option matrix, not a design choice, gameplay
implementation, balance decision, or performance claim.

The server remains authoritative for final positions, movement validity,
pathing, collision, and map constraints. Godot may preview movement and render
feedback, but future snapshots must be able to correct every local prediction.

## Current Foundation

The server already has these non-gameplay foundations:

- deterministic 20 Hz tick loop,
- abstract `EntityState` and `WorldPosition` in millimeter units,
- `MovementDelta` stubs for deterministic scale scenarios,
- SoA-friendly `EntityColumns`,
- deterministic `SpatialGrid` cell assignment,
- mapdata_v0 contract, validation shape, checksum fixture, and change runbook,
- a local deterministic flow-field cost map for the shared-objective movement
  scale scenario,
- bounded collision-admission samples for flow-field candidate steps in local
  movement-scale evidence,
- bounded resolved-admission samples for the same flow-field candidates through
  the local collision physics boundary, including correction-count and
  correction-distance diagnostics,
- bounded local apply-physics samples that sync corrected flow-field sample
  body positions back into the deterministic movement-scale entity columns.

These foundations are enough to define options and future perf scenarios. They
are not enough to ship movement-heavy gameplay.

## Option Matrix

| Option | Server model | Best fit | Main risk | Evidence needed before claim |
| --- | --- | --- | --- | --- |
| Direct target steering | Server accepts target intent and steps units toward target with simple bounds checks. | Early command loop and replay coverage. | Poor obstacle handling and clumping at scale. | Deterministic replay, 1k/5k/10k step cost, rejection cases. |
| Grid corridor pathing | Server maps targets to grid cells and follows a coarse corridor through map-data blockers. | Large maps with simple blockers and predictable costs. | Corridor quality can be too rough for tactical readability. | Path query budget, map checksum fixture, blocked-cell cases, Godot readability smoke. |
| Flow-field per objective | Server builds or caches a direction field per destination or capture objective. | Many units moving to shared objectives. | Field rebuilds can dominate ticks when goals change often. | Field build/cache budget, invalidation rules, replay checksum, 64-entry memory ceiling. |
| Formation anchor with local offsets | Server moves an anchor and validates follower offsets around it. | Squads, lines, and readable group movement. | Formation maintenance can hide collision and avoidance costs. | Formation stress cases, follower correction metrics, render readability check. |
| Local avoidance layer | Server applies deterministic short-range separation after primary pathing. | Preventing pileups near choke points. | Pair checks can explode without spatial partitioning. | Spatial-cell neighbor budget, cap/degrade rules, deterministic ordering tests. |

The options can be combined later, but each combined path must still have a
bounded server cost model and deterministic ordering before it can affect
authoritative gameplay.

## Required Server Boundaries

Future movement code must preserve these boundaries:

- client command payloads are intent only, never client-computed truth,
- server validates target position, map bounds, blocker constraints, and stale
  command behavior,
- all movement decisions are deterministic for a given tick/input/map checksum,
- all iteration over entities, cells, and commands uses stable ordering,
- overload or expensive-path cases degrade predictably instead of extending the
  tick without bounds,
- movement outputs are snapshot state, not Godot scene or node state.

## Future Scenario Inputs

NAV-02 should add movement perf scenarios before any option is selected. Minimum
scenario families:

- 1k/5k/10k units moving to one shared target,
- many small groups moving to separate targets,
- blocker corridor stress using the mapdata_v0 fixture shape,
- choke-point pressure with deterministic local avoidance disabled and enabled,
- formation anchor stress with follower correction counts,
- stale or invalid move command rejection cases.

Each row should record entity count, map checksum, option under test, tick count,
query count, correction count, memory estimate, and whether the result is
informational, blocked, pass, or fail.

## Open Design Gate

Gate: `G-MOVEMENT-SCALE`
Class: `needs_design`
Blocks: selecting final movement/pathfinding/formation/avoidance behavior and
claiming movement-heavy gameplay readiness
Safe preparation done: option matrix, server boundaries, scenario families, and
non-claims
Risk if bypassed: movement cost can become the hidden dominant server budget and
Godot previews can drift from authoritative snapshots
Next safe slice: `NAV-02`

## Non-Claims

NAV-01 does not implement movement, pathfinding, formations, avoidance,
collision, cover, attack ranges, capture behavior, unit balance, animation,
Godot preview UX, live networking, or release-candidate evidence.
