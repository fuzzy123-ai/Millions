# GCORE Local Playtest Loop

Status: local abstract GCORE loop only.

Use this runbook for the first safe test after NAV readiness is green. It checks
that the server can own HQ/squad/move intent state and that Godot can display a
two-client abstract proxy snapshot. It does not test live Steam, public network,
real pathfinding, combat, economy, final roles, or balance.

## Preflight

Run these from the repo root:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_many_units_navigation_readiness.ps1
C:\Users\nkatz\.cargo\bin\cargo.exe test game_core
powershell -ExecutionPolicy Bypass -File scripts\run_godot_gcore_intent_check.ps1
powershell -ExecutionPolicy Bypass -File scripts\run_godot_gcore_local_match_smoke.ps1
powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1
powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1
```

Expected final status lines:

```text
many_units_navigation_readiness status=ok rust_movement=ok rust_mapdata=ok godot_mapdata=ok godot_readability=ok plan=ok foundation=ok
gcore_intent_check status=ok move=ok spawn=ok authority=server
gcore_local_match_smoke status=ok clients=2 hqs=2 squad_proxies=8 entities=10 render_batches=4 intents=2
Foundation check passed.
```

## What This Proves

- Rust owns the local abstract HQ, player start, one basic squad, and move
  intent validation model.
- Move intents are idempotent by `command_id` per `player_session_id`.
- Wrong-owner and out-of-bounds move attempts reject before state mutation.
- Godot can produce intent-only dictionaries without deciding command success.
- Godot can accept a server-shaped snapshot with two HQ proxies and eight squad
  proxies through `ClientWorldState` and `RenderAdapter`.

## What This Does Not Prove

- No live Steam auth, AppID, ticket, lobby, public-network, or two-machine path.
- No real pathfinding, navigation authority, formation movement, avoidance, or
  collision.
- No combat, cover, economy, production, win/loss rule, AI, zombie swarm, role
  mix, or balance.
- No release-candidate or performance acceptance claim.

## First Manual Test Shape

1. Run the preflight commands.
2. Treat `gcore_local_match_smoke status=ok ...` as the first local green light.
3. If Godot opens later for visual inspection, use only abstract placeholder
   proxies and do not promote visual behavior to gameplay authority.
4. Record any unexpected output as evidence before expanding gameplay.

Stop immediately if a test asks for live credentials, Steam tickets, public
networking, installer/download changes, destructive git commands, or final
gameplay/balance decisions.
