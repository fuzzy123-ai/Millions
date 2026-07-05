# Map Data Authority Notes

Date: 2026-07-03
Slice: `MAPDATA-01`
Status: gameplay-facing authority notes

## Rule

Cover, obstacles, spawn points, capture points, and navigation data are not
final gameplay rules until the server imports, validates, and gates a versioned
map data artifact for the specific gameplay system. Godot-authored markers are
useful for editing and local checks, but they do not make gameplay decisions by
themselves. Current repo-only consumers may use validated local map-data
fixtures for bounded evidence, such as the swarm static-obstacle movement smoke.

## Allowed Before Server Import

- naming map markers,
- placing visual/editor markers,
- documenting stable IDs,
- defining export fields,
- preparing local fixtures,
- validating that Godot scene structure is readable and bounded.

## Not Allowed Before Server Import

- authoritative collision or movement from Godot nodes,
- cover bonus or line-of-fire decisions,
- spawn validity,
- capture ownership changes,
- navigation/pathfinding outcomes,
- economy or win/loss outcomes tied to map markers,
- release or playtest claims based on editor-only map data.

## Handoff To Later Slices

- `MAPDATA-02` defines server import format and validation rules.
- `MAPDATA-03` adds export/checksum fixture tests.
- `MAPDATA-04` adds the change checklist and runbook.
- The swarm movement smoke now consumes validated local obstacle map data as a
  bounded backend fixture, not as live/exported gameplay authority.

Until those are done, map-data gameplay remains a gate, not an implementation
surface.
