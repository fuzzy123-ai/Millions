# Gameplay Validation Scenarios

Date: 2026-07-03
Slice: GPLAN-01
Status: infrastructure contract, not gameplay implementation

## Purpose

Gameplay scenarios are the first validation targets for the Millions
foundation. They describe what the backend, Godot interface, protocol,
performance harnesses, and Steam preparation must be able to prove before
real gameplay mechanics are added.

This document does not define final balance, faction counts, unit stats, art,
maps, win conditions, or a playable match loop.

## Standing Go Policy

Repo-only and safe-offline work may continue without a fresh operator Go when
it stays inside these lanes:

- Rust backend scaffolding, protocol parsing, deterministic local tests, fake
  transports, local loopback networking, and offline harnesses.
- Godot interface work such as codecs, command queues, snapshot buffers,
  client-world state, render adapter stubs, headless checks, and local UI
  wiring needed for testability.
- Steam preparation such as facade contracts, mock/local lobby adapters,
  endpoint advertisement, Spacewar-safe placeholders, and dedicated-server
  handoff interfaces that do not call live Steam services.

Fresh operator or design Go is still required for live Steam auth, real AppID
assumptions, external playtest claims, final render technology, final gameplay
scope, final faction counts, final art return, and any live network mutation.

## Non-Goals

- No real gameplay mechanics in GPLAN-01.
- No server-authoritative combat, capture, economy, spawn, production, horde,
  movement, or win/loss implementation.
- No Godot scene work beyond contracts required by later slices.
- No live Steam calls, real tickets, real AppID assumptions, or private account
  data.
- No balance, unit stat, map-content, art-style, or final faction decision.

## Parallel Work Rules

Agents may work in parallel only when their write scopes are disjoint and their
slice class is `repo_only` or `safe_offline`.

Backend slices may proceed on server-owned state, command validation, replay,
performance, hardening, and local loopback evidence. Godot slices may proceed
on intent construction, snapshot application, render-state adapters, local
debug surfaces, and headless checks. Steam slices may proceed on mock/local
identity, lobby metadata, endpoint advertisement, and server handoff contracts.

No agent may make Godot authoritative for durable match state, encode gameplay
truth in scene or resource paths, persist secrets, or mark a gameplay slice done
without tests or evidence tied to the touched system.

## Scenario Matrix

| Scenario | Infrastructure it validates | Required authority boundary | Evidence before gameplay claim |
| --- | --- | --- | --- |
| Two-player command loop | Protocol intents, server session ownership, local multi-client harness, Godot command queue | Client sends intent; server decides admission and state changes | Rust command tests, Godot intent/adapter check, local two-client smoke, replay-ready command log |
| Cover combat readability | Map-data export, LOS/LOF inputs, render feedback, snapshot correction | Server owns cover, obstacle, hit, miss, blocked, damage, and occupancy outcomes | Versioned map-data checksum, server cover tests, render readability smoke, perf report |
| Capture economy pressure | Capture state, income ticks, production spend, event ordering, reconnect restore | Server owns credits, income, capture, production, spawn, and win/loss outcomes | Economy tests, reconnect full-snapshot check, deterministic replay status, bandwidth report |
| Mixed-role squads | Role data plumbing, AOI filtering, render batching, role readability | Server owns role truth and combat effects; Godot renders current authoritative role state | Role fixture tests, Godot snapshot/render checks, mixed-role perf scenario |
| Zombie horde pressure | Large entity simulation, spawn pacing, AI LOD, load shedding, aggregate far-state replication | Server owns swarm timer, spawn, route pressure, aggro, and LOD state | Swarm tests, 1k+ load report, backpressure counters, AOI/bandwidth report |
| Large-army movement | Movement model, pathfinding/formation cost, client interpolation, selection readability | Server owns final movement state; client may preview and reconcile | Movement perf scenario, deterministic replay checksum, render stress evidence |

## Gate Boundaries

- `G-GAMEPLAY-SCOPE` blocks real gameplay implementation until the relevant
  infrastructure contracts and evidence exist.
- `G-GODOT-SCENE-CONTRACT` blocks substantial Godot feature work outside the
  scene-first contract.
- `G-GODOT-BRIDGE` blocks the final bridge choice, but not local codecs,
  queues, buffers, facades, or adapter checks.
- `G-STEAM-AUTH` and `G-REAL-APPID` block live Steam auth and real release
  identity. Mock/local Steam preparation remains allowed.
- `G-MAPDATA-AUTHORITY` blocks authoritative cover, obstacle, spawn, capture,
  and navigation gameplay until versioned checksum map data exists.
- `G-MOVEMENT-SCALE` blocks movement-heavy gameplay claims until movement
  scale evidence exists.
  Player-facing movement feedback preparation lives in
  `docs/gameplay/movement-feedback-contract.md` and remains presentation-only
  until server movement evidence exists.
- `G-PERF-BUDGETS`, `G-PERF-HISTORY`, `G-DETERMINISTIC-REPLAY`,
  `G-BACKPRESSURE`, `G-SOAK-STABILITY`, `G-PROTOCOL-HARDENING`, and
  `G-REPRODUCIBLE-BUILDS` block scale, stability, external playtest, and
  release-candidate claims until their evidence is present.

## Recommended Execution Order

1. Continue backend, Godot interface, and Steam preparation only through
   `repo_only` and `safe_offline` slices.
2. Prefer infrastructure and evidence roadmaps before gameplay mechanics:
   PROTO, SRV, CLNT, TEST, RECON, LOSS, SIM, INT, REND, PERF, BUDGET, DET,
   SOAK, LOADSHED, MAPDATA, NAV, HARDEN, BUILD.
3. Use GCORE and later gameplay roadmaps only after their dependency and gate
   evidence is present.
4. Keep live Steam, release-candidate, final art, final faction count, and
   final balance work deferred until the matching explicit Go exists.

## Done Rules For Future Gameplay Slices

A future gameplay slice is not done until it has:

- server authority preserved for durable match state,
- protocol fixtures or command tests for changed wire behavior,
- Godot checks when client adapter, render, or scene behavior changes,
- deterministic replay or desync evidence when simulation state changes,
- performance evidence for any scale-sensitive path,
- redacted logs and no secrets or private provider data,
- documentation or handoff evidence updated in the relevant repo artifact.
