# Godot Scene And Node Contract

Date: 2026-07-02
Applies to: `client/godot/`
Status: slice GSCENE-01 foundation contract

## Prime Rule

Build Godot like a human Godot developer would: scene-first, node-first,
editor-visible, typed, and inspector-friendly.

Use scripts for behavior and glue. Do not use scripts to hide fixed scene
structure that should be visible as `.tscn` files, nodes, subnodes, exported
references, Resources, signals, or groups.

When Godot MCP/editor tools are available, prefer them for creating, inspecting,
and validating scenes instead of manually writing `.tscn` files.

This contract is subordinate to `docs/architecture/authority-boundaries.md`:
Godot owns presentation, input collection, local prediction feedback, UI, and
debug overlays. The server owns durable match authority.

## Folder Layout

```text
client/godot/
  project.godot
  addons/
    godotsteam/
    godot_mcp/
  scenes/
    app/
      app_root.tscn
    lobby/
      lobby_screen.tscn
      player_slot.tscn
      ready_panel.tscn
    match/
      match_root.tscn
      world_root.tscn
      camera_rig.tscn
      render_proxy_host.tscn
      debug_overlay.tscn
    ui/
      hud_layer.tscn
      command_bar.tscn
      resource_panel.tscn
      selection_overlay.tscn
      network_status_panel.tscn
    gameplay/
      hq_marker.tscn
      capture_point.tscn
      spawn_marker.tscn
      cover_object.tscn
      obstacle_blocker.tscn
    dev/
      local_client_bootstrap.tscn
      perf_render_stress.tscn
      snapshot_fixture_viewer.tscn
  scripts/
    autoload/
      Online.gd
      ClientLog.gd
      PerfLedger.gd
    net/
      ServerConnection.gd
      ProtocolCodec.gd
      CommandQueue.gd
      SnapshotBuffer.gd
      ClientWorldState.gd
    render/
      RenderAdapter.gd
      RenderProxyPool.gd
      SelectionProjector.gd
    ui/
    gameplay/
    tests/
  resources/
    players/
    protocol/
    render/
    ui/
  perf/
  logs/
```

## Scene Ownership

`app_root.tscn`

```text
AppRoot (Node)
  Online (Node)
  ServerConnection (Node)
  SceneRouter (Node)
  ActiveScreen (Node)
```

Responsibilities:

- Owns app lifetime and scene switching.
- Does not own match simulation.
- Does not parse gameplay packets directly.

`lobby_screen.tscn`

```text
LobbyScreen (Control)
  Background (ColorRect)
  Layout (MarginContainer)
    MainPanel (VBoxContainer)
      HostJoinRow (HBoxContainer)
      LobbyIdRow (HBoxContainer)
      PlayerList (VBoxContainer)
      ReadyPanel (Control)
  StatusBar (Label)
```

Responsibilities:

- Host, join, copy lobby ID, ready state, player colors.
- Talks to `Online.gd` and a lobby adapter only.
- Never starts gameplay because a peer merely joined.

`match_root.tscn`

```text
MatchRoot (Node)
  WorldRoot (Node2D)
  CameraRig (Node2D)
  HudLayer (CanvasLayer)
  DebugOverlay (CanvasLayer)
  ClientSystems (Node)
    CommandQueue (Node)
    SnapshotBuffer (Node)
    ClientWorldState (Node)
    RenderAdapter (Node)
```

Responsibilities:

- Orchestrates client-side match systems.
- Applies server snapshots through adapters.
- Keeps UI in `CanvasLayer`.
- Does not decide movement, damage, capture, spawn validity, or combat.

`world_root.tscn`

```text
WorldRoot (Node2D)
  MapVisuals (Node2D)
  EditorPlacedGameplay (Node2D)
    SpawnMarkers (Node2D)
    CapturePoints (Node2D)
    Obstacles (Node2D)
    CoverObjects (Node2D)
  RuntimeEntities (Node2D)
    RenderProxyHost (Node2D)
    VfxHost (Node2D)
    ProjectileHost (Node2D)
```

