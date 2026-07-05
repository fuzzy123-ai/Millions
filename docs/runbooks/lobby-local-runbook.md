# Local Lobby Runbook

Date: 2026-07-03
Status: slice LOBBY-01 local/mock runbook

## Scope

This runbook covers local/mock lobby preparation only. It does not grant live
Steam permission and does not require Steam to be installed or running.

## Local Mock Flow

1. Start in `offline`.
2. Host creates a synthetic `local-...` lobby ID.
3. Joiner enters or receives that synthetic ID.
4. Both clients show `joined`.
5. Ready button queues ready intent and shows `ready_pending`.
6. Mock facade or local server handshake accepts readiness and shows `ready`.
7. Dedicated-server endpoint handoff begins only after local readiness is
   accepted.

## Go/No-Go

Go without asking again:

- docs, facade contracts, local/mock identity models,
- same-workstation local IDs,
- redacted logs and UI state labels,
- adapter methods that do not call Steam.

Needs explicit fresh Go:

- live Steam API calls,
- real Steam auth/session tickets,
- real AppID assumptions,
- two-machine live smoke,
- release-candidate confidence claims.

## Evidence

Future lobby slices should record:

- lobby state transitions,
- copied lobby ID format,
- ready pending/accepted transitions,
- endpoint handoff shape,
- redaction check for logs and fixtures.
