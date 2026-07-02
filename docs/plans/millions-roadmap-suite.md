# Millions Roadmap Suite

Date: 2026-07-02
Mode: Standard ABC

Each roadmap is execution-ready for future Alice/Bob/Charlie delegation. Classes:
`safe_offline`, `repo_only`, `needs_live_go`, `needs_design`, `blocked`.

## RM01 RGAP - Risk And Gap Closure

Goal: Convert major unknowns into explicit defaults, gates, and measurable spike criteria.
Current evidence: Handoff and risk register exist; no code yet.
Non-goals: no gameplay, no live Steam mutation, no crate lock-in beyond spike.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| RGAP-01 | repo_only | Alice worker | Maintain risk register and gate queue. | `docs/plans/*risk*`, `docs/plans/*roadmap*` | Docs-only |
| RGAP-02 | safe_offline | Bob explorer | Compare transport/ECS options against Godot integration needs. | read-only | No tests |
| RGAP-03 | repo_only | Charlie worker | Add decision log template and gate status checklist. | `docs/plans/` | Docs-only |

Gate queue: G-TRANSPORT-QUIC, G-ECS-CHOICE, G-REAL-APPID.
Verification: risk register has owner, decision, gate, and next safe slice for each major gap.

## RM02 ARCH - Architecture Contract

Goal: Freeze the first architecture contract between Godot client, Rust server, Steam facade, and test harness.
Current evidence: target architecture is described in the handoff.
Non-goals: no implementation beyond interface stubs.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| ARCH-01 | repo_only | Alice worker | Document authority boundaries and non-goals. | `docs/architecture/` | Docs-only |
| ARCH-02 | repo_only | Bob worker | Define module boundaries for server, protocol, transport, simulation, and fixtures. | `server/`, `docs/architecture/` | TBD after scaffold |
| ARCH-03 | repo_only | Charlie worker | Define integration checkpoints and stop rules. | `docs/architecture/`, `docs/plans/` | Docs-only |

Gate queue: none.
Verification: every boundary has one owner, one adapter, and one forbidden coupling list.

## RM03 PROTO - Binary Protocol V0

Goal: Define and test a compact versioned binary protocol for commands, events, snapshots, and reconnect.
Current evidence: protocol shape exists conceptually only.
Non-goals: no text-heavy live packets, no Godot Nodes/Resources/scene paths in network messages.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| PROTO-01 | repo_only | Bob worker | Specify message header, ids, ticks, seq/ack, quantization, and entity deltas. | `docs/protocol/` | Docs-only |
| PROTO-02 | repo_only | Bob worker | Add shared packet fixture files. | `protocol/fixtures/`, `server/` | Rust fixture tests |
| PROTO-03 | repo_only | Charlie worker | Add cross-side fixture validation plan for Godot and Rust. | `docs/protocol/`, `tests/` | Fixture checks |

Gate queue: none.
Verification: Rust and Godot can prove identical fixture interpretation before live networking expands.

## RM04 SRV - Rust Simulation Server Spike

Goal: Build a minimal authoritative Rust server that simulates abstract entities and emits snapshots.
Current evidence: no server exists.
Non-goals: no final gameplay, no production ECS commitment before evidence.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| SRV-01 | repo_only | Bob worker | Scaffold Rust workspace and tick loop. | `server/`, `Cargo.toml` | `cargo test` |
| SRV-02 | repo_only | Bob worker | Implement abstract entity state, IDs, movement stub, and snapshot builder. | `server/` | `cargo test` |
| SRV-03 | repo_only | Charlie worker | Add server smoke command and log format. | `server/`, `scripts/`, `docs/runbooks/` | server smoke |

Gate queue: G-ECS-CHOICE.
Verification: server runs headless, ticks deterministically, and can simulate 1,000 abstract entities.

## RM05 CLNT - Godot Client Network Adapter

Goal: Give Godot one adapter for outgoing commands and incoming snapshots.
Current evidence: TankGame has Steam facade and cursor sync lessons; Millions has no client yet.
Non-goals: no gameplay authority in Godot.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| CLNT-01 | repo_only | Alice worker | Document Godot client adapter contract and UI ownership rules. | `docs/architecture/`, `client/` | Docs-only |
| CLNT-02 | repo_only | Bob worker | Implement command/snapshot data structs and adapter stub. | `client/godot/` | Godot check-only |
| CLNT-03 | repo_only | Charlie worker | Add snapshot-render smoke scene with simple proxies. | `client/godot/`, `docs/runbooks/` | Godot headless smoke |

