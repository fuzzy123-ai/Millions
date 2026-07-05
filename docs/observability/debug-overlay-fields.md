# Debug Overlay Fields

Date: 2026-07-03
Slice: GOBS-03
Status: planned debug overlay field contract

## Purpose

The Godot debug overlay should help operators and developers understand local
multiplayer state without exposing secrets or making Godot authoritative. This
document defines the first planned overlay panels and fields.

This is a contract only. It does not implement UI, choose final UI design, or
open gameplay authority.

## Panels

| Panel | Purpose | Default visibility |
| --- | --- | --- |
| Network | Connection, packet, seq/ack, and reconnect health. | collapsed |
| Session | Local/mock identity, lobby/session state, ready state. | collapsed |
| Command | Pending commands, resend, ack/reject state. | collapsed |
| Snapshot | Snapshot stream, AOI, entity counts, stale/full snapshot state. | collapsed |
| Render | Render adapter, proxy count, batch count, frame/render metrics. | collapsed |
| Performance | Budget-adjacent Godot/server counters and p95 report summary. | collapsed |
| Redaction | Redaction status and blocked unsafe fields. | collapsed |

## Field Matrix

| Field | Panel | Source | Redaction/authority rule |
| --- | --- | --- | --- |
| `client_id` | Session | harness/Godot profile | local label only |
| `connection_id` | Network | ServerConnection/transport | diagnostic only, may change on reconnect |
| `player_session_id` | Session | server/local mock handoff | synthetic or redacted display only |
| `endpoint_mode` | Session | SteamLobbyFacade | safe mode label, no real ticket |
| `endpoint_epoch` | Session | SteamLobbyFacade | numeric/local only |
| `ready_epoch` | Session | SteamLobbyFacade | numeric/local only |
| `client_seq` | Network | CommandQueue/ProtocolCodec | diagnostic only |
| `ack_seq` | Network | ServerConnection/ProtocolCodec | diagnostic only |
| `pending_command_count` | Command | PerfLedger/CommandQueue | counter only |
| `last_command_id` | Command | CommandQueue | numeric only |
| `last_command_status` | Command | server ack state later | accepted/rejected/pending text only |
| `server_tick` | Snapshot | snapshot header | server truth display, not client-owned |
| `snapshot_id` | Snapshot | snapshot payload later | numeric only |
| `snapshot_age_ms` | Snapshot | client local clock | diagnostic only |
| `buffered_snapshot_count` | Snapshot | SnapshotBuffer/PerfLedger | counter only |
| `authoritative_entity_count` | Snapshot | ClientWorldState/PerfLedger | mirror of server facts |
| `visible_entities` | Snapshot | AOI/snapshot counters | mirror of server facts |
| `render_proxy_count` | Render | RenderAdapter/PerfLedger | presentation counter |
| `render_batch_count` | Render | RenderAdapter/PerfLedger | presentation counter |
| `godot_decode_ms` | Performance | future timing surface | local metric only |
| `godot_snapshot_apply_ms` | Performance | future timing surface | local metric only |
| `godot_render_update_ms` | Performance | future timing surface | local metric only |
| `godot_frame_ms` | Performance | future timing surface | local metric only |
| `backpressure_events` | Performance | server/loadshedding later | counter/evidence only |
| `redaction_events` | Redaction | logging/security | no raw secret display |

## Display Rules

- Overlay panels default collapsed and must be toggleable.
- Field labels must be stable enough for evidence screenshots or text exports.
- Unknown fields display `unknown` or `blocked`, not invented values.
- Live Steam tickets, provider tokens, private account data, raw packets, and
  private provider payloads are never displayed.
- Godot may display authoritative facts after adapter application, but it must
  not turn display state into match authority.

## Done Rules For Future Overlay Implementation

A future overlay implementation is not done until:

- it consumes `PerfLedger`, adapter status, or redacted facade state only,
- it avoids raw packet and secret display,
- it includes a headless or screenshot evidence path,
- it records fields in a runbook or evidence artifact,
- it preserves `docs/architecture/godot-scene-node-contract.md`.
