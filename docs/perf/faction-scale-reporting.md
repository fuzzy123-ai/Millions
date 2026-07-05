# Faction Scale Reporting

Date: 2026-07-03
Slice: GLOAD-01
Status: reporting contract and catalog surface for provisional faction-scale scenarios

## Purpose

This reporting contract gives future GLOAD and gameplay slices stable scenario
IDs and report fields for faction-scale evidence. It extends the baseline
format in `docs/perf/performance-baseline.md` without changing provisional
budgets.

## Catalog Surface

GLOAD-02 adds the repo-only catalog surface:

- `server/src/faction_scale.rs`
- `tests/perf/faction-scale-scenarios.json`

The Rust catalog exposes the same scenario IDs, entity counts, provisional
role-mix counts, visible entity targets, aggregate far-state requirement, and
budget reference family as this document. It does not run a perf harness,
execute gameplay, or close a scale claim.

## Scenario IDs

| Scenario ID | Stage | Intended first harness owner | Initial status |
| --- | --- | --- | --- |
| `gload_stage_a_2p_neutral_zombie_1k` | A | GLOAD-02 / GCORE / GSWARM | blocked until harness exists |
| `gload_stage_b_ai_pressure_2k` | B | GLOAD-02 / SIM / INT | blocked until synthetic pressure harness exists |
| `gload_stage_c_4plus_mixed_5k` | C | GLOAD-02 / GROLE / INT | blocked until role mix and AOI report exist |
| `gload_stage_d_10k_aoi_lod` | D | GSWARM / LOADSHED / SOAK | blocked until AOI/LOD/backpressure report exists |

## Required Fields

Faction-scale rows must include every field required by the baseline perf
format plus:

- `faction_count`
- `player_faction_count`
- `neutral_system_count`
- `zombie_entity_count`
- `ai_pressure_group_count`
- `simulated_extra_faction_count`
- `entities_per_faction`
- `visible_entities_per_client`
- `aggregate_far_state_count`
- `backpressure_events`
- `deterministic_replay_status`
- `map_data_version_checksum`
- `gate_status`

## Claim Rules

- A row is `blocked` when a required gate or harness does not exist yet.
- A row is `informational` when it records early load shape without claiming a
  budget pass.
- A row is `pass` only when required metrics are present and all relevant
  provisional budgets pass.
- A row cannot close `G-FACTION-COUNT`; that gate needs a later design decision.
- A row cannot claim final gameplay scale unless the touched gameplay roadmaps
  also have implementation, tests, documentation, and evidence.

## Minimal Row Extension

```json
{
  "scenario_id": "gload_stage_a_2p_neutral_zombie_1k",
  "faction_count": 4,
  "player_faction_count": 2,
  "neutral_system_count": 8,
  "zombie_entity_count": 250,
  "ai_pressure_group_count": 0,
  "simulated_extra_faction_count": 0,
  "entities_per_faction": {"player_a": 375, "player_b": 375, "zombie": 250},
  "aggregate_far_state_count": 0,
  "deterministic_replay_status": "pass|fail|blocked",
  "map_data_version_checksum": "blocked-until-mapdata",
  "gate_status": "blocked-by-G-FACTION-COUNT-for-final-claim"
}
```

The example is a schema example only. It is not a measured result.