Gate queue: G-RENDER-TECH.
Verification: client can apply server snapshot data without making authoritative decisions.

## RM06 LOBBY - Steam Lobby Preservation

Goal: Preserve Host/Join/Copy/Ready concepts while preparing dedicated-server match handoff.
Current evidence: TankGame handoff identifies existing lobby shell and `Online.gd` facade.
Non-goals: do not call GodotSteam directly from gameplay/client code.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| LOBBY-01 | repo_only | Alice worker | Document lobby states, copy ID UX, ready semantics, and AppID policy. | `docs/steam/`, `docs/runbooks/` | Docs-only |
| LOBBY-02 | repo_only | Bob worker | Define Steam facade to dedicated-server session handshake interface. | `client/godot/`, `docs/steam/` | Interface tests/checks |
| LOBBY-03 | needs_live_go | Charlie explorer | Run real Steam two-machine smoke when AppID/test machines are available. | read-only/live | Manual smoke |

Gate queue: G-STEAM-AUTH, G-STEAM-TWO-MACHINE, G-REAL-APPID.
Verification: Steam lobby can remain testable while local direct server mode also works.

## RM07 RECON - Reconnect And Session Recovery

Goal: Reconnect returns a player to the same session with full snapshot recovery and resumed deltas.
Current evidence: reconnect is specified in the handoff but not implemented.
Non-goals: no client state overriding server truth.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| RECON-01 | repo_only | Bob worker | Implement `player_session_id`, grace period, and connection rebind model. | `server/`, `protocol/` | Rust tests |
| RECON-02 | repo_only | Bob worker | Add full snapshot on reconnect and resume delta stream. | `server/`, `client/godot/` | reconnect test |
| RECON-03 | repo_only | Charlie worker | Add restart-client harness case. | `scripts/`, `tests/`, `docs/runbooks/` | harness reconnect |

Gate queue: G-STEAM-AUTH for real Steam auth binding.
Verification: local client restart resumes the same session with no double-spawn or lost ownership.

## RM08 LOSS - Packet Loss And Command Reliability

Goal: Commands survive packet loss without duplicate effects; snapshots recover by supersession.
Current evidence: rules exist in handoff only.
Non-goals: no per-frame reliable replication.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| LOSS-01 | repo_only | Bob worker | Implement command sequence, ack, resend, and idempotency checks. | `server/`, `protocol/`, `client/godot/` | reliability tests |
| LOSS-02 | repo_only | Charlie worker | Add packet loss/jitter simulation harness. | `scripts/`, `tests/` | loss harness |
| LOSS-03 | safe_offline | Alice explorer | Review operator-visible failure states and wording. | read-only | No tests |

Gate queue: none.
Verification: duplicate commands cannot double-spend, double-spawn, or double-trigger.

## RM09 TEST - Local Multi-Client Harness

Goal: One command launches one local server and multiple distinguishable clients.
Current evidence: same-workstation multi-client is a non-negotiable gate.
Non-goals: local mode must not replace Steam mode.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| TEST-01 | repo_only | Charlie worker | Add launcher config for server plus N clients with distinct profiles. | `scripts/`, `config/`, `docs/runbooks/` | launcher smoke |
| TEST-02 | repo_only | Bob worker | Add local synthetic identity handshake. | `server/`, `client/godot/`, `protocol/` | handshake tests |
| TEST-03 | repo_only | Alice worker | Write runbook for local, Steam, reconnect, and packet loss tests. | `docs/runbooks/` | Docs-only |

Gate queue: none.
Verification: logs include client index, session ID, connection ID, and server tick where practical.

## RM10 SIM - Data-Oriented Simulation Scale

Goal: Server simulation reaches 1k, 5k, and 10k abstract entities with measured tick budgets.
Current evidence: performance requirement exists; no harness exists.
Non-goals: no expensive per-entity object logic or all-vs-all targeting.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| SIM-01 | repo_only | Bob worker | Build SoA-friendly simulation loop and spatial grid baseline. | `server/` | Rust bench/test |
| SIM-02 | repo_only | Bob worker | Add 1k/5k/10k simulation-only scenarios. | `server/`, `tests/perf/` | perf smoke |
| SIM-03 | repo_only | Charlie worker | Record tick time, memory, and regression thresholds. | `tests/perf/`, `docs/perf/` | perf report |

