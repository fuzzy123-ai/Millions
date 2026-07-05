# Performance History Ledger

Date: 2026-07-03
Slice: PHIST-04
Status: ledger schema, emitters, regression thresholds, why-changed evidence, and changelog

## Purpose

The performance history ledger tracks comparable performance rows over time.
It preserves enough context to answer what changed, which scenario changed,
which budget was affected, and whether the row is evidence or only a contract
sample.

The ledger does not replace focused perf reports. It indexes and normalizes
their rows so later regression tooling can compare them.

## Ledger Row Shape

Each row should be representable as one JSON object:

```json
{
  "schema_version": 1,
  "ledger_id": "local-dev-01__sim_1k_single_client__2026-07-03__local-uncommitted",
  "date": "2026-07-03",
  "machine_label": "local-dev-01",
  "build_id": "local-uncommitted",
  "source_slice": "PHIST-01",
  "scenario_id": "sim_1k_single_client",
  "scenario_family": "simulation_scale",
  "status": "informational",
  "budget_result": "blocked",
  "budget_keys": ["sim_1k_p95_ms_max"],
  "metrics": {},
  "source_artifact": "tests/perf/sim-scale-local-report.json",
  "why_changed": "contract sample only",
  "claim_scope": "informational_contract_only",
  "redaction_status": "pass",
  "notes": ""
}
```

## Required Fields

- `schema_version`
- `ledger_id`
- `date`
- `machine_label`
- `build_id`
- `source_slice`
- `scenario_id`
- `scenario_family`
- `status`
- `budget_result`
- `budget_keys`
- `metrics`
- `source_artifact`
- `why_changed`
- `claim_scope`
- `redaction_status`
- `notes`

## Scenario Families

| Family | Scenario examples | First source |
| --- | --- | --- |
| `server_tick` | `server_idle_20hz` | `docs/perf/performance-baseline.md` |
| `protocol_fixture` | `protocol_fixture_decode` | `docs/perf/performance-baseline.md` |
| `simulation_scale` | `sim_1k_single_client`, `sim_5k_single_client`, `sim_10k_single_client` | `tests/perf/sim-scale-scenarios.json` |
| `swarm_collision` | `swarm_1k_server_behavior_collision_load` | `tests/perf/swarm-load-smoke-report.json` |
| `swarm_batch_vs_single` | `swarm_batch_vs_single_movement_loop_measurement` | `tests/perf/swarm-batch-vs-single-movement-loop-report.json` |
| `interest_bandwidth` | `int_aoi_delta_steady_128_visible`, `int_aoi_delta_churn_256_visible_32_removed`, `int_aoi_10k_aggregate_far_state` | `tests/perf/interest-bandwidth-smoke-report.json` |
| `godot_render` | `godot_render_1k_visible` | `tests/perf/render-stress-smoke-report.json` |
| `local_multiclient` | `local_two_client_loop` | planned local smoke evidence |
| `reconnect` | `reconnect_full_snapshot_1k` | planned reconnect report |
| `loss_jitter` | `loss_jitter_command_ack` | planned loss/jitter report |
| `soak` | `soak_60m_baseline` | planned soak report |
| `faction_scale` | `gload_stage_a_2p_neutral_zombie_1k`, `gload_stage_b_ai_pressure_2k`, `gload_stage_c_4plus_mixed_5k`, `gload_stage_d_10k_aoi_lod` | `tests/perf/faction-scale-scenarios.json` |

## Naming Rules

Scenario IDs are lowercase snake_case and should follow:

```text
{family}_{subject}_{scale_or_condition}_{qualifier}
```

Use stable nouns, not implementation names. Prefer:

- `sim_10k_single_client`, not `soa_vec_test_10000`.
- `int_aoi_10k_aggregate_far_state`, not `grid_bandwidth_trial`.
- `gload_stage_c_4plus_mixed_5k`, not `faction_experiment_big`.

Ledger IDs should be deterministic enough for local comparison:

```text
{machine_label}__{scenario_id}__{date}__{build_id}
```

If two rows share the same ledger ID, a later evidence export must either
replace the earlier local row intentionally or add a suffix such as
`__rerun_02`.

## Status And Claim Rules

- `pass`: all required fields are present and all relevant budgets passed.
- `fail`: the row attempted a claim and a required check or budget failed.
- `blocked`: a required metric, budget key, gate, or artifact is missing.
- `informational`: the row is useful context but does not close a gate.

`claim_scope` must be explicit. Examples:

- `informational_contract_only`
- `local_regression_signal`
- `budget_evidence`
- `release_candidate_evidence`

