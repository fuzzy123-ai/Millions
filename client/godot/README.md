# Millions Godot Client

This folder is the scene-first Godot client root.

Initial decision:

- Use GDScript for the first network/render/client foundation.
- Do not require Godot C#/.NET for the first client spike.
- Keep C# as a later `needs_design` decision only if profiling or tooling proves it useful.
- Fixed UI and authored world structure belong in `.tscn` scenes.
- High-count entities must use pooled or batched render proxies, not rich per-unit scene trees.

Required contract:

- Follow `docs/architecture/godot-scene-node-contract.md`.
- Follow `docs/architecture/authority-boundaries.md` for server/client authority.
- Follow `docs/architecture/godot-server-bridge.md` for network bridge responsibilities.
- Follow `docs/architecture/godot-main-thread-rules.md` before adding async receive/decode work.
- Follow `docs/architecture/godot-client-adapter-contract.md` before UI/input talks to network state.
- Follow `docs/architecture/godot-reusable-scene-resource-contracts.md` before adding reusable scenes or Resources.
- Keep raw packets inside `scripts/net/`.
- Keep server state application inside `ClientWorldState` and `RenderAdapter`.
- `RenderAdapter` may derive sorted render batches from authoritative snapshot
  state, but those batches are copied render inputs only; they must not mutate
  `ClientWorldState` or decide gameplay outcomes.
- During reconnect resume, `ClientAdapter` must wait for a server-marked full
  snapshot, clear stale client world state when that full snapshot arrives, and
  only then resume applying delta snapshots.
- Render stress scenes may use abstract placeholder proxies for budget and
  adapter validation, but they must not choose final render technology or add
  gameplay authority.
- Godot MCP/editor evidence is tracked in
  `docs/evidence/godot-mcp-editor-evidence.md`; if the editor is not connected,
  use headless checks and record the connection state.
- Observability counters use `PerfLedger.gd` and the baseline schema in
  `docs/observability/log-event-schema.md`.
- Backpressure status surfaces must follow
  `docs/runbooks/backpressure-operator-states.md`: Godot may display
  `normal`, `degraded`, `commands_limited`, `disconnect_pending`,
  `disconnected`, and `blocked_live_validation` as read-only diagnostics, but
  the server remains authoritative for overload actions.
- Authored map markers must follow
  `docs/architecture/authored-map-data-contract.md`: Godot may be the
  scene-first editing source for map markers, but server authority requires a
  versioned export, checksum, server import validation, and evidence before map
  data affects gameplay.
- Movement feedback must follow `docs/gameplay/movement-feedback-contract.md`:
  Godot may preview, explain, and reconcile movement intent later, but server
  snapshots and rejection reasons remain authoritative.
- Command feedback must follow `docs/gameplay/command-feedback-states.md`:
  Godot may show selection, context, queued, sent, acknowledged, rejected,
  stale, reconciled, and degraded feedback states, but the Rust server remains
  authoritative for command success, rejection, and durable match state.
- Cover combat feedback must follow
  `docs/gameplay/cover-combat-feedback-language.md`: Godot may later present
  server-owned clear, in-cover, blocked-by-obstacle, out-of-range, hit, or miss
  states, but it must not decide combat success, damage, cover effect, or
  target legality.
- Swarm render stress must follow
  `docs/gameplay/swarm-behavior-contract.md`: Godot may render local proxy
  density, LOD buckets, aggro lane markers, and collision-radius metadata for
  headless checks, but it must not decide swarm behavior, collision resolution,
  pathfinding, or authoritative horde state.
- `scripts/gameplay/GCoreIntent.gd` may create local HQ/squad/move intent
  dictionaries for tests, but the Rust server remains authoritative for spawn
  acceptance, ownership checks, idempotency, target validity, and snapshots.
- `scripts/gameplay/GCoreLocalMatchSmoke.gd` is a local abstract two-client
  smoke harness that feeds server-shaped snapshot dictionaries into
  `ClientWorldState` and `RenderAdapter` for proxy/batch readiness only.
- Do not put gameplay authority in Godot.
