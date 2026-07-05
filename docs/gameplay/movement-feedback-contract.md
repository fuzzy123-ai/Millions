# Movement Feedback Contract

Date: 2026-07-03
Slice: `NAV-04`
Status: repo-only player-facing movement feedback contract

## Purpose

This contract defines what player-facing movement feedback may be prepared
before real movement gameplay exists. It keeps Godot feedback useful for future
tests while preserving server authority and avoiding final UI, balance, or
movement model decisions.

## Allowed Feedback Preparation

Godot may later show these feedback states as local presentation only:

- move intent queued locally,
- move intent sent to server,
- move intent acknowledged,
- move intent rejected with a server reason,
- predicted path or destination preview,
- authoritative snapshot correction,
- group/lane readability stress indicators,
- blocked or degraded movement diagnostics when the server exposes them.

Every state must be replaceable by a later authoritative snapshot. Godot may
preview and explain movement, but it must not decide durable position, path,
collision, formation, avoidance, capture, combat, or cover outcomes.

## Server-Owned Truth

The server owns:

- final entity positions,
- accepted movement target,
- rejected movement reason,
- pathing and collision validity,
- map bounds and blocker constraints,
- deterministic command ordering,
- correction snapshots,
- movement overload or degradation policy.

Client-side feedback must keep enough state to be corrected without hiding a
server rejection or authoritative position change.

## Future Feedback Events

Future gameplay slices may introduce these presentation event names without
changing authority:

| Event | Meaning | Authority |
| --- | --- | --- |
| `move_preview_local` | Client shows a non-authoritative path or destination before send. | Godot presentation only |
| `move_intent_sent` | Client queued or sent a movement command intent. | Godot presentation only |
| `move_intent_accepted` | Server accepted intent for future simulation. | Server |
| `move_intent_rejected` | Server rejected intent with bounded reason text/code. | Server |
| `move_snapshot_corrected` | Client prediction was corrected by authoritative snapshot. | Server snapshot |
| `move_readability_degraded` | Local rendering/readability stress says feedback may be unclear. | Diagnostic only |

Event names are placeholders for future adapters and runbooks. NAV-04 does not
add a protocol event, UI widget, or gameplay state machine.

## UX Stop Rules

Stop and gate if a future change would:

- make a preview look like confirmed movement,
- hide server rejection or correction,
- encode movement truth in scene paths, node names, or local-only resources,
- choose final movement UX, cursor language, art, animation, color semantics, or
  accessibility behavior without design Go,
- require live Steam, public networking, two-machine smoke, deploy, or release
  evidence,
- claim movement readability without measured frame/render/readability evidence.

## Required Evidence Before Claim

A future movement-feedback claim needs all of:

- server-side movement scenario or command evidence,
- deterministic replay or correction evidence for changed simulation behavior,
- Godot headless readability/render check,
- performance row with budget status,
- documentation of accepted/rejected/corrected states,
- JSON handoff listing open gates and non-claims.

## Non-Claims

NAV-04 does not implement movement feedback UI, final cursor language,
animation, path rendering, server movement, pathfinding, formations, avoidance,
collision, combat, cover, capture, balance, live networking, or release-candidate
readability evidence.
