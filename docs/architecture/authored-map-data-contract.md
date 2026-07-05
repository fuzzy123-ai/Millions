# Authored Map Data Contract

Date: 2026-07-03
Slice: `MAPDATA-01`
Status: repo-only authored-data contract

## Purpose

This contract defines how Godot-authored map structure becomes eligible for
server authority later. Godot may be the editing source for map visuals and
authoring markers, but runtime gameplay authority requires a versioned export,
checksum, server import validation, and JSON handoff before cover, obstacles,
spawn points, capture points, or navigation data can affect match outcomes.

## Authored Source

Godot may author these map facts as scene-visible nodes or resources:

- map identity and version label,
- map bounds,
- spawn marker positions and faction/team labels,
- capture point positions, radius, and stable IDs,
- obstacle blockers with stable IDs and simple shapes,
- cover objects with stable IDs and cover class labels,
- navigation regions or coarse blocked-cell hints,
- visual-only terrain labels that are explicitly non-authoritative.

Authored nodes should live under the scene-first structure documented in
`docs/architecture/godot-scene-node-contract.md`, especially `WorldRoot` and
`EditorPlacedGameplay`.

## Authority Boundary

Godot-authored map data is not authoritative until all of these exist:

- export format,
- schema version,
- stable map ID,
- deterministic checksum,
- server import validator,
- fixture or smoke test,
- evidence row or handoff,
- plan update.

Before those exist, authored map nodes are editor placeholders and local
contract inputs only. They must not decide movement, cover, capture, spawn,
combat, navigation, economy, win/loss, or visibility outcomes.

## Export Shape Preview

Later MAPDATA slices may define the exact server import format. The first
authoring contract reserves these top-level fields:

```json
{
  "schema": "millions_mapdata_v0",
  "map_id": "local-dev-map",
  "map_version": 1,
  "checksum": "filled-by-exporter-later",
  "bounds_mm": { "min_x": 0, "min_y": 0, "max_x": 100000, "max_y": 100000 },
  "spawn_points": [],
  "capture_points": [],
  "obstacles": [],
  "cover_objects": [],
  "navigation_hints": []
}
```

All IDs are stable strings or numeric IDs assigned by the authored data
pipeline. Scene paths, Node names, Resource paths, and editor-only display names
must not be the authoritative IDs consumed by the server.

The MAPDATA-02 server import contract lives in
`docs/architecture/map-data-server-import.md`; field-level notes live in
`protocol/mapdata-v0.md`. Safe change order and required local checks live in
`docs/runbooks/map-data-change-checklist.md`.

## Privacy And Evidence

Map data artifacts must be repo-local and must not include:

- secrets,
- private account data,
- Steam tickets or provider output,
- absolute private machine paths,
- live endpoint details,
- generated art/provider metadata unless explicitly allowed by a future asset
  gate.

## Non-Claims

This contract does not implement the server importer, Godot exporter, checksum
fixture, navigation, cover gameplay, spawn logic, capture logic, or movement
model. It only defines the authored-data boundary needed before those slices
can proceed safely.
