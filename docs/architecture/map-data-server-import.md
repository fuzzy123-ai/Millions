# Map Data Server Import

Date: 2026-07-03
Slice: `MAPDATA-02`
Status: repo-only server import contract

## Purpose

This document defines the first server-side map data import shape and
validation rules. It turns the MAPDATA-01 authored-data contract into a local
Rust validation surface without implementing a Godot exporter, JSON parser,
checksum fixture, navigation, cover gameplay, spawn logic, or capture logic.

The Rust assertion surface lives in:

```text
server/src/map_data.rs
```

## Import Shape

The server import model reserves:

- `schema = "millions_mapdata_v0"`,
- stable `map_id`,
- numeric `map_version`,
- non-empty checksum string,
- millimeter `bounds_mm`,
- `spawn_points`,
- `capture_points`,
- `obstacles`,
- `cover_objects`,
- `navigation_hints`.

Marker IDs must be stable IDs from the authored data pipeline. Scene paths,
Node names, Resource paths, and editor-only display names are not authoritative
IDs.

## Validation Rules

The current local validator rejects:

- unsupported schema,
- missing map ID,
- missing checksum,
- invalid bounds smaller than 1000 mm or larger than 1000000 mm per axis,
- more than 4096 markers per marker kind,
- duplicate or empty IDs across all marker kinds,
- points outside map bounds,
- zero capture radius,
- shapes with non-positive extents,
- shape rows placed in the wrong marker kind list.

Validation is deterministic and repo-local. It does not read files, parse JSON,
open sockets, call Steam, or inspect Godot scenes.

## Authority Boundary

Passing `validate_map_data_import` only means the local import structure is
well-formed. Current repo-only consumers may use that validated structure for
bounded local evidence, such as the swarm movement smoke translating `obstacles`
into immovable static collision bodies and local flow-field blocker cells using
their half-extents plus swarm-radius clearance. This still does not make
live/exported map data authoritative for gameplay until later MAPDATA slices add
end-to-end export evidence and release-quality operator gates.

The current checksum fixture is documented in `tests/fixtures/README.md`.
Operator-safe change order and required checks are documented in
`docs/runbooks/map-data-change-checklist.md`.

## Non-Claims

MAPDATA-02 does not implement final gameplay collision, cover effects, spawn
validity, capture ownership, navigation/pathfinding, movement rules, economy,
win/loss, live networking, Steam, or release-candidate evidence.
