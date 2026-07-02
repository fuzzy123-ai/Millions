# Millions Risk And Gap Register

Date: 2026-07-02
Source handoff: `E:/TankGame/docs/server_client_infrastructure_handoff.md`
Mode: Standard ABC

This register turns the largest open architecture risks into explicit decisions,
safe preparation tasks, and gates. It is intentionally architecture-first: no
prototype gameplay, maps, assets, or combat systems are carried forward.

## Executive Decisions

| ID | Risk / Gap | Current decision | Owner | Gate |
| --- | --- | --- | --- | --- |
| RG-01 | Transport choice is open. | Use a transport abstraction immediately. First implementation is custom UDP with explicit channels, sequence numbers, acks, and resend for commands. Defer QUIC until Godot integration cost is proven worth it. | Bob | G-TRANSPORT-QUIC |
| RG-02 | Binary protocol is conceptual only. | Define `protocol_v0` before server/client code. Fixed little-endian binary header, versioned message types, stable numeric IDs, quantized positions. | Bob | none |
| RG-03 | Steam lobby to dedicated-server session mapping is undefined. | Steam remains identity/lobby/discovery. Dedicated server owns `player_session_id`; Steam ID/auth ticket binds to that session. Local mode uses synthetic identity with the same handshake shape. | Alice + Bob | G-STEAM-AUTH |
| RG-04 | Interest management is not concrete. | Use a spatial grid/AOI subscription model from the first 1k server spike. All-entity snapshots are allowed only in isolated test modes. | Bob | none |
| RG-05 | Reconnect behavior is easy to bolt on too late. | Reconnect is part of the first spike: stable session, grace period, full snapshot, resume deltas. | Bob | none |
| RG-06 | Godot may render too many rich nodes. | First renderer uses simple batched proxies and records render budget. Rich scenes stay out of the spike. | Charlie | G-RENDER-TECH |
| RG-07 | Multi-client test setup can diverge from Steam tests. | Maintain two explicit modes: local direct server mode and Steam lobby mode. Both must stay runnable after architecture changes. | Charlie | G-STEAM-TWO-MACHINE |
| RG-08 | 10k entity scale can become a late surprise. | Add automated 1k, 5k, and 10k performance gates before real gameplay. A feature is not ready if it breaks the harness. | Charlie | none |
| RG-09 | Rust server crate choices may cause churn. | Keep server module boundaries independent of ECS and transport crates. Choose minimal crates for spike; swap behind adapters only. | Bob | G-ECS-CHOICE |
| RG-10 | Protocol/schema drift between Godot and Rust. | Generate or validate shared packet fixtures. Both sides must pass fixture encode/decode tests. | Bob + Charlie | none |
| RG-11 | Authority rules may leak into UI/client code. | Client code talks only to a network adapter and renderer-state adapter. No gameplay authority in Godot. | Alice + Charlie | none |
| RG-12 | Steam AppID ambiguity can invalidate tests. | Keep `TANKGAME_STEAM_APP_ID` and project setting behavior. `480` remains fallback only. Do not invent a real AppID. | Alice | G-REAL-APPID |
| RG-13 | Gameplay scope can overtake infrastructure. | Gameplay is the first test target and load profile, not the first implementation layer. Add only abstract gameplay slices after protocol/server/client harness gates exist. | Alice + Charlie | G-GAMEPLAY-SCOPE |
| RG-14 | Current prototype gameplay may be accidentally copied. | Treat TankGame gameplay as behavioral requirements and regression ideas only. Rebuild server-authoritative versions from protocol/contracts. | Alice | none |
| RG-15 | "Huge units for all factions" can become an undefined scale target. | Define faction-scale scenarios: two player factions first, then multi-faction bots/AI, then zombie swarm pressure, all measured by role mix and per-client AOI. | Bob + Charlie | G-FACTION-COUNT |
| RG-16 | Zombie horde can destroy tick/render/network budgets. | Swarm is a dedicated load profile with gradual spawn, AI LOD, route/stream behavior, and aggregate far-state replication. | Bob | none |
| RG-17 | Asset/style work can distort gameplay readability tests. | Use abstract readable shapes for infrastructure and gameplay feel until playtest loop and performance gates are stable. Art return pass is deferred. | Alice | G-ART-RETURN |
| RG-18 | Godot can drift into code-built scene structure. | Every reusable/fixed Godot feature must have a planned folder, `.tscn` scene, node tree, and ownership boundary before implementation. Use editor/Godot MCP tools where available. | Alice + Charlie | G-GODOT-SCENE-CONTRACT |
| RG-19 | Godot server bridge can become mixed with UI/gameplay code. | Add a dedicated Godot server bridge with `ServerConnection`, `ProtocolCodec`, `SnapshotBuffer`, `CommandQueue`, `ClientWorldState`, and `RenderAdapter`. | Bob + Charlie | G-GODOT-BRIDGE |
| RG-20 | Performance history can be lost between spikes. | Maintain a performance ledger from the first running harness, tracking server, network, Godot decode/apply/render, memory, and regressions over time. | Charlie | G-PERF-HISTORY |
| RG-21 | Debugging large multiplayer state can be blind. | Build structured logging and observability early: log categories, tick/session/entity IDs, counters, overlays, and exportable evidence. | Charlie | G-OBSERVABILITY-BASELINE |

