# Desync Smoke

Date: 2026-07-03
Status: slice DET-03 golden checksum fixture

## Purpose

The desync smoke compares deterministic replay frame checksums before gameplay
systems exist. It is a local fixture/evidence check, not a live multiplayer
claim.

## Fixture

`tests/replay/golden-input-checksums.json`

The fixture records:

- seed inputs,
- derived simulation seed,
- canonical input facts,
- expected frame checksums,
- a payload-changed variant with a different checksum.

## Current Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_replay_smoke.ps1
```

Expected final line:

```text
replay_smoke status=ok scope=server_replay
```

## Stop Rules

Stop and gate the smoke if it would require live networking, real Steam,
private account data, raw provider output, gameplay authority, or wall-clock
time as a replay input.
