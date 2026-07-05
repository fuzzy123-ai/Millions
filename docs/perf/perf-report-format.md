# Performance Report Format

Date: 2026-07-03
Status: slice PERF-01 report schema

## Purpose

Performance reports must be comparable across local runs before any scale,
reconnect, soak, or release-candidate claim is accepted. This format links
scenario results to the provisional budget keys in `docs/plans/millions-plan.json`.

## Files

- `tests/perf/perf-report.schema.json`: dependency-light schema descriptor for
  required fields, enum values, metric groups, and privacy rules.
- `tests/perf/sample-perf-report-row.json`: sample row that is intentionally
  `informational` and `blocked` because it contains no measured values.
- `tests/perf/sim-scale-local-report.json`: SIM-03 local informational report
  for 1k/5k/10k simulation-only scenarios.
- `tests/perf/sim-scale-regression-thresholds.json`: SIM-03 provisional
  regression thresholds linked to the JSON plan budgets.
- `tests/perf/interest-bandwidth-smoke-report.json`: INT-03 local informational
  AOI bandwidth byte-estimate report.
- `tests/perf/interest-bandwidth-regression-thresholds.json`: INT-03
  provisional bandwidth regression thresholds.

## Required Semantics

Each row must include:

- machine label,
- build ID,
- scenario ID,
- status,
- budget result,
- budget keys used for comparison,
- metric groups,
- notes for missing or intentionally changed data.

`budget_result` is:

- `pass` when every required metric is present and within budget,
- `fail` when any required metric exceeds budget,
- `blocked` when a required metric, budget key, machine identity, or build/tool
  identity is missing.

`informational` rows cannot close a budget, scale, reconnect, soak, or release
gate.

## Privacy

Reports must not include secrets, Steam tickets, private account data, provider
tokens, raw live provider output, or private machine paths.

## Current Limit

PERF-01 defines the report format. SIM-03 adds the first local informational
simulation-scale report, and INT-03 adds the first local informational
interest-bandwidth report. Later slices still need richer rows from server,
Godot, local multi-client, reconnect, loss, live transport, and soak harnesses.