## Gate Queue

Gate: G-TRANSPORT-QUIC
Class: needs_design
Blocks: adopting QUIC as the main live transport
Decision needed: choose QUIC only after a small Godot integration spike proves client support, packaging, and debugging are acceptable.
Safe preparation done: custom UDP transport abstraction can proceed.
Risk if bypassed: protocol work may become blocked on client library/friction instead of simulation needs.
Next safe slice: PROTO-01

Gate: G-STEAM-AUTH
Class: needs_live_go
Blocks: real Steam auth/session ticket validation against a dedicated server
Decision needed: confirm real Steam AppID and auth validation path for playtest.
Safe preparation done: local synthetic identity and Steam-ID-shaped interface can be implemented.
Risk if bypassed: local tests may pass while real playtest identity/reconnect fails.
Next safe slice: STEAM-01

Gate: G-RENDER-TECH
Class: needs_design
Blocks: committing to final Godot render technology for large armies
Decision needed: choose MultiMesh, RenderingServer, sprite batch, or hybrid after render-only stress evidence.
Safe preparation done: abstract render proxy API can proceed.
Risk if bypassed: client architecture may be shaped around a renderer that cannot hit the budget.
Next safe slice: REND-01

Gate: G-STEAM-TWO-MACHINE
Class: needs_live_go
Blocks: release-candidate confidence
Decision needed: run a real two-machine Steam smoke test with configured AppID.
Safe preparation done: local direct mode and Steam lobby preservation can proceed.
Risk if bypassed: same-workstation success may hide Steam/session/relay failures.
Next safe slice: LOBBY-01

Gate: G-ECS-CHOICE
Class: needs_design
Blocks: locking long-term Rust ECS/data layout
Decision needed: choose `hecs`, `bevy_ecs`, or custom SoA after 1k/5k measurements.
Safe preparation done: implement server state behind a minimal simulation interface.
Risk if bypassed: early crate choice may force costly rewrites before scale evidence.
Next safe slice: SIM-01

Gate: G-REAL-APPID
Class: needs_live_go
Blocks: real release/playtest Steam identity
Decision needed: provide assigned Steam AppID when available.
Safe preparation done: AppID configurability and Spacewar fallback remain.
Risk if bypassed: Steam will show Spacewar and auth/release expectations may be misleading.
Next safe slice: LOBBY-01