Gate queue: G-ECS-CHOICE.
Verification: simulation performance data exists before gameplay complexity is added.

## RM11 INT - Interest Management And Visibility

Goal: Clients receive only relevant entity state based on AOI/visibility.
Current evidence: interest management is required but undefined.
Non-goals: no always-send-all snapshots outside isolated tests.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| INT-01 | repo_only | Bob worker | Define AOI regions and per-client subscription state. | `server/`, `docs/protocol/` | Rust tests |
| INT-02 | repo_only | Bob worker | Encode visible entity deltas and aggregate far state. | `server/`, `protocol/` | snapshot tests |
| INT-03 | repo_only | Charlie worker | Add bandwidth metrics and regression gates. | `tests/perf/`, `docs/perf/` | bandwidth smoke |

Gate queue: none.
Verification: 10k world does not imply 10k entity updates to every client.

## RM12 REND - Godot Large-Entity Rendering

Goal: Godot renders large snapshot-driven entity sets using lightweight proxies and measured budgets.
Current evidence: handoff warns against rich scene trees for 10k entities.
Non-goals: no final art, no real unit assets, no per-entity heavy child nodes.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| REND-01 | repo_only | Charlie worker | Create render-only stress scene with simple proxies. | `client/godot/`, `tests/perf/` | Godot render smoke |
| REND-02 | repo_only | Bob worker | Bind snapshot state to render batches without authority. | `client/godot/` | Godot check-only |
| REND-03 | needs_design | Alice explorer | Compare MultiMesh, RenderingServer, sprite batch, and hybrid evidence. | read-only | No tests |

Gate queue: G-RENDER-TECH.
Verification: render-only visible entity stress test reports frame time and memory.

## RM13 PERF - Performance Gates And Telemetry

Goal: Performance gates become routine checks for server tick, render frame, bandwidth, memory, and reconnect time.
Current evidence: performance testing is mandatory but not implemented.
Non-goals: no manual-only performance feelings.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| PERF-01 | repo_only | Charlie worker | Define metrics schema and perf report format. | `docs/perf/`, `tests/perf/` | Docs-only/schema |
| PERF-02 | repo_only | Bob worker | Emit server tick, snapshot size, and bandwidth metrics. | `server/` | metrics tests |
| PERF-03 | repo_only | Charlie worker | Add CI/local performance smoke commands. | `scripts/`, `tests/perf/` | perf smoke |

Gate queue: none.
Verification: every meaningful simulation/render/network change has a focused performance smoke path.

## RM14 STEAM - Steam Dedicated-Server Bridge

Goal: Map Steam lobby identity and ready flow to a dedicated-server match/session handshake.
Current evidence: Steam role is recommended in the handoff; implementation is absent.
Non-goals: no hardcoded AppID, no direct GodotSteam calls outside facade.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| STEAM-01 | repo_only | Alice worker | Document Steam lobby metadata, endpoint advertisement, and ready-to-match sequence. | `docs/steam/` | Docs-only |
| STEAM-02 | repo_only | Bob worker | Implement facade-shaped local handshake mock. | `client/godot/`, `server/`, `protocol/` | handshake tests |
| STEAM-03 | needs_live_go | Charlie worker | Validate Steam ticket/session flow with real environment. | live Steam only | Steam smoke |

Gate queue: G-STEAM-AUTH, G-REAL-APPID, G-STEAM-TWO-MACHINE.
Verification: dedicated server has stable player sessions independent of transient socket IDs.

## RM15 OPS - Runbooks, Launchers, And Evidence

Goal: Keep simple runbooks and launchers for local, Steam, reconnect, packet loss, and performance tests.
Current evidence: TankGame has launcher paths; Millions has none yet.
Non-goals: no deployment automation before architecture spike evidence.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| OPS-01 | repo_only | Alice worker | Write operator runbooks with Go/No-Go language. | `docs/runbooks/` | Docs-only |
| OPS-02 | repo_only | Charlie worker | Add scripts for local server/client smoke commands. | `scripts/`, `config/` | script smoke |
| OPS-03 | repo_only | Charlie worker | Maintain evidence index for tests and RC gates. | `docs/evidence/` | Docs-only |

