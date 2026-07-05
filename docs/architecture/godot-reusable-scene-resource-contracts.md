# Godot Reusable Scene And Resource Contracts

Date: 2026-07-03
Slice: GSCENE-03
Status: reusable scene/resource contract

## Purpose

Future Godot work needs reusable scenes and Resources that are visible in the
editor, predictable in tests, and clearly separated from server authority. This
contract defines when to create a `.tscn`, when to create a `.tres` or Resource,
and how reusable Godot assets interact with protocol, snapshots, Steam facade
state, and render adapters.

This document does not create gameplay mechanics, final UI design, final render
technology, final art, or server authority inside Godot.

## Reusable Scene Rules

Create a reusable `.tscn` when the object has fixed node structure, inspector
properties, signals, groups, child containers, or editor-visible layout.

Examples that should be scenes:

- lobby screen components and ready panels,
- match root, world root, camera rig, HUD layers, and debug overlay panels,
- render proxy hosts and batch host containers,
- authored map markers such as spawn markers, capture points, cover objects,
  obstacles, and editor-only helper anchors,
- dev/test scenes that exercise adapter, render, snapshot, or perf behavior.

Reusable scenes must define:

- root node type and root name,
- fixed subnodes and ownership boundaries,
- script owner, if any,
- exported references and Resources,
- signal and group names,
- dynamic child hosts, if runtime children are allowed,
- check-only test or docs-only reason.

## Resource Rules

Create a Resource or `.tres` when the data is editor-tweakable, reusable across
scenes, and not authoritative match state.

Allowed Resource categories:

| Resource area | Folder | Owns | Must not own |
| --- | --- | --- | --- |
| Player metadata | `client/godot/resources/players/` | local display color/name defaults, placeholder profiles | real identity, Steam ticket, server session authority |
| Protocol fixture metadata | `client/godot/resources/protocol/` | fixture labels, local decode expectations, debug display options | raw secrets, packet authority, Godot-specific wire truth |
| Render style | `client/godot/resources/render/` | placeholder colors, proxy sizes, batch style hints, selection visuals | final art approval, simulation outcomes |
| UI theme/config | `client/godot/resources/ui/` | local layout/theme values and copy-independent config | command success, match state, private data |

Resources may be used by scripts and scenes, but they must remain inputs to
presentation or local tooling. A Resource must not decide command acceptance,
entity truth, combat, capture, economy, spawn, replay determinism, Steam auth,
or release evidence.

## Naming Rules

- Scene filenames use snake_case and describe the reusable thing:
  `ready_panel.tscn`, `render_proxy_host.tscn`, `debug_overlay.tscn`.
- Resource filenames use snake_case and include the category when useful:
  `default_player_profile.tres`, `placeholder_render_style.tres`.
- Script filenames use PascalCase when they define a system-like class already
  following repo style: `CommandQueue.gd`, `RenderAdapter.gd`.
- Test scripts use snake_case plus `_check.gd`.

## Runtime Instancing Rules

Runtime instancing is allowed only through a documented host:

- `RenderProxyHost` for render proxies,
- `VfxHost` for temporary effects,
- `ProjectileHost` for projectile visuals,
- debug host nodes for temporary diagnostics,
- test scene roots for check-only helpers.

The instanced scene or Resource must not store server-authoritative truth. If a
snapshot corrects local presentation, the snapshot wins.

## Protocol And Authority Boundary

Reusable scenes and Resources may display protocol-derived facts after the
approved adapter decodes and applies them. They must not:

- parse raw packet bytes outside `scripts/net/`,
- put scene paths, Node names, or Resource paths into protocol payloads,
- accept or reject commands,
- compute durable gameplay outcomes,
- persist Steam tickets, real AppID assumptions, secrets, or private account
  data.

## Checklist For New Reusable Assets

Before adding a reusable scene or Resource, record:

- owning slice ID,
- folder and filename,
- root node type or Resource category,
- owning script or no-script reason,
- exported fields and inspector-tweakable values,
- signals/groups or no-signal reason,
- runtime instancing host or `none`,
- adapter boundary touched,
- performance impact,
- check command or docs-only reason.

## GSCENE-03 Check Surface

The current check-only surface is
`client/godot/scripts/tests/scene_resource_contract_check.gd`. It validates the
contract document and the expected reusable scene/resource folder anchors. Later
slices may extend it when actual `.tscn` or `.tres` reusable assets are added.
