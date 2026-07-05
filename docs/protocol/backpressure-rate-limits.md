# Backpressure And Rate Limits

Date: 2026-07-03
Slice: LOADSHED-01
Status: foundation contract

## Purpose

This document defines the first overload caps for command rate, reliable
commands, resend backlog, bandwidth, and log volume. It is a repo-only
Foundation contract. It does not expose a public server, run live networking, or
make multiplayer scale claims.

The Rust assertion surface lives in:

```text
server/src/load_shed.rs
```

## Default Caps

| Cap | Default | Reason |
| --- | ---: | --- |
| Commands per second per client | 30 | Keep input spam bounded before gameplay commands exist. |
| Pending reliable commands per client | 128 | Prevent unbounded command queues. |
| Reliable backlog packets per client | 256 | Bound retransmit memory and disconnect hopeless clients. |
| Slow-client backlog packets | 64 | Begin degradation before the hard backlog cap. |
| Resend window | 40 ticks | Two seconds at the current 20 Hz server tick. |
| Bandwidth per client | 256 KB/s | Matches the normal AOI provisional budget. |
| Log events per minute per client | 120 | Prevent repeated rejects from flooding local evidence. |

These are provisional Foundation caps. Changing them requires updating this
document, the Rust defaults, the JSON plan handoff, and the mandatory plan
checks.

## Server Actions

| Action | Meaning |
| --- | --- |
| `accept` | The client is within current caps. |
| `degrade` | Prefer smaller or less frequent snapshots, aggregate far state, or reduced optional diagnostics. |
| `drop_command` | Drop or reject incoming commands without mutating authoritative state. |
| `disconnect` | End the session/connection when resend backlog or age exceeds hard safety bounds. |

LOADSHED-01 defines these actions only. LOADSHED-02 owns the slow-client degrade
policy that applies them to actual server paths.

## Claim Limits

Rows or logs from this contract are local infrastructure evidence only. They do
not prove live network behavior, Steam behavior, soak stability, release
readiness, gameplay fairness, or final transport choice.
