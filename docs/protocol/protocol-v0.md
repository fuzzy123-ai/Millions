# Protocol V0

Date: 2026-07-02
Status: slice PROTO-01 fixed wire contract draft

## Goals

- Compact binary protocol shared by Rust server and Godot client.
- Server authoritative state.
- Reliable/idempotent commands.
- Unreliable sequenced snapshots that can be superseded.
- Fixture-first validation before live network expansion.

## Byte Order

All integer and fixed-point fields are little-endian.
All packets are byte-aligned. Protocol v0 does not use varints, floats, Godot
Variant payloads, JSON payloads, scene paths, Resource paths, or Node names on
the wire.

## Header

Protocol v0 uses a fixed 48-byte header before the message-specific payload.
The header is present on every UDP datagram payload emitted by the transport
adapter.

| Field | Type | Notes |
| --- | --- | --- |
| magic | `u16` | Constant `0x4D4D` (`MM`) identifying Millions packets. |
| protocol_version | `u16` | `0` for `protocol_v0`. |
| message_type | `u8` | Handshake, command, ack, full snapshot, delta snapshot, disconnect, ping. |
| flags | `u8` | Reserved in v0. Must be `0`; non-zero is rejected until a later gate defines bits. |
| header_len | `u16` | Must be `48` in v0. Allows future extension while preserving decode. |
| payload_len | `u32` | Hard bounded by protocol hardening gate. |
| connection_id | `u64` | Transport connection identity, not player authority. |
| session_id | `u64` | Stable player session identity after handshake. |
| client_seq | `u32` | Monotonic client command packet sequence. |
| server_seq | `u32` | Monotonic server packet sequence. |
| ack_seq | `u32` | Highest received sequence for the relevant channel. |
| tick | `u64` | Server tick for snapshots, client target tick for commands where applicable. |

Header offsets:

| Offset | Size | Field |
| ---: | ---: | --- |
| 0 | 2 | magic |
| 2 | 2 | protocol_version |
| 4 | 1 | message_type |
| 5 | 1 | flags |
| 6 | 2 | header_len |
| 8 | 4 | payload_len |
| 12 | 8 | connection_id |
| 20 | 8 | session_id |
| 28 | 4 | client_seq |
| 32 | 4 | server_seq |
| 36 | 4 | ack_seq |
| 40 | 8 | tick |

Decoders must reject packets before payload allocation when `magic`,
`protocol_version`, `flags`, `header_len`, or `payload_len` fail validation.
`payload_len` excludes the 48-byte header.

## Message Types

| ID | Name | Direction | Reliability | Payload summary |
| ---: | --- | --- | --- | --- |
| 1 | `client_hello` | client to server | reliable by retry | client protocol capabilities and requested identity mode |
| 2 | `server_hello` | server to client | reliable by retry | accepted connection/session IDs and server tick rate |
| 3 | `client_command_batch` | client to server | reliable, ordered, idempotent | one or more intent commands |
| 4 | `server_command_ack` | server to client | reliable enough by repetition | accepted/rejected command range and reason codes |
| 5 | `server_full_snapshot` | server to client | unreliable sequenced | complete visible authoritative state for the client |
| 6 | `server_delta_snapshot` | server to client | unreliable sequenced, superseded | entity create/update/remove deltas since a prior snapshot |
| 7 | `ping` | either | unreliable | timestamp echo and current seq/ack values |
| 8 | `disconnect` | either | best effort | reason code and optional redacted diagnostic code |

IDs `0` and `9..255` are reserved. Unknown message types are rejected and
counted by hardening metrics without mutating authoritative state.

## Identity

- `connection_id` can change across reconnect.
- `player_session_id` remains stable across reconnect grace period.
- Local mode uses synthetic identities with the same handshake shape as Steam mode.
- Steam auth ticket bytes must never be logged or stored in protocol fixtures.

## Command Rules

- Commands are reliable, ordered, acknowledged, and idempotent.
- Each command batch uses header `client_seq`; each command inside the batch
  also carries a `command_id` unique within the stable `session_id`.
- Replayed or stale commands must be ignored without side effects.
- Commands express intent only: select, move, attack, take cover, spawn, produce,
  ready, reconnect.
- Client messages must not contain Godot scene paths, Resources, Node data, or
  client-side gameplay outcomes.

`client_seq` starts at `1` for a connection and increases by one for every
`client_command_batch`. A reconnect can receive a new `connection_id`; command
idempotency remains keyed by `session_id + command_id` so retrying the last
accepted command cannot double-apply it.

The command payload starts with:

| Field | Type | Notes |
| --- | --- | --- |
| command_count | `u16` | Number of commands in the batch. |
| reserved | `u16` | Must be `0`. |

Each command entry starts with:

| Field | Type | Notes |
| --- | --- | --- |
| command_id | `u64` | Stable idempotency key scoped to `session_id`. |
| command_type | `u16` | Intent type. Gameplay-specific values are assigned only after infrastructure gates. |
| command_len | `u16` | Bytes in the command-specific payload. |
| target_tick | `u64` | Client's intended server tick; server may clamp or reject. |

Command type IDs reserved by protocol v0:

| ID | Name | Notes |
| ---: | --- | --- |
| 1 | `ready` | Lobby/session readiness intent. |
| 2 | `select_group` | Selection intent only; no authority transfer. |
| 3 | `move` | Target position intent. |
| 4 | `attack` | Target entity or position intent. |
| 5 | `take_cover` | Cover intent; map authority remains server-side. |
| 6 | `spawn_request` | Production/spawn request intent. |
| 7 | `reconnect_resume` | Resume existing `session_id`. |