Gate queue: none.
Verification: a new agent can run the key local checks from docs without chat history.

## RM16 REL - Release Candidate Gates

Goal: Define release-candidate evidence before any public or playtest build is treated as ready.
Current evidence: release gates are implicit in handoff expectations.
Non-goals: no release until real Steam smoke and performance gates pass.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| REL-01 | repo_only | Alice worker | Define RC checklist and operator signoff language. | `docs/release/` | Docs-only |
| REL-02 | repo_only | Charlie worker | Connect RC checklist to evidence artifacts. | `docs/release/`, `docs/evidence/` | Docs-only |
| REL-03 | needs_live_go | Charlie worker | Run real Steam two-machine RC smoke. | live Steam only | Manual RC smoke |

Gate queue: G-STEAM-TWO-MACHINE, G-REAL-APPID.
Verification: RC cannot be marked ready without local multi-client, packet loss/reconnect, scale perf, and real Steam evidence.

## RM17 GPLAN - Gameplay Scenario Contracts

Goal: Convert the gameplay handoff into server-authoritative scenario contracts without copying prototype implementation.
Current evidence: gameplay handoff describes playable loop, controls, cover, economy, roles, and swarm target.
Non-goals: no final assets, no inherited gameplay code, no tech trees, no deep basebuilding.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GPLAN-01 | repo_only | Alice worker | Document gameplay pillars as infrastructure validation scenarios. | `docs/gameplay/`, `docs/plans/` | Docs-only |
| GPLAN-02 | safe_offline | Bob explorer | Map gameplay actions to protocol commands and authoritative server systems. | read-only | No tests |
| GPLAN-03 | repo_only | Charlie worker | Add dependency matrix from gameplay scenarios to infra roadmaps/gates. | `docs/gameplay/`, `docs/plans/` | Docs-only |

Gate queue: G-GAMEPLAY-SCOPE.
Verification: each gameplay scenario names its required protocol, server, client, test, and perf dependencies.

## RM18 GLOAD - Faction Scale Load Matrix

Goal: Define "huge units for all factions" as measurable scale scenarios.
Current evidence: user target is RTS with huge unit counts for every faction; infrastructure handoff targets 10,000+ entities.
Non-goals: no final faction lore, art, or full balance before scale evidence.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GLOAD-01 | repo_only | Charlie worker | Define scenario matrix for 2 player factions, neutral systems, zombie faction, then 4+ simulated factions. | `docs/gameplay/`, `docs/perf/` | Docs-only |
| GLOAD-02 | repo_only | Bob worker | Translate role mixes into server perf scenarios: infantry, vehicles, static defense, swarm. | `server/`, `tests/perf/`, `docs/perf/` | perf smoke |
| GLOAD-03 | repo_only | Alice worker | Define readable naming and operator descriptions for faction scenarios. | `docs/gameplay/`, `docs/runbooks/` | Docs-only |

Gate queue: G-FACTION-COUNT.
Verification: performance results can be reported per faction, per role mix, and per client AOI.

## RM19 GCORE - Two-Player Playtest Core

Goal: Build the first abstract 2-player match loop on top of the dedicated-server infrastructure.
Current evidence: TankGame prototype had lobby, HQ spawn, selection, movement, attack, credits, capture, and swarm timer.
Non-goals: no real art, no final combat balance, no client authority.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GCORE-01 | repo_only | Bob worker | Implement authoritative HQ, player start, basic squad spawn, and move command model. | `server/`, `protocol/`, `client/godot/` | gameplay command tests |
| GCORE-02 | repo_only | Charlie worker | Add local two-client match smoke using abstract proxies. | `scripts/`, `tests/`, `client/godot/` | local match smoke |
| GCORE-03 | repo_only | Alice worker | Write playtest loop runbook and Go/No-Go checklist. | `docs/runbooks/`, `docs/gameplay/` | Docs-only |

Gate queue: requires PROTO, SRV, CLNT, TEST basics.
Verification: two local clients can select own squads, issue move/attack/take-cover intents, and observe authoritative results.

## RM20 GCTRL - RTS Controls And Readability

