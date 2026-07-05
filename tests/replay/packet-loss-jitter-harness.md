# Packet Loss And Jitter Harness

Date: 2026-07-03
Status: slice LOSS-02 local synthetic harness

`tests/replay/packet-loss-jitter-cases.json` defines deterministic local cases
for command ordering, duplicate/stale detection, gap detection, ack removal, and
resend due checks.

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_packet_loss_jitter_harness.ps1
```

Expected final line:

```text
packet_loss_jitter_harness status=ok cases=2 mode=local_synthetic
```

The harness does not open sockets, call Steam, mutate public network state, or
assign gameplay meaning to commands.
