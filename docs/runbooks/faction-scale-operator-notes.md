# Faction Scale Operator Notes

Date: 2026-07-03
Slice: GLOAD-03
Status: operator wording for provisional faction-scale scenarios

## Purpose

These notes give operators and future agents readable names for the GLOAD
scenario IDs. They are intended for logs, reports, runbooks, and handoff cards.
They do not approve final faction count, gameplay balance, final AI design, or
live playtest claims.

## Scenario Names

| Scenario ID | Operator name | One-line description | Default claim state |
| --- | --- | --- | --- |
| `gload_stage_a_2p_neutral_zombie_1k` | Stage A local pressure | Two player factions, neutral systems, and first zombie pressure at 1k abstract entities. | informational or blocked until measured |
| `gload_stage_b_ai_pressure_2k` | Stage B synthetic pressure | Stage A plus synthetic AI pressure groups at 2k abstract entities. | informational or blocked until harness exists |
| `gload_stage_c_4plus_mixed_5k` | Stage C mixed-faction load | Four-plus simulated factions with mixed role composition at 5k abstract entities. | informational or blocked until role/AOI evidence exists |
| `gload_stage_d_10k_aoi_lod` | Stage D 10k aggregate load | Ten-thousand-plus abstract entities with AOI, aggregate far-state, and horde pressure. | informational or blocked until budget evidence exists |

## Status Language

- Use `cataloged` when the scenario exists in docs, Rust catalog, or JSON
  catalog but has not run.
- Use `blocked` when a required gate, harness, or metric is missing.
- Use `informational` when a local run records shape or timing without making a
  budget claim.
- Use `pass` only when the report includes the required metrics and budget
  comparison.
- Use `fail` when a measured report exceeds a budget or omits required fields
  while trying to make a claim.

## Operator-Facing Descriptions

Stage A local pressure:
Validates that the foundation can describe two player factions, neutral
systems, and first horde pressure without turning any of them into final
gameplay rules.

Stage B synthetic pressure:
Adds extra server-owned non-player pressure groups to test ownership and load
shape. It is not final AI behavior.

Stage C mixed-faction load:
Adds simulated extra factions and mixed role counts so AOI, render batching,
and role data can be tested without claiming final playable factions.

Stage D 10k aggregate load:
Pushes the catalog toward 10k entity stress with aggregate far-state and
backpressure expectations. It is a performance and stability target, not a
gameplay promise.

## Stop Rules

Stop or defer if the work would:

- claim final all-factions scale before `G-FACTION-COUNT` is resolved,
- treat synthetic pressure groups as final AI design,
- treat simulated extra factions as final playable factions,
- claim performance without measured report fields and budget comparison,
- require live Steam, real AppID, external playtest, final art, or final
  balance choices,
- make Godot authoritative for faction truth.

## Handoff Sentence Template

Use this shape in future handoffs:

```text
{operator_name}: {status}; scenario_id={scenario_id}; entities={count};
claim={none|informational|budgeted}; gates={open gates}; next={next evidence}.
```
