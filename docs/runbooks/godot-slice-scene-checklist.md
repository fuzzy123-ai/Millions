# Godot Slice Scene Checklist

Date: 2026-07-03
Slice: GSCENE-02
Status: required checklist for future Godot slices

## Purpose

Every Godot-facing slice must preserve the scene-first, node-first contract in
`docs/architecture/godot-scene-node-contract.md`. Use this checklist before a
slice starts, while reviewing changes, and before marking the slice done.

This checklist applies to scene files, scripts, resources, runbooks, tests, and
adapter work under `client/godot/`.

## Pre-Slice Checklist

| Item | Required answer |
| --- | --- |
| Slice ID | JSON plan slice being executed |
| Slice class | `repo_only` or `safe_offline` unless an explicit gate says otherwise |
| Allowed paths | Exact paths from `docs/plans/millions-plan.json` |
| Scene path | Existing or planned `.tscn` path, or `none` with reason |
| Root node | Root node type and name |
| Fixed subnodes | Editor-visible node tree to be created or preserved |
| Dynamic nodes | Runtime `add_child()` exceptions and owning host node |
| Scripts | Script paths and which node owns each script |
| Resources | Resource paths, exported references, and inspector-tweakable values |
| Signals/groups | Signal names, group names, and connection owner |
| Adapter boundary | Which of CommandQueue, SnapshotBuffer, ClientWorldState, RenderAdapter, ServerConnection, ProtocolCodec, or SteamLobbyFacade is touched |
| Authority boundary | Server-owned truth that Godot must not decide |
| Performance impact | expected entity count, visible count, node count, render proxy count, and allocation risk |
| Logs/evidence | redacted log fields, perf fields, report paths, and check command |

## Scene-First Rules

- Fixed UI, lobby, match, map, camera, debug overlay, and reusable gameplay
  structure must be visible in `.tscn` scene files.
- Scripts may wire behavior, but must not hide fixed scene structure.
- High-count runtime entities must use `RenderProxyHost`, batching, pooling, or
  another documented runtime host.
- Dynamic `add_child()` calls must name their host and explain why the child is
  runtime-only.
- Scene paths, Resource paths, and Node names must not become protocol payload
  authority.

## Adapter Rules

- UI may create intent through adapter methods, not direct protocol mutation.
- `CommandQueue.gd` owns pending local intent dictionaries only.
- `ProtocolCodec.gd` owns fixed wire helpers, not scene changes.
- `SnapshotBuffer.gd` and `ClientWorldState.gd` mirror server facts, but do not
  make prediction authoritative.
- `RenderAdapter.gd` consumes render records; it does not parse packets or
  accept/reject commands.
- `SteamLobbyFacade.gd` may prepare mock/local lobby handoff, but must not store
  real Steam tickets or real AppID assumptions.

## Done Checklist

A Godot slice is not done until:

- scene/root/subnode ownership is documented or explicitly unchanged,
- fixed structure is in `.tscn` or the slice explains why no scene file changes
  were needed,
- dynamic nodes are limited to documented runtime hosts,
- Godot remains presentation/input/prediction feedback only,
- server-owned state, command success, gameplay outcomes, and replay authority
  remain outside Godot,
- performance/log/evidence impacts are recorded,
- Godot check-only tests or docs-only justification are recorded,
- `scripts\validate_plans.ps1` and `scripts\check_foundation.ps1` pass after
  any plan update.

## Stop Rules

Stop and gate the slice if it would:

- make Godot authoritative for durable match state,
- implement real gameplay before `G-GAMEPLAY-SCOPE` is resolved,
- create hidden fixed scene structure in scripts,
- store secrets, Steam auth tickets, real AppID assumptions, or private account
  data,
- require live Steam, live networking mutation, final render technology, final
  art, or a broad UI/design decision,
- skip required scene, performance, or evidence documentation.
