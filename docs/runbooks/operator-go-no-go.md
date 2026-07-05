# Operator Go And No-Go Runbook

Date: 2026-07-03
Status: slice OPS-01 operator language

## Purpose

This runbook defines the operator words that future Foundation slices use in
handoffs, gates, and evidence. It keeps safe repo-only implementation moving
while live Steam, real AppID, final design, deploy, secrets, and release claims
remain explicit decisions.

The JSON plan is still the source of truth for slice order and gate status.
This runbook explains how to interpret operator language when a slice reaches a
boundary.

## Decision Words

| Word | Meaning | What Codex may do |
| --- | --- | --- |
| `Go` | The named action is explicitly allowed in this context. | Execute only the bounded action named by the operator or plan. |
| `Safe Go` | The action is `repo_only` or `safe_offline` and already allowed by `operator_go_policy`. | Continue implementation, local checks, docs, fixtures, mock adapters, and plan handoffs. |
| `Partial Go` | Only part of a requested action is allowed. | Execute the allowed sub-scope and record the rest as a gate. |
| `Deferred` | A live, design, release, or manual decision is not available yet. | Do safe preparation, mark the gate, and choose the next safe slice. |
| `Blocked` | Work cannot safely continue in that path until external state changes. | Stop that path, record the blocker, and move only if another safe slice exists. |
| `No-Go` | The action must not run. | Do not perform the action; document why and preserve safe state. |

## Standing Safe Go

The operator has granted standing Safe Go for:

- Rust backend/server scaffolding, protocol parsing, deterministic tests, fake
  transports, local loopback work, and offline harnesses.
- Godot GDScript adapters, protocol codecs, command queues, snapshot buffers,
  client world state, render adapter stubs, local UI wiring needed for tests,
  and headless/check-only validation.
- Steam facade contracts, local/mock Steam adapters, Spacewar-safe placeholders,
  endpoint advertisement models, and dedicated-server handshake interfaces that
  do not call live Steam services.
- Runbooks, evidence templates, local scripts, CI/local matrix work, redacted
  logs, and JSON plan updates.

Safe Go does not mean release readiness. A slice still needs implementation,
checks or explicit check deferral, documentation or evidence, and a JSON handoff
before it is done.

## Always Needs Fresh Go

These actions are never covered by standing Safe Go:

- real Steam API calls,
- real Steam auth/session ticket validation,
- real AppID release or playtest assumptions,
- two-machine live Steam smoke,
- public network mutation,
- deploy, publish, backup, restore, or external write smoke,
- downloads or installers,
- secrets, tokens, private provider data, private account data, or raw live
  session output,
- destructive Git, force-push, reset, or unrelated cleanup,
- final product/design choices for renderer, ECS crate, art return, movement
  model, faction scale, gameplay scope, or balance.

If a slice touches one of these, mark it `Deferred`, `Blocked`, or `No-Go` and
continue only with another safe slice.

## Go Language In Handoffs

Every slice handoff should include:

- `Slice`: exact slice ID from `docs/plans/millions-plan.json`,
- `Status`: `done`, `blocked`, `deferred`, or `failed`,
- `Changed files`: paths touched,
- `Tests/Checks`: exact commands and final result lines,
- `Documentation updated`: runbook, protocol, architecture, perf, evidence, or
  JSON plan artifact,
- `Open gates`: live, design, release, or safety decisions still blocked,
- `Next recommended slice`: next safe slice, or the specific gate if none.

Use `Deferred` when a future live/design action is expected but not needed for
the current safe slice. Use `Blocked` only when the current path cannot make
safe progress. Use `No-Go` when the requested action would violate stop rules.

## Gate Examples

| Gate | Class | Safe preparation | Fresh Go needed for |
| --- | --- | --- | --- |
| `G-STEAM-AUTH` | `needs_live_go` | Facade contracts, mock identity, endpoint metadata, local handoff tests. | Real Steam auth/session ticket validation. |
| `G-REAL-APPID` | `needs_live_go` | Config placeholders and Spacewar-safe docs. | Real release/playtest AppID identity. |
| `G-STEAM-TWO-MACHINE` | `needs_live_go` | Same-workstation local multi-client and mock lobby checks. | Live two-machine Steam smoke. |
| `G-RENDER-TECH` | `needs_design` | Render adapter interfaces, placeholder proxies, stress scenes, copied render batches. | Final render technology decision. |
| `G-ECS-CHOICE` | `needs_design` | Plain Rust data structures and deterministic tests. | Long-term ECS crate lock-in. |
| `G-GAMEPLAY-SCOPE` | `needs_design` | Infrastructure validation scenarios and abstract command contracts. | Real gameplay implementation beyond contracts. |
| `G-SOAK-STABILITY` | `repo_only` until a run is requested, then `needs_live_go` for live/two-machine cases. | Scenario catalogs, metric emitters, dry-run/local contract smoke, redacted runbooks. | Any long-running soak execution, live/two-machine soak, or release-candidate stability signoff. |
| `G-BACKPRESSURE` | `repo_only` until live or release evidence is requested. | Caps, slow-client policy, local overload harness cases, operator-visible state words, and read-only Godot diagnostics. | Live Steam overload, public-network overload, two-machine overload, long-soak overload, release-candidate overload signoff, or final transport/gameplay claims. |
| `G-PROTOCOL-HARDENING` | `repo_only` until public exposure or live security evidence is requested. | Packet limits, local parser/property/fuzz seeds, hostile-input harnesses, and security/privacy logging checklists with redacted diagnostics. | Public-network exposure, broader playtests, real Steam auth validation, raw packet diagnostics, live security coverage, or release-candidate hardening signoff. |

## Local Verification Order

Use this order when a slice does not define stricter checks:

1. `powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1`
2. `powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1`
3. Slice-specific local checks, such as `cargo test`, Godot headless checks, or
   runbook smoke scripts.

If a local check cannot run because a tool is missing or blocked by the
environment, record that as a gate or check deferral. Do not silently skip it.

## Stop Rules

Stop or gate immediately if work would:

- persist secrets, tickets, tokens, private account data, or raw provider data,
- mutate live Steam, public network, external services, deploy targets, backups,
  or release state without explicit fresh Go,
- make Godot, Steam, UI, transport, or fixtures authoritative for match state,
- claim release, scale, soak, live Steam, or performance readiness without
  matching evidence,
- mark a slice done without documentation/evidence,
- create Markdown planning files under `docs/plans/`,
- use destructive Git or revert unrelated user changes.

## Related Runbooks

- `docs/runbooks/local-toolchain-setup.md`
- `docs/runbooks/local-steam-reconnect-loss-test-plan.md`
- `docs/runbooks/local-multiclient-harness.md`
- `docs/runbooks/lobby-local-runbook.md`
- `docs/runbooks/soak-go-no-go.md`
- `docs/runbooks/backpressure-operator-states.md`
- `docs/observability/security-privacy-logging-checklist.md`
- `docs/steam/session-advertisement-contract.md`
