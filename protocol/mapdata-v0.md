# Mapdata V0 Import Fields

Date: 2026-07-03
Slice: `MAPDATA-02`
Status: local import field contract

## Scope

`mapdata_v0` is a repo-local server import contract for authored map facts. It
is not a network protocol message and is not sent over `protocol_v0` packets.
Future exporters may serialize it as JSON or another deterministic artifact,
but the server authority boundary is the same: import, validate, checksum, then
use only validated facts.

## Top-Level Fields

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schema` | string | yes | Must be `millions_mapdata_v0`. |
| `map_id` | string | yes | Stable authored map ID, not a scene path. |
| `map_version` | unsigned integer | yes | Monotonic authored version. |
| `checksum` | string | yes | Filled by exporter/checksum slice later. |
| `bounds_mm` | object | yes | `min_x`, `min_y`, `max_x`, `max_y` in millimeters. |
| `spawn_points` | array | yes | Stable spawn marker facts. |
| `capture_points` | array | yes | Stable capture marker facts. |
| `obstacles` | array | yes | Stable blocker shape facts. |
| `cover_objects` | array | yes | Stable cover shape facts. |
| `navigation_hints` | array | yes | Coarse navigation hint facts. |

## Server Validator

`server/src/map_data.rs` owns the current validation surface. Later import
parsers must produce the same typed shape before gameplay systems can consume
map data.

## Non-Claims

This contract does not define live networking, packet payloads, Godot scene
serialization, gameplay collision, cover effects, spawn validity, capture
ownership, navigation/pathfinding, movement, or release evidence.