Goal: Make selection, command context, camera, and UI feedback readable under large-unit pressure.
Current evidence: prototype had selection rectangle, multi-squad commands, RTS camera, and intrusive UI lessons.
Non-goals: no oversized panels, no world-space UI for screen UI, no final minimap unless explicitly scoped later.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GCTRL-01 | repo_only | Alice worker | Define command feedback states: move, attack, cover, invalid, contested, owned. | `docs/gameplay/`, `client/godot/` | Docs-only/check-only |
| GCTRL-02 | repo_only | Bob worker | Route command context through client adapter as intent only. | `client/godot/`, `protocol/` | Godot check-only |
| GCTRL-03 | repo_only | Charlie worker | Add readability smoke for selection overlays under many visible proxies. | `client/godot/`, `tests/perf/` | render/readability smoke |

Gate queue: G-RENDER-TECH for final large-entity rendering choice.
Verification: player can tell what is selected, what command will be sent, and what feedback came from server state.

## RM21 GCOV - Cover Combat Authority

Goal: Rebuild cover, LOS/LOF, and firefight rules as authoritative server systems with readable client feedback.
Current evidence: prototype had cover slots, auto-cover, buildings blocking LOS/LOF, hit chance, and obstacle lessons.
Non-goals: no client-side combat outcomes, no expensive all-vs-all targeting, no final suppression until baseline is fun.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GCOV-01 | repo_only | Bob worker | Define obstacle, cover, LOS/LOF, and occupancy data model on server. | `server/`, `protocol/`, `docs/gameplay/` | Rust tests |
| GCOV-02 | repo_only | Bob worker | Add range-first targeting and cover effect tests. | `server/`, `tests/` | combat tests |
| GCOV-03 | repo_only | Alice worker | Define readable hit/miss/blocked/cover feedback language. | `docs/gameplay/`, `client/godot/` | Docs-only |
| GCOV-04 | repo_only | Charlie worker | Add cover combat perf smoke with dense squads. | `tests/perf/`, `server/` | perf smoke |

Gate queue: none.
Verification: cover changes firefights, buildings block shots/vision, and perf does not regress into O(n^2).

## RM22 GECON - Capture Economy Match Loop

Goal: Create a 10-minute MVP economy loop with credits, capture points, production, and simple win/loss candidates.
Current evidence: prototype had credits, capture points, HQ production, and 10-minute match target.
Non-goals: no complex economy, no meta progression, no full tech tree.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GECON-01 | repo_only | Bob worker | Implement authoritative credits, income ticks, capture state, and production spend. | `server/`, `protocol/` | economy tests |
| GECON-02 | repo_only | Alice worker | Define capture feedback, production queue clarity, and simple win/loss options. | `docs/gameplay/`, `client/godot/` | Docs-only |
| GECON-03 | repo_only | Charlie worker | Add 10-minute accelerated match simulation smoke. | `tests/`, `scripts/` | match sim smoke |

Gate queue: needs_design for final win condition after playtest evidence.
Verification: match has beginning, middle, end pressure without requiring debug tools.

## RM23 GROLE - Unit Role Expansion

Goal: Add unit roles one at a time only when each role creates a clear tactical choice and has server authority.
Current evidence: gameplay handoff recommends rifleman, rocket, MG, tank, mortar, dog/scout or officer.
Non-goals: no many-role content dump, no final art dependency, no abilities without command validation.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GROLE-01 | repo_only | Alice worker | Define role matrix: purpose, counterplay, cost, timing, command needs, feedback. | `docs/gameplay/` | Docs-only |
| GROLE-02 | repo_only | Bob worker | Implement rifle/basic infantry baseline and role data plumbing. | `server/`, `protocol/`, `client/godot/` | role tests |
| GROLE-03 | repo_only | Bob worker | Add rocket/MG/tank/mortar only as separate gated slices after baseline. | `server/`, `protocol/`, `client/godot/` | per-role tests |
| GROLE-04 | repo_only | Charlie worker | Add mixed-role faction performance scenario. | `tests/perf/`, `docs/perf/` | mixed-role perf |

Gate queue: G-GAMEPLAY-SCOPE for each new role after baseline.
Verification: every added role has tactical purpose, counterplay, cost/timing, UI support, and multiplayer authority path.

## RM24 GSWARM - Zombie Horde Scale Event