Gameplay slices may add command payload fields, but they cannot add client-owned
outcomes or bypass command idempotency.

## Sequence And Ack Rules

- Client-to-server reliable command channel uses `client_seq` and server
  acknowledgements in `server_command_ack`.
- Server-to-client snapshot channel uses `server_seq`; the client reports the
  highest received server sequence in `ack_seq` on subsequent client packets.
- `ack_seq` acknowledges packet sequence, not gameplay authority.
- Gap detection can request or trigger a full snapshot, but missing deltas do
  not block applying a later full snapshot.
- Sequence values are unsigned 32-bit and wrap only after explicit hardening
  tests define wrap handling.

Ack payload:

| Field | Type | Notes |
| --- | --- | --- |
| acked_client_seq | `u32` | Highest client packet sequence processed. |
| first_command_id | `u64` | First command id covered by this ack range, or `0`. |
| command_count | `u16` | Number of command results following. |
| reserved | `u16` | Must be `0`. |
| result entries | repeated | `command_id: u64`, `status: u16`, `reason: u16`. |

Status `1` means accepted, `2` means duplicate ignored, `3` means rejected.
Reason `0` means no extra reason. Other reason codes are defined by later
hardening and gameplay slices.

## Snapshot Rules

- Snapshots are unreliable sequenced state from the authoritative server.
- Lost delta snapshots are superseded by later deltas or a full snapshot.
- Reconnect starts with a full snapshot, then resumes delta stream.
- AOI/visibility decides which entity states a client receives.
- Far state can be aggregated when detail is outside the client's area of interest.

The INT-01/INT-02 area-of-interest contract lives in
`docs/protocol/interest-management-v0.md`. Protocol v0 payload headers do not
change for INT-02; the server filters entity IDs per `player_session_id` before
delta snapshot construction. `visible_entities` become entity records and
`left_entities` become removed entity IDs. Aggregate far-state is modeled on the
server side for later bandwidth work and is not a wire fixture yet.

Full snapshot payload starts with:

| Field | Type | Notes |
| --- | --- | --- |
| snapshot_id | `u64` | Monotonic server snapshot identifier. |
| baseline_snapshot_id | `u64` | `0` for full snapshots. |
| entity_count | `u32` | Number of entity records. |
| removed_count | `u32` | Must be `0` for full snapshots. |

Delta snapshot payload starts with:

| Field | Type | Notes |
| --- | --- | --- |
| snapshot_id | `u64` | Monotonic server snapshot identifier. |
| baseline_snapshot_id | `u64` | Snapshot this delta is based on. |
| entity_count | `u32` | Number of create/update entity records. |
| removed_count | `u32` | Number of removed entity IDs following the entity records. |

Entity records are authoritative render-state facts, not Godot node updates:

| Field | Type | Notes |
| --- | --- | --- |
| entity_id | `u64` | Stable server entity ID. |
| entity_kind | `u16` | Abstract kind/category, not scene/resource path. |
| faction_id | `u16` | Stable numeric faction ID. |
| flags | `u32` | Bitset for alive/visible/aggregate/etc.; v0 fixture bits start at `0`. |
| x_mm | `i32` | World x position in millimeters. |
| y_mm | `i32` | World y position in millimeters. |
| facing_millirad | `i32` | Facing angle in milliradians. |
| health_q8 | `u16` | Health ratio in unsigned Q8.8, clamped to `0..256`. |
| state_id | `u16` | Abstract animation/state hint for client rendering. |
| state_param_q8 | `i16` | Optional bounded state parameter in Q8.8. |
| reserved | `u16` | Must be `0`. |

Removed entity lists contain repeated `entity_id: u64` values.

## Quantization Defaults

- Positions use fixed-point integers, not floating-point wire data.
- Angles, health, progress, and timers use bounded integer ranges.
- Entity IDs, player IDs, faction IDs, role IDs, and command IDs are stable numeric IDs.

Initial quantization table:

| Domain | Wire type | Unit/range | Decode note |
| --- | --- | --- | --- |
| World position | `i32` | millimeters | Client converts to visual units only after decode. |
| Facing | `i32` | milliradians | Normalize on decode to renderer convention. |
| Health ratio | `u16` | Q8.8, `0..256` | `256` means full health. |
| Progress ratio | `u16` | Q8.8, `0..256` | Used for production/capture progress later. |
| Tick | `u64` | server ticks at 20 Hz | Tick rate is also sent in `server_hello`. |
| IDs | `u64`/`u16` | numeric only | Never reuse during an active match unless a later replay-safe rule exists. |

## Hardening Defaults

- Unknown protocol version: reject with explicit reason.
- Unknown message type: reject and count.
- Oversized payload: reject before allocation.
- Malformed payload: reject without changing authoritative state.
- Stale/replayed command: acknowledge or report according to command channel rules,
  but do not reapply.
- Backpressure and rate-limit caps are defined in
  `docs/protocol/backpressure-rate-limits.md`. Over-limit commands can be
  dropped or rejected without mutating authoritative state.
- Packet size, version mismatch, malformed handling, auth failure, stale
  command, and replay rejection actions are defined in
  `docs/protocol/protocol-hardening-v0.md`.

## Fixture Plan

The first fixtures should cover:

- handshake request/accept,
- command with seq/ack,
- duplicate command,
- full snapshot,
- delta snapshot,
- missing-delta recovery,
- reconnect full snapshot,
- malformed packet rejection.
