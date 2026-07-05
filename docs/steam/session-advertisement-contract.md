# Steam Session Advertisement Contract

Date: 2026-07-03
Status: slice STEAM-01 repo-only contract

## Purpose

This contract defines the lobby metadata, dedicated-server endpoint
advertisement, and ready-to-match sequence that the Steam facade and local/mock
facade must share. It is safe preparation for STEAM-02 and does not call live
Steam APIs, validate real Steam auth tickets, assume a real AppID, or require a
two-machine smoke.

Steam remains discovery, identity, and lobby convenience. The Rust dedicated
server owns admission, `player_session_id`, readiness acceptance, reconnect, and
all match state.

## Metadata Envelope

Lobby metadata is a compact set of ASCII string keys. Future live Steam metadata
must use the same keys as local/mock metadata so the Godot facade, local harness,
and dedicated-server handshake stay testable without live Steam.

| Key | Required | Local/mock example | Meaning |
| --- | --- | --- | --- |
| `millions.schema` | yes | `steam_bridge_v0` | Contract version for metadata parsing. |
| `millions.identity_mode` | yes | `local_mock` | `local_mock` now; `steam` only after live Go. |
| `millions.lobby_state` | yes | `joinable` | Facade state from `docs/steam/lobby-flow.md`. |
| `millions.protocol` | yes | `protocol_v0` | Client/server protocol compatibility label. |
| `millions.server_mode` | yes | `local_direct` | Endpoint handoff mode; not gameplay authority. |
| `millions.endpoint` | yes for ready | `127.0.0.1:7777` | Redacted host:port or loopback endpoint. |
| `millions.endpoint_epoch` | yes for ready | `1` | Monotonic string number for endpoint refresh. |
| `millions.ready_epoch` | yes | `0` | Monotonic string number for ready-state refresh. |
| `millions.build_id` | yes | `local-uncommitted` | Comparable build/evidence label. |
| `millions.host_slot` | yes | `0` | Stable lobby host slot, not account identity. |

Metadata must not contain:

- Steam auth/session tickets,
- provider tokens,
- private account data,
- raw Steam persona names if they are not already intentionally displayable,
- public server credentials,
- unredacted IPs for non-local test environments,
- gameplay state or authority decisions.

## Endpoint Advertisement

Endpoint advertisement is a lobby-to-client hint that a dedicated-server
handshake may begin. It is not admission and not proof of readiness.

Safe local/mock endpoint modes:

| Mode | Endpoint shape | Meaning |
| --- | --- | --- |
| `local_direct` | `127.0.0.1:<port>` | Same-workstation local harness. |
| `local_config` | `config:<profile>` | Endpoint comes from a checked-in local profile. |
| `pending` | empty string | Host has not advertised an endpoint yet. |

Future live modes stay gated:

| Mode | Gate | Why gated |
| --- | --- | --- |
| `steam_live` | `G-STEAM-AUTH` | Requires live Steam session context. |
| `real_appid` | `G-REAL-APPID` | Requires real release/playtest identity. |
| `two_machine` | `G-STEAM-TWO-MACHINE` | Requires live two-machine evidence. |

The facade may cache the latest endpoint metadata, but the client may connect
only after the ready-to-match sequence has produced a valid handoff dictionary.

## Ready-To-Match Sequence

The sequence is deliberately split so UI, Steam facade, and server authority do
not blur together.

| Step | Producer | State/Event | Authority |
| --- | --- | --- | --- |
| 1 | Godot UI or harness | user joins or creates lobby | UI intent only |
| 2 | Steam/local facade | metadata state becomes `joinable` or `joined` | lobby discovery |
| 3 | Godot UI or harness | user presses ready | client intent only |
| 4 | Steam/local facade | `ready_pending`, increment `millions.ready_epoch` | facade bookkeeping |
| 5 | Local/mock server handshake | endpoint accepted, session derived or assigned | server authority |
| 6 | Steam/local facade | metadata state becomes `ready`, endpoint published | handoff hint |
| 7 | Godot adapter | consumes handoff dictionary | client connection setup |
| 8 | Dedicated server | accepts protocol hello and owns `player_session_id` | match authority |

The facade must clear or supersede pending readiness when lobby membership,
endpoint epoch, protocol version, or server build label changes. A client may
show a pending or ready UI state, but only the server acceptance path can make a
player match-ready.

## Handoff Dictionary

STEAM-02 should implement the facade-shaped local/mock handoff with this shape:

```json
{
  "ok": true,
  "schema": "steam_bridge_v0",
  "identity_mode": "local_mock",
  "lobby_id": "local-abc123",
  "lobby_state": "ready",
  "protocol": "protocol_v0",
  "server_mode": "local_direct",
  "server_endpoint": "127.0.0.1:7777",
  "endpoint_epoch": 1,
  "ready_epoch": 1,
  "build_id": "local-uncommitted",
  "player_display_name": "Player 1",
  "player_session_id": "local-session-local-abc123-player-1"
}
```

`player_session_id` is local/mock in this dictionary. In live Steam mode it still
must be server-owned and must not be treated as a Steam account identifier or
auth ticket.

## STEAM-02 Acceptance Checklist

STEAM-02 may proceed without fresh operator Go if it:

- implements only local/mock metadata and handoff dictionaries,
- reuses the keys in this contract,
- keeps raw Steam tickets and provider data out of logs, fixtures, docs, and
  tests,
- proves ready cannot skip `ready_pending`,
- proves endpoint changes increment or supersede endpoint epoch,
- proves the dedicated-server handshake interface receives a redacted handoff,
- leaves live Steam validation, real AppID behavior, and two-machine smoke
  behind their gates.

## Stop Rules

Stop and record a gate if implementation would:

- call live Steam APIs,
- request, validate, persist, or log real Steam auth/session tickets,
- assume a real AppID,
- require a public endpoint or deploy,
- require a two-machine live smoke,
- let lobby metadata start gameplay or decide match authority.
