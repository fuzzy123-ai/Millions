# Map Data Change Checklist

Date: 2026-07-03
Slice: `MAPDATA-04`
Status: repo-only map data change runbook

## Purpose

This checklist defines the safe path for changing authored map data fixtures and
future Godot exports. It keeps Godot-authored data, server import validation,
checksum evidence, and JSON plan handoffs aligned before any map data can affect
gameplay authority.

The JSON plan remains the source of truth for slice order and gates. This
runbook is the operator checklist for map-data changes inside the allowed
repo-only lane.

## Safe Change Order

Use this order for any repo-only map data change:

1. Update the authored contract or fixture in `tests/fixtures/`.
2. Recompute the deterministic checksum sidecar for the exact fixture bytes.
3. Update server-side validation or checksum tests in `server/` when the import
   shape changes.
4. Update the Godot headless fixture check when the exported shape changes.
5. Update architecture or protocol docs describing the changed field, rule, or
   non-claim.
6. Run focused Rust and Godot fixture checks.
7. Run plan validation and foundation checks after any JSON plan update.
8. Record evidence in the JSON handoff or a dedicated evidence artifact.

Do not skip the checksum sidecar. A map data change without a matching checksum
and cross-side fixture check is incomplete.

## Parallel Work Rules

The work may run in parallel only when write scopes are disjoint:

- server import or checksum logic: `server/`
- Godot fixture reading or exporter adapter checks: `client/godot/`
- shared fixture data: `tests/fixtures/`
- architecture/runbook wording: `docs/architecture/` and `docs/runbooks/`

Only one worker should edit a shared fixture file at a time. If two workers need
the same fixture, one owns the fixture update and the other waits for the
updated checksum sidecar.

## Required Local Checks

For the current MAPDATA foundation, run:

```powershell
cargo test map_data
```

```powershell
$godotRoot = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe"
$godot = Get-ChildItem -Path $godotRoot -Filter "Godot*_console.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $godot) { $godot = Get-ChildItem -Path $godotRoot -Filter "Godot*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1 }
& $godot.FullName --headless --path client\godot --script res://scripts/tests/map_data_fixture_check.gd
```

After any JSON plan update, run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1
powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1
```

If the local sandbox blocks `cargo` or Godot process startup, rerun the same
local check with the narrow approved escalation and record that the original
blocker was process-launch sandboxing, not a product gate.

## Done Criteria

A map-data change is done only when all are true:

- the fixture or contract change is intentional and repo-local,
- checksum sidecar matches the final fixture bytes,
- Rust validation or checksum tests pass,
- Godot headless fixture check passes when Godot-readable data is affected,
- docs state any new field, validation rule, authority boundary, or non-claim,
- JSON plan handoff records checks, documentation, open gates, and next slice,
- no live, release-candidate, real Steam, secret, or gameplay-authority claim is
  introduced.

## No-Go Boundaries

Stop and record a gate instead of continuing if the change would:

- make cover, obstacle, spawn, capture, movement, or navigation data
  authoritative for gameplay,
- require live Steam, public networking, deploy, release, or two-machine smoke,
- persist secrets, tickets, private account data, or absolute private paths,
- rely on scene paths, node names, or editor-only display labels as server
  authority IDs,
- introduce a real exporter without a matching local smoke check and handoff,
- change gameplay balance or product design instead of the map-data contract.

## Current Non-Claims

MAPDATA-01 through MAPDATA-04 define the authored-data boundary, server import
shape, deterministic local fixture checksum, Rust/Godot fixture parity checks,
and this change checklist. They do not implement final exporter UX, gameplay
collision, cover effects, spawn validity, capture ownership, pathfinding,
movement, live validation, or release-candidate evidence.