Gate: G-GAMEPLAY-SCOPE
Class: needs_design
Blocks: adding real gameplay before infrastructure contracts exist
Decision needed: only move a gameplay slice from design to implementation when its required protocol, server authority, client adapter, harness, and performance gates are named.
Safe preparation done: gameplay can be converted into test scenarios and abstract contracts.
Risk if bypassed: the project recreates prototype coupling and loses the server-authoritative path.
Next safe slice: GPLAN-01

Gate: G-FACTION-COUNT
Class: needs_design
Blocks: final "all factions" scale target
Decision needed: choose initial faction count and target unit counts per faction for the first scale test. Default proposal: 2 player factions, neutral capture systems, zombie faction; later 4+ simulated factions.
Safe preparation done: scenario matrix can define scalable placeholders without final faction lore/art.
Risk if bypassed: performance numbers become impressive but not representative of intended gameplay.
Next safe slice: GLOAD-01

Gate: G-ART-RETURN
Class: needs_design
Blocks: replacing abstract placeholders with final/generated art
Decision needed: return to art only after gameplay readability, scale, and UI clarity are proven.
Safe preparation done: abstract render proxies and role silhouettes can proceed.
Risk if bypassed: visual work may hide simulation/readability problems and increase rendering cost too early.
Next safe slice: GART-01

Gate: G-GODOT-SCENE-CONTRACT
Class: needs_design
Blocks: building any substantial Godot client feature
Decision needed: approve the scene-first structure for folders, scenes, nodes, subnodes, ownership, and dynamic-node exceptions.
Safe preparation done: a Godot scene/node contract can be documented before project scaffolding.
Risk if bypassed: Godot code will accumulate hidden node creation, NodePath coupling, and performance-hostile scene trees.
Next safe slice: GSCENE-01

Gate: G-GODOT-BRIDGE
Class: needs_design
Blocks: final Godot-to-server implementation approach
Decision needed: choose initial client transport path: GDScript `PacketPeerUDP` for spike, ENet, GDExtension, or later QUIC.
Safe preparation done: Godot bridge boundaries and fixture tests can be specified without committing to the final transport.
Risk if bypassed: UI/gameplay may couple directly to packets, Steam, or local prototype state.
Next safe slice: GNET-01

Gate: G-PERF-HISTORY
Class: repo_only
Blocks: calling any scale milestone complete
Decision needed: none for baseline; the project must create a durable performance ledger before 1k/5k/10k claims.
Safe preparation done: metrics schema and evidence folders can be created immediately.
Risk if bypassed: regressions will be anecdotal and hard to attribute.
Next safe slice: PHIST-01

Gate: G-OBSERVABILITY-BASELINE
Class: repo_only
Blocks: meaningful multiplayer debugging at scale
Decision needed: none for baseline; choose log categories, event schema, and debug overlay fields before complex gameplay.
Safe preparation done: logging contracts can be documented before code.
Risk if bypassed: command loss, desync, AOI bugs, render stalls, and reconnect failures become difficult to diagnose.
Next safe slice: GOBS-01

## Clarified Architecture Defaults

- Godot is a renderer/input/UI client.
- Rust server is authoritative for match state.
- Commands are reliable, ordered, idempotent, and acknowledged.
- Snapshots are unreliable sequenced; lost snapshots are superseded.
- Reconnect is designed from the first spike.
- Interest management exists before 5k and 10k targets.
- Local multi-client testing and real Steam testing are both first-class.
- Live packets are compact binary, not JSON, scene paths, Resources, or Node data.
- Gameplay content starts only after protocol, harness, and scale gates exist.
- Gameplay handoff content is used first as infrastructure validation scenarios:
  playtest core, role mix, cover combat, capture economy, and zombie swarm load.
- Godot work is scene-first/editor-first: reusable or fixed structure must be
  `.tscn`/Resource/inspector-visible before script-driven shortcuts.
- Runtime `add_child()` is reserved for truly dynamic entities: render proxies,
  cursors, VFX, projectiles, temporary feedback, and pooled runtime objects.
- Performance history and structured logs are project artifacts, not optional
  troubleshooting notes.
