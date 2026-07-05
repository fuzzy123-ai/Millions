# Cover Combat Authority

Date: 2026-07-05
Slice: `GCOV-01`
Status: repo-only server authority contract

## Purpose

This contract defines the first server-owned obstacle, cover, line-of-sight,
line-of-fire, and cover occupancy model. It turns validated `mapdata_v0`
obstacle and cover shapes into deterministic Rust data structures without
adding damage, hit rolls, unit stats, balance, UI, or final combat rules.

## Server-Owned Data

`server/src/cover.rs` owns the GCOV-01 data model:

- `CoverCombatMap` stores map ID, version, checksum, obstacle volumes, cover
  objects, and per-cover occupancy.
- `AxisAlignedVolume` stores simple rectangular map volumes derived from
  validated map shapes.
- `CoverObject` wraps a cover volume and bounded slot capacity.
- `CoverOccupancy` stores occupied entity IDs in deterministic order.
- `LineQuery` reports whether a server-side line query is clear and which
  obstacle blocks it first in stable map order.

The model imports only after `validate_map_data_import` accepts the map data.
Invalid shape kinds, duplicate IDs, invalid bounds, missing checksums, or
out-of-bounds shapes remain map-data validation failures before combat authority
can be built.

## Queries

GCOV-01 supports these server-owned query surfaces:

- obstacle lookup by point,
- cover lookup by point,
- line-of-sight query,
- line-of-fire query,
- claim and release of bounded cover slots by authoritative `EntityId`.

Line-of-sight and line-of-fire currently share the same obstacle blocker model.
Future slices may separate smoke, suppression, height, stance, weapon class,
faction, stealth, or projectile behavior, but those are not part of GCOV-01.

GCOV-02 adds a range-first targeting query:

- calculate squared distance against a bounded max range,
- return `OutOfRange` before doing obstacle blocker work,
- return `BlockedByObstacle` when line-of-fire crosses an obstacle,
- return `InRangeTargetInCover` when the target position lies inside a cover
  object,
- return `InRangeClear` when range and line-of-fire are clear and the target is
  not in cover.

`InRangeTargetInCover` is a server-side effect classification only. It does not
apply damage reduction, suppression, hit chance, weapon rules, stance, animation,
audio, UI language, or balance.

Player-facing language for these states lives in
`docs/gameplay/cover-combat-feedback-language.md`.

## Grid And Performance Note

The server already has a deterministic entity `SpatialGrid` in
`server/src/simulation.rs`, used by AOI code in `server/src/interest.rs`.
GCOV-01 does not add a second performance grid. Cover and obstacle queries are
linear over the validated shape lists because this slice defines correctness and
authority boundaries first.

If cover/LOS/LOF query counts become scale-sensitive, a later slice should reuse
or mirror the existing spatial-cell semantics for map shapes with stable
ordering, bounded neighbor-cell scans, and regression evidence.

## Non-Claims

GCOV-01 does not implement:

- attack commands,
- targeting legality,
- hit/miss outcomes,
- damage,
- suppression,
- cover bonuses,
- movement/pathfinding around obstacles,
- fog of war,
- final combat balance,
- Godot UI/readability language,
- measured performance claims,
- live networking or release readiness.

GCOV-02 also does not implement attack command admission, cooldowns, projectile
simulation, damage, final cover bonuses, unit roles, or combat balance. It only
adds deterministic targeting classification tests.

GCOV-04 adds a dense local smoke that runs 6,144 deterministic targeting
classifications across 64 attacker positions and 96 target positions. The smoke
requires clear, in-cover, blocked-by-obstacle, and out-of-range buckets, but it
is still informational and blocked until measured p95 query cost exists.

## Stop Rules

Stop and gate a future cover/combat change if it would:

- make Godot scene paths or node names authoritative for cover truth,
- bypass map-data checksum or validation before importing cover state,
- let the client decide line of sight, line of fire, damage, or occupancy,
- claim scale without performance evidence,
- choose final combat balance, role stats, visuals, sounds, or accessibility
  semantics without the relevant design gate,
- require live Steam, public networking, two-machine validation, or
  release-candidate evidence.
