# Gameplay Infrastructure Dependency Matrix

Date: 2026-07-03
Slice: GPLAN-03
Status: dependency matrix for future gameplay slices

## Purpose

This matrix connects the gameplay validation scenarios from
`docs/gameplay/gameplay-validation-scenarios.md` to the infrastructure roadmaps
and gates that must be satisfied before a future agent implements real gameplay
behavior.

The matrix is an execution guide. It is not a design approval, gameplay
implementation, balance choice, art direction, or live Steam approval.

## Global Order

Future agents should preserve this order unless the JSON plan is changed:

1. Foundation and contracts: PROTO, SRV, CLNT, TEST.
2. Session resilience: RECON, LOSS.
3. Scale and evidence: SIM, INT, REND, PERF, BUDGET, DET, SOAK, LOADSHED.
4. Authoritative content inputs: MAPDATA.
5. Movement scale: NAV.
6. Hardening and reproducibility: HARDEN, BUILD.
7. Initial gameplay validation: GCORE, GCTRL, GCOV, GECON.
8. Role and horde expansion: GROLE, GSWARM.
9. Balance and final win/loss choice: GBAL.
10. Art return and final readability: GART.

## Parallel Lane Rules

| Lane | May proceed without fresh Go | Must stop for |
| --- | --- | --- |
| Backend | server-owned command validation, idempotency, replay, deterministic sim scaffolds, fake/local transport, perf harnesses | final ECS design, real gameplay scope, live external playtest, destructive data changes |
| Godot interface | command queues, codecs, snapshot buffers, client-world mirrors, render adapters, local/headless checks, debug surfaces | Godot owning durable state, final render technology, broad UI/design choices |
| Steam preparation | mock/local lobby facade, endpoint advertisement, dedicated-server handoff, Spacewar-safe placeholders | real AppID, live Steam auth tickets, live two-machine release-candidate claim |
| Evidence | local smoke scripts, redacted logs, fixture checks, perf report schemas, release/evidence indexes | scale or RC claims without measured artifacts |

Parallel work is safe only when file scopes are disjoint and the selected slice
is `repo_only` or `safe_offline`.

## Scenario To Roadmap Matrix

| Scenario | Primary roadmaps | Blocking gates | Minimum evidence before gameplay claim |
| --- | --- | --- | --- |
| Two-player command loop | PROTO, SRV, CLNT, TEST, RECON, DET, GCORE, GCTRL | G-GAMEPLAY-SCOPE, G-DETERMINISTIC-REPLAY | command fixture/ack coverage, server idempotency tests, Godot command queue check, command feedback state contract, local two-client smoke, replay record/checksum |
| Cover combat readability | MAPDATA, BUDGET, GCOV, REND, GCTRL | G-MAPDATA-AUTHORITY, G-PERF-BUDGETS, G-RENDER-TECH | versioned map-data checksum, cover/LOS/LOF server tests, render readability check, perf report with entity visibility |
| Capture economy pressure | MAPDATA, GECON, RECON, DET, PERF | G-MAPDATA-AUTHORITY, G-GAMEPLAY-SCOPE, G-DETERMINISTIC-REPLAY | capture/economy server tests, production intent fixture, reconnect full snapshot check, deterministic replay evidence, bandwidth report |
| Mixed-role squads | GROLE, INT, REND, BUDGET, TEST | G-GAMEPLAY-SCOPE, G-PERF-BUDGETS | role data contract, AOI-visible role snapshot check, Godot render batching check, mixed-role perf scenario |
| Zombie horde pressure | GSWARM, SIM, INT, LOADSHED, SOAK, BUDGET | G-BACKPRESSURE, G-SOAK-STABILITY, G-PERF-BUDGETS | swarm spawn/pressure tests, aggro/AI LOD contract, collision-prep boundary, 1k+ local load report, AOI/bandwidth report, backpressure counters, soak/long-run result |
| Large-army movement | NAV, SIM, MAPDATA, DET, REND, PERF | G-MOVEMENT-SCALE, G-MAPDATA-AUTHORITY, G-DETERMINISTIC-REPLAY | movement model option selected, path/movement perf report, replay checksum, Godot render stress/readability check |