Goal: Build the zombie swarm as a late-match load and gameplay pressure event, not instant spawn spam.
Current evidence: prototype had timer, debug trigger, gradual 1,000 zombie target, route pressure, and aggro split idea.
Non-goals: no uncontrolled frame-drop spawn, no rich per-zombie scene tree, no final zombie art.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GSWARM-01 | repo_only | Bob worker | Implement authoritative swarm timer, gradual spawn, and route pressure baseline. | `server/`, `protocol/` | swarm tests |
| GSWARM-02 | repo_only | Bob worker | Add aggro trail split and AI LOD model. | `server/`, `docs/gameplay/` | AI tests/perf |
| GSWARM-03 | repo_only | Charlie worker | Add 1,000 zombie load scenario with network/render budget reporting. | `tests/perf/`, `client/godot/`, `server/` | swarm perf |
| GSWARM-04 | repo_only | Alice worker | Define player-facing swarm warning, debug trigger rules, and readability feedback. | `docs/gameplay/`, `docs/runbooks/` | Docs-only |

Gate queue: none.
Verification: swarm creates pressure while preserving server tick, bandwidth, and client render budgets.

## RM25 GBAL - Match Balance And Win Conditions

Goal: Turn the abstract playtest loop into a coherent 10-minute match with measurable pacing.
Current evidence: handoff proposes capture, destroy HQ, survive swarm, or hybrid win conditions.
Non-goals: no ranked balance, no meta progression, no deep faction asymmetry before infrastructure proof.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GBAL-01 | needs_design | Alice explorer | Choose initial win/loss rule after GCORE/GECON/GSWARM evidence. | read-only | No tests |
| GBAL-02 | repo_only | Bob worker | Add accelerated simulation tests for pacing once rule is chosen. | `server/`, `tests/` | pacing tests |
| GBAL-03 | repo_only | Charlie worker | Add playtest evidence template and balance telemetry checklist. | `docs/evidence/`, `docs/gameplay/` | Docs-only |

Gate queue: needs_design for first official MVP win condition.
Verification: 10-minute match has a readable beginning, middle, and end.

## RM26 GART - Art Return And Readability

Goal: Reintroduce generated/local art only after abstract gameplay and performance gates prove the shape of the game.
Current evidence: gameplay handoff says visual quality was not good enough and abstract readable forms are preferred for core testing.
Non-goals: no asset-first rebuild, no final style lock before gameplay readability.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GART-01 | needs_design | Alice explorer | Define art return criteria and category order. | read-only | No tests |
| GART-02 | repo_only | Charlie worker | Add asset readability/perf acceptance checklist. | `docs/art/`, `docs/perf/` | Docs-only |
| GART-03 | repo_only | Bob worker | Ensure render proxies can swap placeholder visuals without protocol changes. | `client/godot/` | render smoke |

Gate queue: G-ART-RETURN.
Verification: new art must improve readability and preserve performance before broad replacement.

## RM27 GNET - Godot Server Bridge

Goal: Build a clear Godot bridge to the dedicated server so UI/gameplay never talks directly to raw packets, Steam internals, or server state.
Current evidence: Godot client adapter exists as a roadmap, but the concrete Godot bridge modules are not yet specified.
Non-goals: no gameplay authority in Godot, no packet parsing inside UI scripts, no direct GodotSteam calls outside the Steam facade.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GNET-01 | repo_only | Bob worker | Specify `ServerConnection`, `ProtocolCodec`, `SnapshotBuffer`, `CommandQueue`, `ClientWorldState`, and `RenderAdapter` responsibilities. | `docs/architecture/`, `client/godot/` | Docs-only/check-only |
| GNET-02 | repo_only | Bob worker | Add Godot/Rust fixture contract for protocol version, handshake, command ack, full snapshot, and delta snapshot. | `docs/protocol/`, `protocol/fixtures/`, `client/godot/`, `server/` | fixture tests |
| GNET-03 | repo_only | Charlie worker | Define Godot main-thread rules: network receive queues bytes/state; scene mutations happen only through main-thread adapters. | `docs/architecture/`, `client/godot/` | Godot check-only |
| GNET-04 | needs_design | Alice explorer | Compare initial transport options for Godot: `PacketPeerUDP`, ENet, GDExtension, or later QUIC. | read-only | No tests |

Gate queue: G-GODOT-BRIDGE, G-TRANSPORT-QUIC.
Verification: Godot can connect, handshake, send commands, receive snapshots, and expose bridge counters without gameplay/UI coupling.

