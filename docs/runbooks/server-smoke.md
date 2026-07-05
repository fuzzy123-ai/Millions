# Server Smoke

Date: 2026-07-03
Status: slice SRV-03 local smoke

## Purpose

The server smoke verifies that the Rust server crate can start, advance the
deterministic foundation tick loop, build one abstract authoritative snapshot,
and emit a stable log line. It does not open sockets, call Steam, load gameplay,
or mutate external state.

## Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_server_smoke.ps1
```

Equivalent direct command:

```powershell
C:\Users\nkatz\.cargo\bin\cargo.exe run -p millions-server -- --smoke
```

## Expected Log Shape

```text
event=server_smoke status=ok protocol_version=0 tick_hz=20 tick=3 snapshot_id=1 entities=1 x_mm=100 y_mm=50
```

Fields:

- `event`: stable event name.
- `status`: `ok` when the local smoke completes.
- `protocol_version`: protocol version exposed by the crate.
- `tick_hz`: authoritative server tick target.
- `tick`: deterministic tick reached by the smoke.
- `snapshot_id`: local smoke snapshot id.
- `entities`: abstract authoritative entity count in the snapshot.
- `x_mm`, `y_mm`: movement-stub result in protocol millimeter units.

## Stop Rules

Stop and gate the smoke if it would need live network, Steam, secrets, external
provider state, gameplay rules, or destructive Git.
