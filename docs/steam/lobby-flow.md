# Steam Lobby Flow

Date: 2026-07-03
Status: slice LOBBY-01 repo-only contract

## Purpose

Steam is a discovery, identity, and lobby convenience layer. It is not match
authority. The dedicated Rust server assigns or resumes `player_session_id` and
owns readiness, admission, and all match state.

## Lobby States

| State | Meaning | Allowed without live Go |
| --- | --- | --- |
| `offline` | No local lobby/session connection. | yes |
| `local_mock` | Local synthetic lobby identity for same-workstation tests. | yes |
| `creating` | User requested host flow; adapter preparing metadata. | yes for mock |
| `joinable` | Lobby ID can be copied/shared. | yes for mock/local ID |
| `joining` | User entered lobby ID or accepted invite. | yes for mock |
| `joined` | Local user is present in lobby member list. | yes for mock |
| `ready_pending` | Ready intent queued but not accepted by server. | yes |
| `ready` | Server/lobby facade accepted readiness. | yes for mock |
| `launching` | Dedicated-server endpoint/handshake flow begins. | yes for local mock |
| `in_match` | Match adapter owns server connection. | yes for local mock |
| `left` | User left lobby or returned offline. | yes |
| `error` | Redacted, operator-visible failure. | yes |

Real Steam lobby state, real Steam auth/session ticket validation, real AppID
behavior, and two-machine smoke remain explicit live gates.

## Copy ID UX

- The lobby ID must be copyable as plain text from the lobby screen.
- Local/mock mode may use a synthetic ID with a visible `local-` prefix.
- Real Steam IDs must be treated as identifiers, not secrets, but logs should
  avoid noisy dumps and should redact associated auth/session data.
- Copy success/failure is UI-local and does not imply match readiness.

## Ready Semantics

- Ready is a client intent, not final authority.
- UI can show `ready_pending` immediately after user input.
- `ready` requires acceptance from the lobby/session facade or server-side
  handshake path.
- Leaving, reconnecting, or changing lobby membership clears local pending
  readiness unless a later server/session rule says otherwise.
- A peer joining does not auto-start gameplay.

## AppID Policy

- AppID must be configurable and never hardcoded as a release assumption.
- Spacewar or a mock/local mode may be used for development documentation and
  adapter shape only.
- Real AppID playtest/release identity requires explicit fresh Go.
- No Steam auth tickets, provider tokens, or private account data may be stored
  in fixtures, logs, docs, or tests.

## Dedicated-Server Handoff Shape

Lobby/session preparation hands the client adapter a redacted connection model:

```json
{
  "lobby_id": "local-abc123",
  "identity_mode": "local_mock|steam",
  "server_endpoint": "127.0.0.1:0",
  "player_display_name": "Player 1",
  "player_session_id": "assigned-by-server-after-handshake"
}
```

`player_session_id` is stable only after server acceptance. Before that, lobby
identity is discovery/session intent, not server authority.

`docs/steam/dedicated-server-handoff.md` defines the current local/mock Godot
facade and headless interface check.

`docs/steam/session-advertisement-contract.md` defines the STEAM-01 metadata
keys, endpoint advertisement shape, and ready-to-match sequence that future
local/mock and live-gated Steam facade work must share.

## Stop Rules

Stop and gate the slice if work would:

- call live Steam APIs,
- validate or persist real Steam auth/session tickets,
- assume a real AppID,
- require two-machine live smoke,
- auto-start gameplay from lobby state,
- make Steam or Godot authoritative for match state.