## Action To Authority Matrix

| Player action | Protocol shape today | Authoritative system later | Godot responsibility | Required later slice |
| --- | --- | --- | --- | --- |
| Ready | `command_type` 1, empty or bounded lobby payload | session admission/readiness on server | local button state, pending/ack display | GCORE or lobby follow-up when match admission expands |
| Select group | `command_type` 2, selection intent only | optional server validation if selection affects command legality | local selection presentation and command context | GCTRL before UI feedback claims |
| Move | `command_type` 3, target position intent | movement validation, pathing, collision, map constraints | preview, pending state, interpolation, reconciliation | NAV and GCORE movement baseline |
| Attack | `command_type` 4, target entity or position intent | targeting, damage, cooldowns, cover effects | cursor/context feedback, pending/rejected state, render hints | GCOV after GCORE |
| Take cover | `command_type` 5, cover intent | cover occupancy, LOS/LOF, map-data authority | highlight candidate cover as non-authoritative preview | MAPDATA then GCOV |
| Spawn or produce | `command_type` 6, production/spawn request intent | credits, production queues, spawn validation, entity creation | queue request UI and ack/reject state | GECON and GCORE spawn baseline |
| Reconnect resume | `command_type` 7 plus reconnect handshake/session data | session rebind, stale command handling, full snapshot restore | reconnect state display and snapshot reconciliation | RECON/LOSS already provide foundation; gameplay slices must preserve it |

`docs/gameplay/swarm-behavior-contract.md` owns the current GSWARM-02 behavior
boundary for aggro trails, AI LOD, and collision preparation.

## Gate Queue For Agents

Gate: `G-GAMEPLAY-SCOPE`
Class: `needs_design`
Blocks: real gameplay mechanics and final gameplay scope
Safe preparation done: gameplay validation scenarios, action mapping, and this
dependency matrix
Risk if bypassed: gameplay code can outrun protocol, authority, determinism,
performance, and Godot scene contracts
Next safe slice: use the next `repo_only` or `safe_offline` infrastructure
slice from `docs/plans/millions-plan.json`

Gate: `G-STEAM-AUTH` / `G-REAL-APPID`
Class: `needs_live_go`
Blocks: live Steam auth, real AppID, real ticket validation, and release
identity claims
Safe preparation done: mock/local Steam bridge, endpoint advertisement, and
server handoff contracts
Risk if bypassed: secrets or real provider state could be persisted or used in
unverified flows
Next safe slice: Steam facade, mock identity, or local dedicated-server
handoff work only

Gate: `G-MAPDATA-AUTHORITY`
Class: `repo_only`
Blocks: authoritative cover, obstacle, spawn, capture, and navigation gameplay
Safe preparation done: scenario and dependency requirements
Risk if bypassed: Godot scene placeholders could become hidden gameplay truth
Next safe slice: MAPDATA-01

Gate: `G-MOVEMENT-SCALE`
Class: `needs_design`
Blocks: complex movement, pathfinding, formations, and movement-heavy gameplay
claims
Safe preparation done: movement scenario requirements and the NAV-01 option
matrix in `docs/architecture/movement-model-options.md`
Risk if bypassed: movement cost can dominate scale without evidence
Next safe slice: NAV-02

## Completion Rule

A future gameplay slice may be marked done only when its implementation,
focused tests, documentation, and evidence satisfy every row it touches in this
matrix. If a row references an unopened gate, the slice must stay blocked or
deferred and the next safe infrastructure slice should be selected.

Any slice that plausibly changes runtime cost must also run the closest
available performance/readability smoke and preserve or strengthen the related
`tests/perf/` report. This includes entity volume, simulation work, rendering
work, AI, movement, collision, combat, selection density, snapshots, or
bandwidth. Docs-only, contract-only, planning-only, and small non-runtime
refactors may skip performance smokes when the handoff records why runtime cost
cannot change. Functional correctness alone is not enough for done status on
scale-sensitive gameplay work.
