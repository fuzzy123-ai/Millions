# Snapshot Render Smoke

Date: 2026-07-03
Status: slice CLNT-03 local Godot smoke

## Purpose

This smoke verifies that a local authoritative snapshot dictionary can pass
through `ClientAdapter`, `SnapshotBuffer`, `ClientWorldState`, and
`RenderAdapter`, then create one simple runtime render proxy under a scene-owned
`RenderProxyHost`.

It does not open sockets, call Steam, implement gameplay, or choose final render
technology.

## Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_godot_snapshot_render_smoke.ps1
```

Expected terminal line:

```text
snapshot_render_smoke status=ok proxies=1 entities=1 pending=1
```

## Scene

`client/godot/scenes/dev/snapshot_render_smoke.tscn`

```text
SnapshotRenderSmoke (Node2D)
  RuntimeEntities (Node2D)
    RenderProxyHost (Node2D)
```

Runtime proxy creation is allowed here because high-count render proxies are a
documented dynamic-node exception in the Godot scene contract.
