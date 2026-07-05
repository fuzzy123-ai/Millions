# GCORE Go/No-Go Checklist

Status: GCORE local abstract baseline.

## Go

GCORE local testing is Go when all of these are true:

- `scripts\run_many_units_navigation_readiness.ps1` reports `status=ok`.
- `cargo test game_core` passes.
- `scripts\run_godot_gcore_intent_check.ps1` reports `status=ok`.
- `scripts\run_godot_gcore_local_match_smoke.ps1` reports `status=ok`.
- `scripts\validate_plans.ps1` and `scripts\check_foundation.ps1` pass.
- The test remains local, abstract, server-authoritative, and credential-free.

## Partial Go

Partial Go is acceptable for diagnosis when Rust GCORE tests pass but Godot
headless checks are blocked by local editor setup. In that case, do not claim a
playtest loop. Record the Godot block and continue only with server-side
repo-only checks.

## No-Go

GCORE local testing is No-Go when any of these happen:

- A command result is decided in Godot instead of the Rust server model.
- Duplicate `command_id` values mutate state more than once.
- Wrong-owner or out-of-bounds move intents mutate state.
- The smoke needs live Steam, credentials, public networking, two machines,
  downloads, installers, deploys, or release packaging.
- A test outcome depends on final pathfinding, combat, economy, roles, art, or
  balance choices.

## Deferred Gates

- Real Steam auth/AppID/session tickets.
- Public network and two-machine validation.
- Final movement/pathfinding/formation/avoidance design.
- Combat, cover, economy, production, win/loss, role mix, AI, and swarm.
- Final render technology and art return.
- Release-candidate and measured performance acceptance.
