# Release Evidence Artifacts

Date: 2026-07-03
Status: slice BUILD-04 naming contract

## Current State

Release artifact packaging is not started. This document only reserves evidence
names and stop rules so later release slices do not invent incompatible output
names.

## Required Evidence Families

- foundation matrix report,
- protocol fixture report,
- replay/desync report,
- performance report,
- packet loss/jitter report,
- local multi-client report,
- Godot check report,
- release gate report.

Artifact naming is defined in `docs/evidence/artifact-names.md`.

Release-candidate checklist and signoff language are defined in
`docs/release/rc-checklist.md`.

## RC Checklist Mapping

| RC checklist field | Evidence family | Current local source | RC state |
| --- | --- | --- | --- |
| Toolchain check | foundation matrix report | `scripts/check_environment.ps1`, `docs/runbooks/local-toolchain-setup.md` | local evidence only |
| Plan/Foundation checks | foundation matrix report | `scripts/validate_plans.ps1`, `scripts/check_foundation.ps1` | local evidence only |
| Local smoke | local multi-client report, Godot check report | `scripts/run_local_smoke_commands.ps1`, `config/local-smoke-commands.json` | local evidence only |
| Protocol/replay | protocol fixture report, replay/desync report | `protocol/fixtures/`, `scripts/run_replay_smoke.ps1` | local evidence only |
| Performance budgets | performance report | `tests/perf/*.json` | blocked until measured RC-grade pass/fail rows exist |
| Soak/chaos | release gate report | `docs/runbooks/soak-go-no-go.md`, `scripts/run_soak_smoke.ps1`, planned long-run reports | blocked until bounded soak result exists |
| Hardening | release gate report | planned `HARDEN` evidence | blocked |
| Steam auth/AppID/two-machine | release gate report | planned `STEAM-03`/`REL-03` live evidence | blocked until explicit live Go |
| Secrets/redaction audit | release gate report | redaction rules in runbooks and evidence index | local policy only |
| Operator decision | release gate report | `docs/release/rc-checklist.md` signoff record | blocked until explicit operator signoff |

`RC Candidate` requires every row to be pass or explicitly accepted by the
operator. Local-only and blocked rows cannot be promoted to pass automatically.

## Bundle Shape

A future release evidence bundle should contain:

- an artifact manifest using the naming contract,
- final `validate_plans` and `check_foundation` outputs,
- local smoke command output,
- protocol fixture and replay/desync outputs,
- performance report rows with measured values and budget results,
- soak and hardening reports,
- live Steam/AppID/two-machine reports when the live gates are open,
- redaction audit result,
- operator signoff record from `docs/release/rc-checklist.md`.

## Release Gate Rules

An artifact may support release-candidate confidence only when:

- the relevant run completed,
- the artifact name follows the evidence naming contract,
- machine/build/tool identity is present,
- budget/replay/soak/hardening gates for the touched path are satisfied,
- live Steam/two-machine gates are explicitly open where required.

Until then, artifacts are local evidence only.

For soak evidence, `scripts/run_soak_smoke.ps1` may prove the local contract and
emitter surface, but release-candidate stability requires the decision/result
records defined in `docs/runbooks/soak-go-no-go.md`.
