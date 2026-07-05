# Soak Scenario Runbook

Date: 2026-07-03
Slice: SOAK-01
Status: definition runbook

## Purpose

This runbook explains how to treat the SOAK-01 scenario catalog before real
soak execution exists. It is a preparation document, not a command to run a
long session today.

Operator Go/No-Go language for actual long-running soak execution lives in
`docs/runbooks/soak-go-no-go.md`.

## Safe Preparation

Without fresh operator Go, agents may:

- edit `tests/soak/soak-scenarios.json`,
- add local/mock metrics emitters in later SOAK slices,
- add dry-run harness commands,
- validate JSON shape,
- update performance history and evidence docs.

Without fresh operator Go, agents must not:

- start live Steam,
- run two-machine live tests,
- persist Steam tickets, private account data, provider output, or secrets,
- publish servers or mutate public network state,
- claim release-candidate stability from local-only evidence.

SOAK-02 emitter surfaces:

- Server: `server/src/soak_metrics.rs`
- Godot: `client/godot/scripts/perf/SoakMetrics.gd`

These emitters only provide stable metric names and local snapshots. They do
not run a soak scenario by themselves.

## Execution Order

1. Start with `soak_15m_foundation_local` once SOAK-02 metrics exist.
2. Move to `soak_60m_baseline` only after the 15 minute local run produces all
   required metric groups.
3. Move to `soak_120m_loss_jitter_reconnect` only after loss/jitter and
   reconnect metrics are emitted without unbounded queues.
4. Treat `soak_live_two_machine_60m` as blocked until explicit fresh operator
   Go names the machines, Steam mode, AppID boundary, and data-retention rules.

Any 15, 60, or 120 minute run also needs the bounded decision record from
`docs/runbooks/soak-go-no-go.md`; contract smokes alone are not enough.

## Stop Rules

Stop and mark the run blocked if:

- required metric groups are missing,
- memory or queue growth appears unbounded,
- reconnect fallback is unclear,
- shutdown is not graceful,
- live Steam or public network mutation would be needed without fresh Go,
- any secret, ticket, private account value, or raw live provider payload would
  be written to disk.

## Handoff Evidence

A future soak handoff should include:

- scenario ID,
- duration actually completed,
- machine label and build ID,
- metrics artifact path,
- performance history row path,
- whether each required metric group was present,
- budget result,
- why-changed note,
- open gates.
