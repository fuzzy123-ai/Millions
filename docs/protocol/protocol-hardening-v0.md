# Protocol Hardening V0

Date: 2026-07-03
Slice: `HARDEN-01`
Status: repo-only hardening contract

## Purpose

This document defines the first protocol hardening surface for packet size
limits, version mismatch behavior, malformed handling, stale/replay rejection,
and auth failure states. It is local Foundation infrastructure only. It does not
open a public server, expose a dedicated server to broader playtests, validate
live Steam tickets, or claim release readiness.

The Rust assertion surface lives in:

```text
server/src/hardening.rs
```

## Size Limits

| Limit | Default | Reason |
| --- | ---: | --- |
| Max packet bytes | 1200 | Stay within the current Foundation datagram budget before transport finalization. |
| Max payload bytes | 1152 | Max packet bytes minus the fixed 48-byte protocol header. |
| Max client command batch bytes | 1024 | Leave headroom for header/transport metadata and prevent command spam allocations. |
| Max redacted diagnostic bytes | 128 | Keep rejection evidence bounded and secret-safe. |

Packets larger than the max packet size are disconnected before payload decode.
Declared payloads larger than the max payload size are rejected without
authoritative state mutation. Client command batches above their smaller cap are
also rejected without authoritative state mutation.

## Reject Reasons

| Reason | Meaning | Required action |
| --- | --- | --- |
| `packet_too_large` | Datagram exceeds local hard cap. | `disconnect` before payload decode. |
| `payload_too_large` | Header declares a payload beyond the hard cap. | `reject_no_state_mutation`. |
| `client_command_batch_too_large` | Command batch payload exceeds command cap. | `reject_no_state_mutation`. |
| `version_mismatch` | Packet uses a protocol version other than v0. | `reject_no_state_mutation`. |
| `malformed_header` | Magic, flags, header length, message type, or payload length are invalid. | `reject_no_state_mutation`. |
| `malformed_payload` | Message-specific payload fails structural validation. | `reject_no_state_mutation`. |
| `auth_missing` | Handshake requires proof that is absent. | `reject_no_state_mutation`. |
| `auth_rejected` | Provided auth proof is invalid. | `disconnect`. |
| `stale_command` | Command sequence is older than the accepted window. | `ack_duplicate_no_state_mutation`. |
| `replayed_command` | Command idempotency key was already processed. | `ack_duplicate_no_state_mutation`. |

## Version And Malformed Handling

Decoders must reject packets before payload allocation when header checks fail.
Unsupported versions are separated from other malformed header failures so
operators can distinguish protocol drift from hostile or corrupted traffic.

Malformed payload validation is reserved for message-specific parsers in
`HARDEN-02`; the action word is already defined here so later parsers share the
same no-mutation contract.

## Auth Failure States

`HARDEN-01` defines local/mock auth state handling only:

- `local_mock_accepted`: accepted for local harnesses.
- `auth_missing`: rejected without state mutation.
- `auth_rejected`: disconnect with redacted diagnostics only.

Real Steam session tickets, AppID validation, and provider calls remain behind
`G-STEAM-AUTH` and `G-REAL-APPID`. No ticket bytes or raw provider output may be
persisted in fixtures, logs, docs, or handoffs.

## Stale And Replay Rejection

Stale or replayed commands must be acknowledged or reported as duplicates
without mutating authoritative simulation state. This preserves idempotency and
keeps reconnect/retry behavior deterministic. It does not define command
payload semantics or gameplay balance.

## Claim Limits

This contract supports local parser and policy tests only. Broader external
playtest exposure remains blocked until the HARDEN roadmap adds parser,
property/fuzz, hostile-input harness, and privacy logging coverage, and until
any live/release gates receive explicit fresh Go.