Rows cannot close scale, soak, release, gameplay, live Steam, or security claims
unless their source artifact and gates support that claim.

## Redaction Rules

Ledger rows must not contain Steam tickets, provider tokens, secrets, private
account data, raw provider payloads, raw unapproved packet dumps, or private
machine paths. Use repo-relative artifact paths and local/mock identifiers.

## First Sample

The initial machine-readable sample lives at:

```text
tests/perf/performance-history-ledger-sample.json
```

It is a schema sample and does not record a measured result.

## Emitters

PHIST-02 adds first emitter surfaces for repo-only harnesses:

- Server: `server/src/perf_history.rs`
- Godot: `client/godot/scripts/perf/PerfHistoryRow.gd`
- Server sample: `tests/perf/perf-history-server-row-sample.json`
- Swarm/collision sample: `tests/perf/perf-history-swarm-row-sample.json`
- Swarm batch-vs-single sample: `tests/perf/perf-history-swarm-batch-vs-single-row-sample.json`
- Godot sample: `tests/perf/perf-history-godot-row-sample.json`

The emitters create machine-readable ledger rows with deterministic
`ledger_id` values and all required fields. Their default rows are
`status: "informational"` and `budget_result: "blocked"` until a harness
provides measured timing, frame, bandwidth, memory, and threshold evidence.
That keeps them useful for automation without accidentally closing performance,
scale, soak, release, gameplay, live Steam, or final Godot rendering claims.

Swarm/collision rows may include `metrics.local_elapsed_us` stage counters from
the local swarm load smoke. These counters support local trend comparison for
spawn, behavior, flow-field preview, opt-in movement, configured movement
ticks/loop, snapshots, and collision diagnostics, but they do not satisfy
measured p95 server-tick, live-bandwidth, memory, soak, or release gates.

## Regression Thresholds

PHIST-03 adds the first ledger-wide comparison contract:

```text
docs/perf/performance-history-regression-thresholds.json
```

The file normalizes the existing local simulation-scale and interest-bandwidth
threshold artifacts into ledger scenario IDs. It also includes a swarm/collision
local regression row for the 1,000-zombie load smoke's elapsed-stage counters,
snapshot bytes, and estimated local bandwidth. It includes a swarm
batch-vs-single comparison row for local single-loop p95, batch-loop p95, and
batch/single p95 ratio in basis points. The file reserves the Godot render
threshold keys from the provisional budgets, but marks that row blocked until
measured p95 frame, render, decode, and apply metrics exist.

Rows are comparable only when their `machine_label`, `scenario_id`, and
`scenario_family` match. Missing metrics, mismatched budget keys, or absent
`why_changed` evidence must block the regression claim instead of passing it.
Swarm/collision and swarm batch-vs-single comparisons remain local-smoke
comparisons only; they do not replace measured p95 server tick, memory, soak,
live transport, or release evidence.

GSWARM-12 adds a promotion guard for the swarm batch-vs-single comparison:

- `scripts/run_swarm_batch_vs_single_promotion_check.ps1`
- `tests/perf/swarm-batch-vs-single-promotion-report.json`

The guard requires at least three comparable, redacted
`swarm_batch_vs_single_movement_loop_measurement` rows before the comparison can
be treated as a local budget-candidate signal. A single row remains blocked even
when its local p95 ratio and elapsed-time metrics are under the provisional
thresholds. This prevents one-off local noise from being promoted into a budget,
release, live transport, or final swarm-physics claim.

## Why-Changed Evidence

PHIST-03 starts the machine-readable change ledger at:

```text
docs/evidence/performance-history-why-changed-evidence.json
```

A `why_changed` entry is required whenever a ledger row changes a metric,
`status`, `budget_result`, `claim_scope`, `budget_keys`, `source_artifact`, or
when a threshold changes. The first entry records the threshold contract itself
as `new_baseline` with `budget_action: "no_budget_change"`.

The accepted reason codes are:

- `new_baseline`
- `implementation_change`
- `harness_change`
- `toolchain_change`
- `budget_change`
- `measurement_noise`
- `gate_unblocked`
- `contract_only`

Why-changed evidence explains movement; it does not override a failed or
blocked metric.

## Human-Readable Changelog

PHIST-04 adds:

```text
docs/perf/performance-history-changelog.md
```

The changelog records the Foundation performance timeline, current claim map,
and update rule for future performance rows. It is the place to explain
operator-facing performance movement before a future handoff promotes any row
from contract-only evidence to local regression, budget, release, soak, or live
evidence.
