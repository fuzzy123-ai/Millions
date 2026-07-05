# Faction Scale Scenarios

Date: 2026-07-03
Slice: GLOAD-01
Status: provisional scenario matrix, not final faction-count design

## Purpose

This document defines safe faction-scale scenarios for future performance and
gameplay-validation work. The scenarios let backend, Godot, and evidence
slices reason about player factions, neutral systems, zombie pressure, and
later simulated factions without deciding the final all-factions target.

`G-FACTION-COUNT` still blocks the final faction count, final balance target,
and any release or marketing claim about total playable factions.

## Non-Goals

- No final faction count.
- No final unit roster, role balance, art, map, win condition, or economy rule.
- No gameplay implementation.
- No claim that 4+ simulated factions are playable factions.
- No scale claim without performance evidence and budget comparison.

## Faction Categories

| Category | Purpose | Authority owner | Notes |
| --- | --- | --- | --- |
| Player faction A | First human/player command source | Server owns durable state; Godot sends intent | Required for two-player loop |
| Player faction B | Second human/player command source | Server owns durable state; Godot sends intent | Required for local two-client smoke |
| Neutral systems | Capture points, resource nodes, map pressure | Server owns capture, ownership, income, and visibility | Requires MAPDATA before authority |
| Zombie pressure | Non-player route pressure and horde load | Server owns spawn, route pressure, AI/LOD state | Requires GSWARM and LOADSHED evidence |
| AI pressure groups | Optional non-player factions for stress and pacing | Server owns all state; local harness may synthesize input | Not final AI design |
| Simulated extra factions | 4+ faction stress entities | Server owns all state; no player UX claim | Used for load and AOI evidence only |

## Scenario Matrix

| Stage | Scenario ID | Composition | Initial load target | Primary validation | Blocking gate notes |
| --- | --- | --- | ---: | --- | --- |
| A | `gload_stage_a_2p_neutral_zombie_1k` | 2 player factions, neutral systems, zombie pressure | 1,000 entities | two-client command loop, neutral visibility, first horde pressure | Final gameplay scope and map authority remain gated |
| B | `gload_stage_b_ai_pressure_2k` | Stage A plus AI pressure groups | 2,000 entities | server ownership under extra non-player pressure | AI behavior is synthetic until gameplay slices define it |
| C | `gload_stage_c_4plus_mixed_5k` | 4+ simulated factions with mixed role composition | 5,000 entities | AOI, role distribution, snapshot size, render batching | Not a final playable faction-count claim |
| D | `gload_stage_d_10k_aoi_lod` | Stage C plus aggregate far-state and horde pressure | 10,000+ entities | AOI/LOD, bandwidth, server tick, render stress, backpressure | Must report budgets before any scale claim |

Load targets are starting points for later harnesses. A later slice may record a
failed or blocked result, but it may not claim the stage complete without the
required report fields and budget result.

Readable operator names and status language live in
`docs/runbooks/faction-scale-operator-notes.md`.

## Stage Dependencies

| Stage | Required infrastructure before implementation | Future gameplay roads that consume it |
| --- | --- | --- |
| A | PROTO, SRV, CLNT, TEST, RECON, DET, BUDGET, REND | GCORE, GCTRL, GECON, GSWARM |
| B | Stage A plus SIM, INT, LOADSHED basics | GSWARM, SOAK, GBAL evidence |
| C | Stage B plus role data contract and AOI reporting | GROLE, GLOAD-02, PHIST |
| D | Stage C plus aggregate far-state, backpressure, soak, perf history | GSWARM, NAV, LOADSHED, SOAK, release evidence |

## Safe Parallel Work

- Backend may build deterministic synthetic faction inputs and server-owned
  counters for these scenarios.
- Godot may render abstract faction IDs, proxy counts, and visible/aggregate
  state from authoritative snapshots.
- Steam preparation may keep player-session identity and lobby handoff separate
  from faction ownership.
- Evidence work may define report rows, redacted logs, and pass/fail semantics.

Stop if work requires final faction count, live Steam, real AppID, final art,
final balance, real AI design, or making Godot own faction truth.

## Done Rule

A faction-scale scenario is evidence-ready only when its report names scenario
ID, entity count, entities per faction, visible entities per client, faction
count, client count, server tick, snapshot size, bandwidth, Godot frame/render
cost, memory, replay status, backpressure events, and budget result.
