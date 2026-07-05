# Cover Combat Feedback Language

Date: 2026-07-05
Slice: `GCOV-03`
Status: repo-only presentation language contract

## Purpose

This contract defines readable player-facing language for future cover combat
feedback. It maps server-owned targeting and cover classifications to bounded
presentation states without adding UI, icons, colors, animation, sound, combat
balance, or gameplay authority to Godot.

## Server Result To Feedback State

| Server result | Future feedback state | Meaning |
| --- | --- | --- |
| `InRangeClear` | `combat_target_clear` | The target is in range and line-of-fire is clear. |
| `InRangeTargetInCover` | `combat_target_in_cover` | The target is in range and line-of-fire is clear, but the target position is inside server-known cover. |
| `BlockedByObstacle` | `combat_blocked_by_obstacle` | A server-known obstacle blocks line-of-fire. |
| `OutOfRange` | `combat_out_of_range` | The target is outside the command's max range. |
| future hit result | `combat_hit_confirmed` | The server confirmed a hit. Not implemented in GCOV-03. |
| future miss result | `combat_miss_confirmed` | The server confirmed a miss. Not implemented in GCOV-03. |

These state names are stable enough for future adapter tests and logs. They are
not final labels, colors, icons, cursor semantics, animations, sounds, or
accessibility copy.

## Presentation Rules

- Godot may preview a target or cover candidate locally, but confirmed combat
  feedback must come from server-owned results.
- `combat_target_clear` means targeting is clear, not that damage will happen.
- `combat_target_in_cover` means cover is present, not that a numeric bonus has
  been applied.
- `combat_blocked_by_obstacle` should expose that the server blocked line of
  fire, not hide it as a generic failure.
- `combat_out_of_range` should stay distinct from blocker and cover states.
- Future hit/miss feedback must not be displayed until the server has an
  explicit hit/miss result contract.

## Allowed Later Godot Preparation

Future Godot work may prepare:

- local hover/selection presentation,
- pending attack intent state,
- server-result feedback badges or log text,
- reconciliation when a target state changes before server result,
- degraded feedback diagnostics under render/perf stress.

The client must not decide hit, miss, blocked, cover effect, damage, suppression,
cooldown, or target legality.

## Stop Rules

Stop and gate a future change if it would:

- make Godot decide combat success,
- display a hit or miss without a server-owned hit/miss result,
- hide `BlockedByObstacle` or `OutOfRange` as local-only UI uncertainty,
- encode cover/combat truth in scene paths, node names, or local Resources,
- choose final colors, icons, cursor language, animation, audio, accessibility
  copy, or art without design Go,
- claim gameplay balance, measured readability, live networking, two-machine, or
  release-candidate readiness from this language contract.

## Non-Claims

GCOV-03 does not implement combat UI, attack commands, hit/miss, damage,
suppression, cover bonuses, cooldowns, final combat labels, final art, final
accessibility language, measured readability, live networking, or release
readiness.

