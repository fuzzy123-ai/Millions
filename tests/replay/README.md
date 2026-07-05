# Replay Tests

Replay tests record deterministic command streams and compare server
checksums/snapshots to find first divergent tick.

Current repo-only smoke:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_replay_smoke.ps1
```

Expected final line:

```text
replay_smoke status=ok scope=server_replay
```

The DET-02 recorder groups deterministic inputs by server tick, canonicalizes
input order, emits per-frame checksums, and reports the first divergent tick
between two reports. It does not execute gameplay or open live networking.
