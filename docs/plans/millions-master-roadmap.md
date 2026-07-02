# Millions Master Roadmap

Date: 2026-07-02
Source handoff: `E:/TankGame/docs/server_client_infrastructure_handoff.md`
Execution mode: Standard ABC

## Goal

Create a real client/server RTS foundation where Godot handles rendering,
input, UI, Steam lobby flow, and client prediction feedback, while a dedicated
Rust server owns authoritative simulation for very large entity counts.

## Current Evidence

- Handoff exists at `E:/TankGame/docs/server_client_infrastructure_handoff.md`.
- `E:/Millions` exists as an empty project directory.
- `C:/Users/nkatz/Documents/Millions` is an empty Git repository.
- No production server, protocol, client adapter, harness, or performance gates
  exist yet.

## Non-Goals

- Do not copy TankGame gameplay, maps, assets, terrain, combat, VFX, or debug
  scenes into Millions.
- Do not build real gameplay before the server/client architecture spike.
- Do not make Steam AppID assumptions beyond the existing configurable fallback.
- Do not create rich per-unit Godot scene trees for scale testing.
- Do not treat gameplay handoff systems as inherited code. They are test
  scenarios and behavioral targets to rebuild against the new infrastructure.
- Do not build substantial Godot features without a planned folder, scene,
  node, subnode, Resource, and ownership structure.
- Do not replace editor-visible scene structure with script-created nodes
  unless the node is genuinely dynamic runtime state.

## Master Phases

| Phase | Kürzel | Name | Primary roadmaps | Done state |
| --- | --- | --- | --- | --- |
| 0 | FND | Foundation And Decisions | RGAP, ARCH, PROTO | Architecture defaults, gates, and protocol draft are explicit. |
| 1 | GFOUND | Godot Client Foundation | GSCENE, GNET, GOBS | Godot folder, scene, node, bridge, and logging contracts are explicit. |
| 2 | LOOP | Minimal Network Loop | SRV, CLNT, LOBBY | Two local clients can connect to one server and render authoritative snapshots. |
| 3 | RES | Resilience | RECON, LOSS, TEST | Command ack/resend, packet loss recovery, reconnect, and local harness work. |
| 4 | GPLAN | Gameplay-As-Test Planning | GPLAN, GLOAD | Gameplay handoff becomes scenario contracts and faction-scale load profiles. |
| 5 | SCALE | Scale And Performance | SIM, INT, REND, PERF, PHIST | 1k/5k/10k stress gates exist, are measured, and have historical trend evidence. |
| 6 | GLOOP | Playtest Gameplay Loop | GCORE, GCTRL, GCOV, GECON | Abstract 2-player RTS loop validates controls, cover, capture economy, and authority. |
| 7 | HORDE | Swarm And Role Expansion | GROLE, GSWARM, GBAL | Unit-role mix and zombie pressure validate large multi-faction simulation. |
| 8 | FLOW | Production Match Flow | STEAM, OPS, REL, GART | Steam lobby to dedicated-server flow, runbooks, release evidence, RC gates, and art-return gates exist. |

## Roadmap Suite

