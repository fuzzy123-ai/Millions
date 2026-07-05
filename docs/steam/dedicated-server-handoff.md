# Dedicated Server Handoff

Date: 2026-07-03
Status: slice LOBBY-02 facade contract

## Purpose

This contract defines the local/mock Steam lobby facade shape that hands a
client to the dedicated server adapter. It is repo-only and does not call live
Steam APIs.

## Godot Facade

`client/godot/scripts/net/SteamLobbyFacade.gd`

Responsibilities:

- own local/mock lobby state transitions,
- produce copyable local lobby IDs,
- queue ready intent as `ready_pending`,
- accept mock server readiness into `ready`,
- derive local/mock synthetic session IDs for offline harnesses,
- produce a redacted dedicated-server handoff dictionary,
- avoid live Steam APIs, real AppID assumptions, secrets, tickets, or provider
  tokens.

## Handoff Dictionary

```json
{
  "ok": true,
  "lobby_id": "local-abc123",
  "identity_mode": "local_mock",
  "server_endpoint": "127.0.0.1:7777",
  "player_display_name": "Player 1",
  "player_session_id": "local-session-local-abc123-player-1",
  "state": "launching"
}
```

The synthetic session ID shape is specified in
`docs/protocol/local-identity-handshake.md`. It is a local/mock identifier for
same-workstation harnesses and reconnect preparation, not a Steam auth result.

STEAM-02 should extend this local/mock shape with the metadata keys and epoch
rules in `docs/steam/session-advertisement-contract.md` before any live Steam
validation is considered.

## Check

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_godot_lobby_facade_check.ps1
```

Expected terminal line:

```text
steam_lobby_facade_check status=ok state=in_match endpoint=127.0.0.1:7777
```

## Live Gates

Still requires explicit fresh Go:

- real Steam auth/session ticket validation,
- real AppID release/playtest identity,
- two-machine Steam smoke,
- release-candidate confidence claims.
