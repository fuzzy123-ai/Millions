# Millions Gameplay Roadmap Integration

Date: 2026-07-02
Source handoff: `E:/TankGame/docs/gameplay_plan_handoff.md`
Depends on: `millions-master-roadmap.md`, `millions-roadmap-suite.md`, `millions-risk-gap-register.md`
Mode: Standard ABC

## Integration Rule

The gameplay handoff becomes the first real validation target for the
infrastructure, not a replacement for the infrastructure roadmap.

That means:

- Use abstract readable gameplay first.
- Rebuild gameplay as server-authoritative systems.
- Treat existing TankGame behavior as requirements and test inspiration, not as
  code to copy.
- Prove local multi-client, Steam compatibility, reconnect, packet loss, and
  performance continuously.
- Delay final art until readability, performance, and playtest loop are stable.

## Where It Fits In The Master Roadmap

| Master phase | Gameplay role |
| --- | --- |
| FND | Convert gameplay into constraints: authority, protocol needs, scale targets, and non-goals. |
| LOOP | Do not build gameplay yet, except command/snapshot shapes needed for abstract entities. |
| RES | Ensure gameplay commands can survive resend, reconnect, packet loss, and duplicate delivery. |
| GPLAN | Create gameplay scenario contracts and faction load profiles. |
| SCALE | Validate large unit counts before feature richness. |
| GLOOP | Build the first abstract 2-player RTS loop. |
| HORDE | Add unit-role mix and zombie horde pressure as scale tests. |
| FLOW | Preserve Steam match flow, runbooks, release gates, and later art return. |

## Gameplay Roadmaps Added

| Kürzel | Name | Purpose |
| --- | --- | --- |
| GPLAN | Gameplay Scenario Contracts | Turns the gameplay handoff into infrastructure-ready contracts. |
| GLOAD | Faction Scale Load Matrix | Defines huge-unit scenarios for all factions as measurable tests. |
| GCORE | Two-Player Playtest Core | First abstract match loop: HQ, squads, move/attack/cover, credits, capture, swarm trigger. |
| GCTRL | RTS Controls And Readability | Selection, camera, command context, UI readability under large visible counts. |
| GCOV | Cover Combat Authority | Server-side cover, LOS/LOF, targeting, and firefight feedback. |
| GECON | Capture Economy Match Loop | Credits, capture points, production spend, and match pacing. |
| GROLE | Unit Role Expansion | Adds roles one at a time with tactical purpose and authority path. |
| GSWARM | Zombie Horde Scale Event | Late-match horde as gameplay pressure and performance load. |
| GBAL | Match Balance And Win Conditions | 10-minute match pacing and first win/loss rule. |
| GART | Art Return And Readability | Reintroduces visual assets only after abstract loop and scale gates pass. |

## Dependency Order

1. `PROTO`, `SRV`, `CLNT`, `TEST` before `GCORE`.
2. `RECON` and `LOSS` before gameplay is considered multiplayer-stable.
3. `SIM`, `INT`, `REND`, `PERF` before huge-unit claims.
4. `GPLAN` and `GLOAD` can run early as docs/contracts.
5. `GCORE`, `GCTRL`, `GCOV`, and `GECON` build the playable core.
6. `GROLE` and `GSWARM` expand tactical and scale pressure.
7. `GBAL` waits for playtest evidence.
8. `GART` waits for readability and performance evidence.

## Faction Scale Interpretation

"Huge amounts of units for all factions" is defined as a scenario family:

- Stage A: 2 player factions, neutral capture systems, zombie faction.
- Stage B: 2 player factions plus AI-controlled pressure groups.
- Stage C: 4+ simulated factions with mixed role composition.
- Stage D: 10,000+ total entities with AOI, LOD, aggregate far state, and measured bandwidth.

Each scenario must report:

- total entity count,
- entities per faction,
- visible entities per client,
- server tick time,
- client render frame time,
- snapshot size,
- bandwidth,
- memory,
- reconnect full snapshot time.

## ABC Assignment Pattern

- Alice turns gameplay into clear contracts, playtest wording, role matrices,
  runbooks, and Go/No-Go criteria.
- Bob implements authoritative gameplay systems on the Rust server and protocol:
  commands, HQ/spawn, combat, cover, economy, roles, swarm.
- Charlie owns harnesses, scale scenarios, render/perf evidence, integration
  checks, and release/test gates.

## First Safe Gameplay Slices

These can start before gameplay implementation:

| Slice | Why safe |
| --- | --- |
| GPLAN-01 | Docs-only conversion of gameplay pillars into scenario contracts. |
| GPLAN-02 | Read-only mapping from gameplay actions to protocol/server systems. |
| GLOAD-01 | Docs-only definition of faction scale matrix. |
| GLOAD-03 | Operator-facing scenario names and runbook language. |

Implementation should wait until the required infrastructure slices exist.

## Hard Rule

No gameplay slice is done unless it preserves:

- server authority,
- command idempotency,
- local multi-client testability,
- future Steam testability,
- performance measurement,
- abstract readability before final art,
- planned Godot folder/scene/node/subnode structure,
- scene-first/editor-first implementation using Godot MCP where available,
- structured logs and debug overlay coverage for the touched system,
- performance ledger updates for any changed simulation, networking, rendering,
  or UI cost.