## RM28 GSCENE - Godot Scene And Node Architecture

Goal: Make every substantial Godot feature scene-first, editor-visible, and structured like a human would build it in Godot.
Current evidence: user explicitly requires clear `.tscn`, folder, node, and subnode structure; infrastructure handoff requires node-first/editor-first work.
Non-goals: no hidden deep NodePath coupling, no code-built fixed scene hierarchy, no rich per-unit scene tree for 10k-scale render proxies.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GSCENE-01 | repo_only | Alice worker | Document folder conventions, scene ownership, node trees, subnode names, and dynamic-node exceptions. | `docs/architecture/`, `client/godot/` | Docs-only |
| GSCENE-02 | repo_only | Charlie worker | Create scene checklist for every Godot slice: folders, `.tscn`, node tree, exported refs, groups, signals, Resources, perf budget. | `docs/architecture/`, `docs/runbooks/` | Docs-only |
| GSCENE-03 | repo_only | Bob worker | Define reusable scene/resource contracts for lobby shell, match root, world root, camera, UI HUD, render proxy host, debug overlay, capture point, HQ, and spawn marker. | `docs/architecture/`, `client/godot/` | Godot check-only |
| GSCENE-04 | repo_only | Charlie worker | Use Godot MCP/editor tools where available to create or inspect scenes instead of hand-writing scene files. | `client/godot/`, `docs/evidence/` | Godot MCP/editor evidence |

Gate queue: G-GODOT-SCENE-CONTRACT.
Verification: no substantial Godot implementation starts without a scene/node contract and performance note.

## RM29 GOBS - Logging Debug And Observability

Goal: Provide enough structured logs, counters, overlays, and evidence exports to debug networking, authority, rendering, performance, and gameplay at scale.
Current evidence: large multiplayer state will be hard to debug without early observability.
Non-goals: no secrets/tokens/private Steam tickets in logs, no noisy unstructured spam as the primary debug path.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| GOBS-01 | repo_only | Charlie worker | Define log event schema, categories, severity, redaction rules, and correlation IDs. | `docs/observability/`, `docs/architecture/` | Docs-only |
| GOBS-02 | repo_only | Bob worker | Add planned counters for server tick, command seq/ack, packet loss, snapshot decode/apply, AOI, reconnect, entity counts, and bandwidth. | `server/`, `client/godot/`, `docs/observability/` | logging tests |
| GOBS-03 | repo_only | Alice worker | Define debug overlay fields and operator-readable troubleshooting runbooks. | `docs/observability/`, `docs/runbooks/` | Docs-only |
| GOBS-04 | repo_only | Charlie worker | Add evidence export plan for logs, metrics, test reports, screenshots, and perf trend rows. | `docs/evidence/`, `docs/observability/`, `tests/` | evidence smoke |

Gate queue: G-OBSERVABILITY-BASELINE.
Verification: a failed local multi-client run can be diagnosed from logs/counters without guessing.

## RM30 PHIST - Performance History Ledger

Goal: Track performance over the life of the project, not just at one-off test moments.
Current evidence: performance gates exist, but historical trend documentation is now a hard requirement.
Non-goals: no manual feelings as evidence, no isolated benchmark numbers without scenario/version context.

Slice queue:

| Slice | Class | Owner | Objective | Allowed paths | Tests |
| --- | --- | --- | --- | --- | --- |
| PHIST-01 | repo_only | Charlie worker | Create performance ledger schema and scenario naming: 1k, 5k, 10k, faction mix, swarm, render-only, network-only, reconnect. | `docs/perf/`, `tests/perf/` | Docs-only/schema |
| PHIST-02 | repo_only | Bob worker | Emit machine-readable perf rows from server and client harnesses. | `server/`, `client/godot/`, `tests/perf/` | perf smoke |
| PHIST-03 | repo_only | Charlie worker | Add regression thresholds and "why changed" notes to every meaningful sim/render/network change. | `docs/perf/`, `docs/evidence/` | perf report |
| PHIST-04 | repo_only | Alice worker | Maintain human-readable performance changelog for user-facing project state. | `docs/perf/` | Docs-only |

Gate queue: G-PERF-HISTORY.
Verification: every scale claim includes date, scenario, commit/build, entity counts, tick time, bandwidth, Godot decode/apply/render cost, memory, and notes.
