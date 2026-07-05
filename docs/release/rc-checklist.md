# Release Candidate Checklist

Date: 2026-07-03
Status: slice REL-01 checklist contract

## Purpose

This checklist defines the evidence and operator signoff language required
before any Millions build can be called a release candidate. It is repo-only
preparation and does not grant live Steam permission, real AppID assumptions,
deploy permission, public network permission, or release-candidate confidence.

## RC Status Words

| Status | Meaning |
| --- | --- |
| `Not RC` | Required gates or evidence are missing. This is the default. |
| `RC Deferred` | The build may be technically prepared, but a live/design/manual gate is not open. |
| `RC Blocked` | A required check failed or evidence is unsafe/incomplete. |
| `RC Candidate` | All required evidence exists, gates are open, and operator signoff is recorded. |
| `RC Rejected` | Operator or validation explicitly rejects the candidate. |

Only `RC Candidate` may be used after every checklist section below is satisfied.

## Required Preflight Evidence

| Area | Required evidence | Current Foundation status |
| --- | --- | --- |
| Toolchain | `scripts/check_environment.ps1` and pinned toolchain evidence. | Local foundation gate clear; version-string cleanup remains a build follow-up. |
| Plans | `scripts/validate_plans.ps1` passes. | Available. |
| Foundation | `scripts/check_foundation.ps1` passes. | Available. |
| Local smoke | `scripts/run_local_smoke_commands.ps1` passes. | Available as local-only evidence. |
| Protocol | Rust/Godot protocol fixtures agree. | Local fixture checks exist. |
| Determinism | Replay smoke and golden checksums pass. | Local replay smoke exists; real gameplay streams missing. |
| Reconnect/loss | Reconnect and packet-loss evidence pass. | Local models exist; broader live evidence missing. |
| Performance | Reports include measured values, machine label, budgets, and pass/fail. | Informational and blocked rows exist; RC-grade perf evidence missing. |
| Soak/chaos | Long-running stability evidence passes. | Not complete. |
| Hardening | Malformed/stale/replay/abuse tests pass. | Not complete. |
| Steam | Real Steam auth, AppID, and two-machine evidence pass. | Blocked by live gates. |
| Release artifacts | Evidence names follow `docs/evidence/artifact-names.md`. | Naming contract exists; package bundle not complete. |

## Live Gates

These gates must be explicitly open before RC signoff:

- `G-STEAM-AUTH`: real Steam auth/session ticket validation,
- `G-REAL-APPID`: real release/playtest AppID identity,
- `G-STEAM-TWO-MACHINE`: live two-machine Steam smoke,
- any deploy, public network, backup/restore, or external write-smoke gate named
  by the operator for that release attempt.

If any gate is closed, the release state is `RC Deferred` or `Not RC`, not
`RC Candidate`.

## Operator Signoff Record

Use this shape when the operator explicitly evaluates a candidate:

```text
Release candidate:
- Build ID:
- Branch or commit:
- Evidence bundle:
- Toolchain check: pass | fail
- Plan/Foundation checks: pass | fail
- Local smoke: pass | fail
- Protocol/replay: pass | fail
- Performance budgets: pass | fail | blocked
- Soak/chaos: pass | fail | blocked
- Hardening: pass | fail | blocked
- Steam auth/AppID/two-machine: pass | fail | blocked
- Secrets/redaction audit: pass | fail
- Operator decision: RC Candidate | RC Deferred | RC Blocked | RC Rejected
- Notes:
```

Do not infer operator signoff from green tests. The operator decision must be
explicit and dated in the handoff or evidence bundle.

## No-Go Conditions

The candidate is `RC Blocked` or `RC Rejected` if any of these are true:

- secrets, Steam tickets, provider tokens, private account data, raw live
  session output, or private paths are persisted,
- a live gate was bypassed,
- a check failed without a documented fix or explicit deferral,
- performance or soak evidence is informational/blocked but claimed as pass,
- Godot, Steam, UI, transport, fixtures, or docs become authoritative for match
  state,
- release artifacts omit build/tool/machine identity,
- evidence is not reproducible enough to compare with later runs,
- destructive Git or unrelated cleanup was required.

## Current RC Position

Current status: `Not RC`.

Reason: Foundation local evidence exists, but release-candidate confidence is
still missing live Steam/AppID/two-machine evidence, soak stability, hardening
coverage, RC-grade performance budget rows, release artifact bundle wiring, and
explicit operator signoff.

Soak stability requires a bounded decision and result record using
`docs/runbooks/soak-go-no-go.md`. Green `scripts/run_soak_smoke.ps1` output is
contract evidence only; it is not a long-running stability result.