Responsibilities:

- Editor-visible fixed layout lives under named containers.
- Dynamic render proxies, VFX, and projectiles live under runtime hosts.
- No hidden deep NodePath dependencies.

`render_proxy_host.tscn`

```text
RenderProxyHost (Node2D)
  InfantryBatch (MultiMeshInstance2D or Node2D pool host)
  VehicleBatch (MultiMeshInstance2D or Node2D pool host)
  ZombieBatch (MultiMeshInstance2D or Node2D pool host)
  AggregateMarkers (Node2D)
  SelectionProjector (Node2D)
```

Responsibilities:

- Renders snapshot state only.
- Uses pooling or batching for scale.
- Never creates rich child scene trees per entity in high-count paths.

`debug_overlay.tscn`

```text
DebugOverlay (CanvasLayer)
  Root (Control)
    NetworkPanel (PanelContainer)
    PerfPanel (PanelContainer)
    SelectionPanel (PanelContainer)
    LogPanel (PanelContainer)
```

Responsibilities:

- Shows server tick, client tick, command seq/ack, ping, jitter, dropped
  packets, snapshot age, visible entities, render proxies, decode/apply/render
  timings, memory, selected entity ID, and reconnect state.
- Must be toggleable.
- Must not log secrets or raw Steam auth tickets.

## Node-Type Rules

- Use `Node` for coordinators and pure systems.
- Use `Node2D` for world-space containers.
- Use `CanvasLayer` for screen-space UI.
- Use `Control` and normal Control containers for UI layout.
- Use `Marker2D` for spawn points, rally points, and authored anchors.
- Use `Area2D` plus `CollisionShape2D` for capture areas and click/hover zones.
- Use `StaticBody2D` only for Godot-side visual/editor collision helpers; the
  authoritative server still owns movement/collision truth.
- Use `MultiMeshInstance2D`, RenderingServer patterns, or pooled lightweight
  nodes for high-count visible entities.
- Use Resources for player metadata, visual tunables, command descriptions,
  render style, UI theme data, and protocol fixture metadata.

## Runtime Add-Child Exceptions

Runtime `add_child()` is allowed for:

- network-created render proxies,
- pooled VFX,
- projectiles,
- cursor/selection feedback,
- temporary debug markers,
- local dev/test harness helpers.

Runtime `add_child()` is not allowed for:

- fixed UI layout,
- lobby structure,
- authored map structure,
- reusable gameplay objects that should be scenes,
- capture points,
- HQ/building definitions,
- camera rig,
- debug overlay panels.

## Performance Contract

Every Godot feature slice must state:

- expected entity count,
- expected visible count,
- node count impact,
- per-frame allocation risk,
- decode/apply/render timing target,
- UI overlay cost,
- log volume impact,
- performance ledger scenario affected.

Minimum metrics:

- `godot_decode_ms`
- `godot_snapshot_apply_ms`
- `godot_render_update_ms`
- `godot_frame_ms`
- `visible_entities`
- `render_proxy_count`
- `node_count`
- `draw_calls` where available
- `memory_mb`

## Logging Contract

Godot logs must include:

- client index,
- local profile/session ID,
- connection ID,
- server tick,
- client tick,
- command sequence,
- last acknowledged command,
- snapshot tick,
- message type,
- entity ID when relevant,
- scene path or system name when relevant.

Do not log:

- Steam auth tickets,
- secrets,
- private account data,
- raw provider/session tokens,
- large packet dumps by default.

## Implementation Checklist

Before a Godot slice starts:

- folder path is named,
- scene file is named,
- root node type is chosen,
- required subnodes are listed,
- exported references/resources are listed,
- signals/groups are listed,
- dynamic runtime exceptions are listed,
- Godot MCP/editor use is planned where available,
- performance metrics are named,
- logs/debug overlay fields are named,
- tests/checks are named.

The per-slice runbook checklist lives in
`docs/runbooks/godot-slice-scene-checklist.md` and must be used for future
Godot-facing slices.
