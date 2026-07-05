# Interest Management V0

Date: 2026-07-03
Status: slices INT-01 and INT-02 foundation contract

## Purpose

Interest management decides which authoritative entity facts are visible to each
client before snapshot encoding. It is a server-side filter, not a gameplay rule
and not a Godot ownership boundary.

INT-01 defines:

- area-of-interest regions as deterministic spatial grid cell windows,
- per-client subscription state keyed by `player_session_id`,
- visible, entered, and left entity sets for later snapshot delta encoding.

INT-02 adds:

- visible entity delta snapshot selection,
- removed IDs for entities that left the client's AOI,
- aggregate far-state summaries for occupied cells outside the visible region.

## Authority

- The Rust server owns AOI membership.
- The Godot client receives already-filtered snapshots and must not request
  arbitrary entities by node path, resource path, or scene ownership.
- Steam lobby identity can select a session, but it does not own AOI membership.
- Reconnect keeps the same `player_session_id`; the server can rebuild the
  subscription before sending a full snapshot.

## Region Model

`AoiRegion` is:

| Field | Type | Meaning |
| --- | --- | --- |
| center | `SpatialCell` | Grid cell at the client's current focus or server-chosen camera anchor. |
| radius_cells | `u16` | Square radius in grid cells around `center`. |

The first implementation uses square cell windows because they are deterministic
and easy to diff. Later slices may add frustum, faction, stealth, aggregate far
state, or priority bands, but they must preserve server authority and replayable
ordering.

Cells are iterated in stable row-major order. Entity IDs are stored in sorted
sets before diffing so snapshot selection is deterministic across runs.

## Subscription State

`ClientInterestState` is keyed by `player_session_id` and stores:

- current `AoiRegion`,
- last visible entity set,
- next refresh result with:
  - `visible_entities`,
  - `entered_entities`,
  - `left_entities`.

`visible_entities` are emitted in deterministic entity-id order when a later
snapshot delta is built. `left_entities` become removed/hidden entity IDs.
`entered_entities` remain available for future create/update priority, but INT-02
does not yet distinguish dirty updates from unchanged visible entities.

## Delta Snapshot Selection

`build_visible_delta_snapshot` maps an `InterestUpdate` into a protocol v0 delta
snapshot:

- `visible_entities` become entity records when the authoritative entity state is
  still present,
- `left_entities` become removed entity IDs,
- missing entity states are omitted rather than invented,
- all selected IDs are already sorted by `BTreeSet` order.

This is still a server-side model. INT-02 does not write new binary fixtures or
change packet header fields.

## Aggregate Far State

`AggregateFarState` summarizes occupied cells outside the client's current
`AoiRegion`.

| Field | Meaning |
| --- | --- |
| `cell` | Spatial cell represented by the aggregate. |
| `entity_count` | Number of authoritative entities from that cell included in the aggregate. |
| `representative_entity_id` | Lowest entity ID in the aggregate for stable ordering/debugging. |
| `faction_mask` | Bit mask for faction IDs `0..63` present in the aggregate. |
| `flags_or` | Bitwise OR of authoritative entity flags in the aggregate. |

Aggregate far state is informational server output for later bandwidth and UX
work. It is not a fog-of-war, stealth, AI, combat, or gameplay rule.

## Protocol Impact

Protocol v0 snapshot payloads remain unchanged in INT-01. The filter runs before
payload construction:

1. Server builds or refreshes the spatial grid.
2. Server refreshes each client's `AoiRegion` subscription.
3. Later snapshot builders receive only the visible entity IDs for that client.
4. Delta builders select visible entity records and removed IDs from the
   `InterestUpdate`.
5. Far occupied cells can be summarized as `AggregateFarState`.

## Non-Goals

- No bandwidth claim.
- No AOI compression.
- No wire fixture for aggregate far-state yet.
- No stealth, fog-of-war, faction visibility, camera model, or gameplay rule.
- No client-owned visibility.
- No live networking or Steam state.

## Open Follow-Up

- INT-02: encode visible entity deltas and aggregate far state.
- INT-03: add bandwidth metrics and regression gates.
- GCORE/GCTRL later decide how player camera or command context influences the
  server-chosen AOI anchor without transferring authority to Godot.
