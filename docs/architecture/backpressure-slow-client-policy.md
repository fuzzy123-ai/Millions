# Backpressure And Slow-Client Policy

Date: 2026-07-03
Slice: LOADSHED-01
Status: policy contract before implementation

## Purpose

The server owns overload decisions. Clients may receive feedback, but clients do
not decide whether a command is accepted, degraded, dropped, retried, or whether
a session is disconnected.

## Decision Order

1. Validate protocol header and payload bounds before allocation.
2. Evaluate per-client command rate and pending reliable commands.
3. Evaluate server-to-client reliable backlog and oldest unacked packet age.
4. Evaluate estimated bandwidth per client.
5. Evaluate log volume so rejection loops do not flood evidence.
6. Apply the most severe safe action: `accept`, `degrade`, `drop_command`, then
   `disconnect`.

## Authority Rules

- Dropped commands must not mutate authoritative simulation state.
- Duplicate or stale commands remain idempotent and are not counted as accepted
  gameplay outcomes.
- Degradation may reduce optional snapshot detail, use aggregate far-state, or
  skip non-essential diagnostics.
- Degradation must not hide authoritative corrections or make Godot
  authoritative.
- Disconnect must preserve enough redacted evidence for triage without logging
  secrets, Steam tickets, private account data, or raw live provider output.

## LOADSHED Roadmap Split

- LOADSHED-01 defines caps and the Rust decision surface.
- LOADSHED-02 implements slow-client degrade policy in server/protocol paths.
- LOADSHED-03 adds overload harness cases.
- LOADSHED-04 adds operator-visible states.

## Open Limits

- No real socket transport is measured here.
- No live Steam or two-machine scenario is used.
- No gameplay command balance is implied.
- Final QUIC/custom UDP choice remains gated by transport design work.
