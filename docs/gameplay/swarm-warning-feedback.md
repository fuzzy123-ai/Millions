# Swarm Warning And Feedback

Status: GSWARM-04 repo-only player-facing language.

The Rust server owns swarm timing, spawned entity IDs, aggro trail state, AI LOD,
and future collision admission. Godot may present feedback only from server or
adapter evidence; it must not decide horde behavior.

## Player-Facing States

Use these state words for future UI, audio, logs, and debug overlays:

| State | Meaning | Allowed presentation |
| --- | --- | --- |
| `swarm_dormant` | Swarm timer has not reached its start tick. | No warning, optional debug-only timer. |
| `swarm_gathering` | Spawn has started but active count is below warning threshold. | Subtle map-edge or minimap hint. |
| `swarm_pressure` | Route-pressure buckets are active and moving toward objectives later. | Directional warning and density hint. |
| `swarm_aggro_direct` | Fresh aggro stimulus is driving immediate intent. | Strong warning at stimulus area. |
| `swarm_aggro_memory` | Older aggro trail remains relevant. | Fading trail or caution marker. |
| `swarm_collision_pressure` | Collision-prep contacts exceed debug threshold. | Debug density marker only until collision resolution exists. |
| `swarm_degraded` | AI LOD or load shedding uses aggregate behavior. | Read-only performance/debug marker. |
| `swarm_blocked_validation` | Required evidence is missing or blocked. | Developer/operator state, not a player promise. |

## Feedback Rules

- Warnings must be grounded in server-owned swarm reports or bounded local
  adapter smokes.
- Direct aggro should read as urgent; memory aggro should read as fading.
- Aggregate LOD must not hide authoritative entities from snapshots without a
  later protocol and interest-management slice.
- Collision-pressure feedback must not imply physical blocking, damage, or
  pushback until collision resolution exists.
- A 1,000-proxy readability smoke is not a p95 frame-rate signoff.

## Readability Stop Rules

Stop and mark the slice blocked or deferred if:

- swarm density markers overlap command feedback or cover/combat feedback,
- direct and memory aggro cannot be visually distinguished,
- collision-pressure markers look like final physics,
- aggregate LOD hides the fact that the server remains authoritative,
- a report claims pass/release readiness without measured p95 evidence.

## Non-Claims

This document does not approve final UI design, audio, animation, minimap
language, horde balance, collision behavior, pathfinding, live networking,
Steam validation, or release-candidate readiness.
