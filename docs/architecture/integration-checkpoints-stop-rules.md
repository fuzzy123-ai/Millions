# Integration Checkpoints And Stop Rules

Date: 2026-07-03
Slice: `ARCH-03`
Status: repo-only integration contract

## Purpose

This document defines the checkpoints that must be true before Foundation lanes
are integrated across backend, protocol, Godot, Steam preparation, evidence, and
future gameplay. It keeps safe parallel work moving while live, design, release,
and gameplay gates remain explicit.

## Checkpoint Ladder

| Checkpoint | Required evidence | May proceed | Stop or gate if |
| --- | --- | --- | --- |
| `C0_plan_queue` | `docs/plans/millions-plan.json` validates and names the current slice. | Select the next `repo_only` or `safe_offline` slice. | Slice is not in the JSON plan, dependency is missing, or a Markdown planning file would be created under `docs/plans/`. |
| `C1_toolchain` | `scripts/check_environment.ps1` has no blocking toolchain gate, or missing tools are documented. | Local Rust/Godot checks that are already installed. | Installer, download, PATH mutation, or live environment change would be required without fresh Go. |
| `C2_protocol_fixture` | Rust and Godot fixture expectations are aligned by `docs/protocol/cross-side-fixture-validation-plan.md`. | Protocol, codec, command queue, snapshot buffer, and local harness work. | A fixture changes without binary, descriptor, matrix, Rust/Godot expectation, privacy check, and JSON handoff. |
| `C3_authority_boundary` | `docs/architecture/authority-boundaries.md` and `docs/architecture/module-boundaries.md` still hold. | Server-owned logic, Godot adapter/render surfaces, mock Steam preparation. | Godot, transport, Steam facade, logs, or fixtures become durable match authority. |
| `C4_hardening_backpressure` | LOADSHED and HARDEN handoffs are done with local harness/evidence. | Local hostile-input, overload, reconnect, and reliability checks. | Public exposure, live security coverage, raw packet diagnostics, or RC hardening is claimed from local evidence. |
| `C5_performance_evidence` | Budget, perf, soak, and history artifacts name hardware/scenario/percentiles. | Scale and stability claims for local scenarios only. | A 5k/10k, soak, or release claim lacks matching evidence rows and gate status. |
| `C6_live_release` | Operator gives fresh bounded Go for the exact live action. | Real Steam, real AppID, two-machine, public-network, deploy, or release-candidate validation. | Go is broad, stale, missing, or would persist secrets/provider output. |

## Parallel Lane Rules

Backend/protocol, Godot interface, Steam preparation, and evidence/build lanes
may run in parallel only when:

- the JSON plan names the slice and dependencies are satisfied,
- allowed paths are disjoint or the integrating agent owns the handoff,
- the slice class is `repo_only` or `safe_offline`,
- no worker marks a slice done without implementation or docs/evidence,
- checks are either run or explicitly deferred with the reason and gate,
- the JSON handoff records next safe slice and open gates.

## Integration Done Rule

A Foundation integration slice is done only when all of these are true:

- implementation or docs match the slice objective,
- focused checks passed or were explicitly gated,
- documentation/evidence is updated in the appropriate non-plan location,
- `docs/plans/millions-plan.json` records the handoff,
- `scripts/validate_plans.ps1` passes,
- `scripts/check_foundation.ps1` passes,
- open live, design, release, gameplay, and toolchain gates are named.

## Stop Rules

Stop the current path and record a gate if work would:

- add gameplay mechanics before the infrastructure contract says they are safe,
- make Godot, transport, Steam, logs, or fixtures authoritative for match state,
- use real Steam auth/session tickets, real AppID assumptions, provider data, or
  public endpoints without fresh Go,
- persist secrets, tokens, raw provider output, private account data, or raw
  packet diagnostics without a future explicit diagnostic gate,
- claim scale, soak, live security, public exposure, or release-candidate status
  from local-only evidence,
- choose final ECS, transport, rendering, movement, art, faction scale, or
  gameplay balance without the relevant design gate,
- create or edit Markdown planning files under `docs/plans/`,
- use destructive Git or revert unrelated worktree changes.

## Next-Slice Selection Rule

When a lane finishes, choose the next slice in this order:

1. The just-recorded `next_recommended_slice`.
2. The relevant gate's `next_safe_slice`.
3. The earliest incomplete `repo_only` or `safe_offline` infrastructure slice
   whose dependencies are satisfied.
4. If only live/design/gameplay slices remain, record the gate and stop with a
   handoff report.

Do not jump into gameplay merely because the gameplay slice is marked
`repo_only`; infrastructure gates, authority boundaries, map-data authority,
movement-scale evidence, and performance evidence still apply.
