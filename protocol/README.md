# Protocol

Shared protocol assets live here.

- `docs/protocol/protocol-v0.md` is the human-readable draft.
- `protocol/fixtures/` will hold binary and JSON-descriptor fixtures.
- Rust and Godot must validate identical fixture interpretation before live networking expands.

No fixture may contain secrets, Steam auth tickets, private account data, or raw provider/session tokens.

## Reconnect Session Contract

`player_session_id` is the stable server-owned identity for reconnect and
command idempotency. `connection_id` is transport-local and may change when a
client reconnects.

`server/src/reconnect.rs` owns the current repo-only rebind model:

- a connected session accepts only its active `connection_id`,
- disconnect moves the session into a bounded grace period,
- a new `connection_id` can rebind only during grace,
- the previous connection is no longer accepted after rebind,
- a successful rebind requires a full snapshot before delta resume,
- expired sessions reject late rebind attempts.

This contract is local/mock and deterministic. Real Steam auth/session tickets
remain outside fixtures and require the `G-STEAM-AUTH` live gate.

## Steam Bridge V0

`steam_bridge_v0` is the local/mock facade handoff shape between the Steam lobby
facade and the dedicated-server handshake path. It carries redacted lobby
metadata, endpoint mode, endpoint epoch, ready epoch, build label, display name,
and local/mock `player_session_id`. It does not carry real Steam auth tickets,
real AppID assumptions, provider tokens, or gameplay authority.

Fixture:

- `protocol/fixtures/steam_bridge_v0_local_handoff_mock.json`

## Command Reliability Contract

`server/src/reliability.rs` owns the current repo-only command reliability
model:

- `client_seq` starts at `1` and must arrive as the next monotonic value,
- duplicate or stale `client_seq` values are ignored without side effects,
- gaps are detected without advancing accepted sequence state,
- `command_id` is an idempotency key scoped to the stable player session,
- server reliable packets stay pending until `ack_seq` removes them,
- resend is driven by bounded tick age and increments a resend counter.

`client/godot/scripts/net/CommandQueue.gd` mirrors the client-side adapter
surface by assigning local `command_id` values, exposing resend candidates,
marking resend attempts, and clearing acknowledged commands. It does not decide
whether a gameplay command succeeds.

## GCORE Local Command Contract

`server/src/game_core.rs` owns the first local, abstract gameplay core contract:

- the server creates authoritative HQ and basic squad entity IDs,
- one basic squad may be spawned per started local player,
- move intents are accepted only for the owning player session,
- `command_id` is idempotent per `player_session_id`,
- out-of-bounds targets reject before authoritative state changes,
- accepted move intents store a target only; they do not pathfind, collide,
  attack, spend economy, or claim final balance.

`client/godot/scripts/gameplay/GCoreIntent.gd` creates intent-only dictionaries
for local tests. Godot does not decide command success or mutate authoritative
positions.

## GCTRL Command Context Contract

`client/godot/scripts/net/ClientAdapter.gd` may route local command context
through `queue_command_context` for future controls/readability checks. The
context can carry selected authoritative entity IDs, a bounded target kind, and
the local feedback state `command_intent_queued`.

Allowed target kinds are `none`, `world_position`, `entity_id`,
`map_marker_id`, and `cover_candidate_id`. Unknown local target kinds normalize
to `none` before reaching the pending command queue.

This is a Godot adapter payload contract, not a new wire format. The Rust server
still owns command acceptance, rejection reason codes, idempotency, target
validity, authoritative state changes, and snapshot correction.

## GCOV Cover Authority Contract

`server/src/cover.rs` owns the first server-side obstacle, cover, line-of-sight,
line-of-fire, and cover occupancy data model. It imports only validated
`mapdata_v0` shapes and keeps map ID, version, and checksum with the server
authority surface.

GCOV-01 does not add a protocol message or wire fixture. Future attack,
take-cover, hit/miss, blocked, damage, suppression, or cover-bonus commands must
add explicit protocol evidence before clients can rely on those results.

GCOV-02 keeps range-first targeting classification inside the Rust server model.
`InRangeClear`, `InRangeTargetInCover`, `BlockedByObstacle`, and `OutOfRange`
are not wire messages yet. A future protocol slice must define bounded server
ack/reject or result payloads before Godot can display them as authoritative
combat outcomes.

## GSWARM Local Swarm Contract

`server/src/swarm.rs` owns the first repo-only zombie swarm scheduling contract:

- the server owns swarm start tick, spawn interval, batch size, active cap, and
  deterministic spawn-point rotation,
- spawned swarm entity IDs are authoritative server IDs,
- route pressure is an aggregate server-side sample by route target position,
- no client may treat route pressure as movement, aggro, combat, economy, or
  final balance authority.

GSWARM-01 does not add a protocol message, binary fixture, Godot UI event, live
networking behavior, or client-side swarm authority. Future swarm warning,
AI LOD, aggregate far-state, and render/load reporting slices must add explicit
protocol or adapter evidence before clients rely on them.

## Interest Management Contract

`docs/protocol/interest-management-v0.md` owns the current AOI and visibility
contract. Server-side interest management:

- keys subscriptions by stable `player_session_id`,
- selects visible entity IDs from deterministic spatial-grid AOI regions,
- emits visible entity records and removed IDs for later delta snapshots,
- summarizes far occupied cells as aggregate far-state for later bandwidth work.

This is a server-owned visibility filter, not a Godot authority surface. INT-02
does not add live networking, binary aggregate fixtures, fog-of-war, stealth, or
gameplay visibility rules.

## Backpressure And Slow-Client Contract

`server/src/load_shed.rs` owns the current repo-only backpressure and
slow-client policy model:

- per-client command rate, pending reliable commands, reliable backlog, resend
  window, bandwidth, and log-volume caps are bounded,
- over-rate command pressure drops new commands without mutating authoritative
  state,
- bandwidth or slow-client backlog pressure degrades snapshots before
  disconnecting,
- degradation can reduce delta rate or switch far-state detail to aggregate
  summaries,
- exhausted resend windows or hard backlog overflow disconnect the connection
  and require full-snapshot resume,
- optional diagnostics can be disabled before log volume floods evidence.

This contract is local/mock and deterministic. It does not measure live
transport behavior, call Steam, choose final transport technology, or implement
gameplay fairness.

## Protocol Hardening Contract

`server/src/hardening.rs` owns the current repo-only protocol hardening model:

- packets are capped at 1200 bytes,
- payloads are capped at 1152 bytes after the fixed 48-byte header,
- client command batches are capped at 1024 bytes,
- unsupported protocol versions are rejected as version mismatches,
- malformed headers and future malformed payloads reject without authoritative
  state mutation,
- missing local/mock auth proof rejects without mutation,
- rejected auth proof disconnects with redacted diagnostics only,
- stale or replayed commands are acknowledged as duplicates without mutation.

`docs/protocol/protocol-hardening-v0.md` is the human-readable contract. Real
Steam auth/session tickets, public-network exposure, broader playtests, and
release-candidate hardening claims remain gated.