| # | Kürzel | Roadmap | Main owner | Support | Status |
| --- | --- | --- | --- | --- | --- |
| 01 | RGAP | Risk And Gap Closure | Alice | Bob, Charlie | Planned |
| 02 | ARCH | Architecture Contract | Alice | Bob | Planned |
| 03 | PROTO | Binary Protocol V0 | Bob | Alice, Charlie | Planned |
| 04 | SRV | Rust Simulation Server Spike | Bob | Charlie | Planned |
| 05 | CLNT | Godot Client Network Adapter | Bob | Alice, Charlie | Planned |
| 06 | LOBBY | Steam Lobby Preservation | Alice | Bob | Planned |
| 07 | RECON | Reconnect And Session Recovery | Bob | Charlie | Planned |
| 08 | LOSS | Packet Loss And Command Reliability | Bob | Charlie | Planned |
| 09 | TEST | Local Multi-Client Harness | Charlie | Alice, Bob | Planned |
| 10 | SIM | Data-Oriented Simulation Scale | Bob | Charlie | Planned |
| 11 | INT | Interest Management And Visibility | Bob | Charlie | Planned |
| 12 | REND | Godot Large-Entity Rendering | Charlie | Bob | Planned |
| 13 | PERF | Performance Gates And Telemetry | Charlie | Bob | Planned |
| 14 | STEAM | Steam Dedicated-Server Bridge | Alice | Bob, Charlie | Planned |
| 15 | OPS | Runbooks, Launchers, And Evidence | Alice | Charlie | Planned |
| 16 | REL | Release Candidate Gates | Charlie | Alice, Bob | Planned |
| 17 | GPLAN | Gameplay Scenario Contracts | Alice | Bob, Charlie | Planned |
| 18 | GLOAD | Faction Scale Load Matrix | Charlie | Bob, Alice | Planned |
| 19 | GCORE | Two-Player Playtest Core | Bob | Alice, Charlie | Planned |
| 20 | GCTRL | RTS Controls And Readability | Alice | Bob, Charlie | Planned |
| 21 | GCOV | Cover Combat Authority | Bob | Alice, Charlie | Planned |
| 22 | GECON | Capture Economy Match Loop | Bob | Alice, Charlie | Planned |
| 23 | GROLE | Unit Role Expansion | Alice | Bob, Charlie | Planned |
| 24 | GSWARM | Zombie Horde Scale Event | Bob | Charlie, Alice | Planned |
| 25 | GBAL | Match Balance And Win Conditions | Alice | Bob, Charlie | Planned |
| 26 | GART | Art Return And Readability | Alice | Charlie | Planned |
| 27 | GNET | Godot Server Bridge | Bob | Charlie, Alice | Planned |
| 28 | GSCENE | Godot Scene And Node Architecture | Alice | Charlie, Bob | Planned |
| 29 | GOBS | Logging Debug And Observability | Charlie | Bob, Alice | Planned |
| 30 | PHIST | Performance History Ledger | Charlie | Bob, Alice | Planned |

## Stop Rules

- Stop on secrets, tokens, private content, or raw provider/session data being
  persisted.
- Stop on destructive Git needs, force-push, reset, or unrelated staged files.
- Stop on live Steam/provider/server mutation without explicit Go.
- Stop on hot-file conflicts between agents.
- Stop if a slice would copy prototype gameplay/assets into Millions.
- Stop if architecture changes break either local multi-client mode or Steam
  lobby mode without an explicit gate.

## ABC Coordination

- Alice owns documentation, operator wording, runbooks, lobby UX contracts, and
  gate language.
- Bob owns protocol, Rust server, validation, simulation, transport, and tests.
- Charlie owns integration, harnesses, performance gates, repo hygiene, and
  release readiness.

Workers must have disjoint write scopes. Explorers may read and report only.
Implementation should be delegated by roadmap slice, not by vague topic.

## Master Verification Gates

| Gate | Requirement |
| --- | --- |
| MV-G1 | Protocol fixtures encode/decode on Rust and Godot sides. |
| MV-G2 | Two local Godot clients connect to one local dedicated server. |
| MV-G3 | Server simulates at least 1,000 abstract entities. |
| MV-G4 | Clients render only authoritative snapshot state. |
| MV-G5 | Reconnect restores the same player session with a full snapshot. |
| MV-G6 | Packet loss does not cause double-spend, double-spawn, or stale state authority. |
| MV-G7 | 1k, 5k, and 10k performance harnesses produce tick/render/network metrics. |
| MV-G8 | Steam lobby ID copy/join/ready flow remains testable. |
| MV-G9 | Same-workstation multi-client smoke passes for each architecture change. |
| MV-G10 | Real two-machine Steam smoke is required for release candidates. |
| MV-G11 | Two-player abstract RTS loop supports select/move/attack/take-cover, HQ spawn, credits, capture points, and late swarm trigger. |
| MV-G12 | Faction-scale scenarios measure per-faction unit counts, zombie pressure, AOI bandwidth, server tick, and Godot render budget. |
| MV-G13 | Gameplay additions cannot bypass server authority or command idempotency. |
| MV-G14 | Every substantial Godot feature has a documented folder, scene, node/subnode tree, Resource contract, and dynamic-node exception list before implementation. |
| MV-G15 | Godot server bridge exposes logs/counters for connect, handshake, command seq/ack, snapshot decode/apply, interpolation, AOI, reconnect, and packet loss. |
| MV-G16 | Performance ledger records trend rows for server tick, snapshot size, bandwidth, Godot decode/apply/render, memory, visible entities, and reconnect full snapshot time. |

## Go Language

- Go: scope, gates, and tests are green for the roadmap slice.
- Partial: safe preparation is complete, but live/design evidence is still gated.
- Deferred: roadmap remains valid, but an explicit gate blocks further work.
- No-Go: work would violate authority, scale, Steam, safety, or repo rules.
- Blocked: no safe slice remains until user/live/design input changes.
