# Local Synthetic Identity Handshake

Date: 2026-07-03
Status: slice TEST-02 local identity contract

## Purpose

Local same-workstation tests need stable session identity without live Steam.
This contract defines a synthetic identity path that can feed reconnect and
local multi-client work without pretending to be real authentication.

## Shape

Input:

```json
{
  "lobby_id": "local-abc123",
  "player_display_name": "Player 1",
  "identity_mode": "local_mock"
}
```

Godot facade output:

```json
{
  "player_session_id": "local-session-local-abc123-player-1"
}
```

Rust server utility:

```text
synthetic_session_id("local-abc123:Player 1") -> stable non-zero u64
```

The Rust value is for server-side deterministic tests. The Godot string is a
redacted facade/session token suitable for local UI and handoff dictionaries.
Neither is a Steam auth result.

## Rules

- Synthetic identity is local/mock only.
- Synthetic identity may be logged in local test output.
- Synthetic identity must not be used as proof of real Steam identity.
- Live Steam auth/session tickets remain blocked by `G-STEAM-AUTH`.
