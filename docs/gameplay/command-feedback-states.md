# Command Feedback States

Date: 2026-07-05
Slice: `GCTRL-01`
Status: repo-only command feedback contract

## Purpose

This contract defines the command feedback states that Godot may present during
future RTS control work. It keeps selection, targeting, pending command display,
and rejection language testable while preserving the server as the only source
of durable match truth.

GCTRL-01 does not implement UI, input handling, cursors, selection overlays,
combat, cover, pathfinding, final color language, final accessibility behavior,
or gameplay balance.

## Authority Rule

Godot may show what the local player is trying to do. The Rust server decides
whether the command is accepted, rejected, stale, corrected, or applied to
authoritative state.

Client feedback must always be replaceable by later authoritative server data.
If a pending or preview state disagrees with a server ack, rejection, or
snapshot, the server result wins.

## Feedback State Model

| State | Meaning | Authority |
| --- | --- | --- |
| `selection_local` | The local player has selected one or more visible proxies. | Godot presentation only |
| `command_context_ready` | A command type and local target/context can be formed. | Godot presentation only |
| `command_intent_queued` | The adapter has queued an intent before send or resend. | Godot/adapter queue |
| `command_intent_sent` | The intent has been handed to the connection layer. | Godot/transport evidence |
| `command_acknowledged` | The server acknowledged receipt or idempotent duplicate handling. | Server |
| `command_rejected` | The server rejected the intent with a bounded reason code. | Server |
| `command_stale` | The intent is no longer relevant because it was superseded, timed out, or contradicted by newer server state. | Server or adapter reconciliation |
| `command_snapshot_reconciled` | Authoritative snapshot state corrected local feedback or prediction. | Server snapshot |
| `command_feedback_degraded` | Local diagnostics indicate feedback may be incomplete or visually unclear. | Diagnostic only |

These names are stable enough for future adapter tests, logs, and local
headless checks. They are not final UI labels, icons, colors, animations, or
sound cues.

## Command Context Shape

Future GCTRL slices may pass a bounded command context dictionary through the
client adapter:

```gdscript
{
	"command_id": 1,
	"client_seq": 1,
	"command_type": 3,
	"selected_entity_ids": [1001, 1002],
	"target": {
		"kind": "world_position",
		"x_mm": 1000,
		"y_mm": -2000
	},
	"target_tick": 0,
	"feedback_state": "command_intent_queued"
}
```

Allowed target kinds for local preparation are:

- `none`
- `world_position`
- `entity_id`
- `map_marker_id`
- `cover_candidate_id`

The shape must not contain raw packet bytes, Godot node paths, Resource paths,
Steam tickets, provider tokens, private account data, or client-computed
gameplay outcomes.

## Server-Owned Results

The server owns:

- command acceptance and rejection,
- bounded rejection reason codes,
- idempotent duplicate handling by `command_id` and `player_session_id`,
- authoritative entity ownership and target validity,
- combat, cover, movement, capture, economy, spawn, and production outcomes,
- snapshot correction and reconcile timing.

Future client code may display reason codes such as `wrong_owner`,
`out_of_bounds`, `invalid_target`, `stale_command`, `rate_limited`, or
`server_busy` only after those codes are supplied or mirrored by server-owned
contracts. Godot must not invent success.

## Presentation Rules

- Pending feedback must look temporary and be easy to replace with server state.
- Rejection feedback must remain visible long enough for local diagnosis, but
  must not mutate authoritative world state.
- Selection feedback may be local, but command legality must wait on server
  validation when legality depends on ownership, range, cover, map data, or
  current simulation state.
- Command feedback may reference render proxies by authoritative entity ID, not
  by scene path.
- Diagnostics may say feedback is degraded, but degraded feedback is not a
  gameplay result.

## Evidence Before Claim

A future command-feedback implementation claim needs all of:

- Godot check-only coverage for queued, sent, acked, rejected, stale, and
  reconciled states,
- server or protocol evidence for any new ack/reject reason,
- local multi-client or GCORE smoke coverage for the touched command path,
- performance/readability evidence if overlays or many-proxy feedback are
  touched,
- JSON handoff listing open live, design, gameplay, and release gates.

## Stop Rules

Stop and gate a future change if it would:

- let Godot decide command success or rejection,
- hide a server rejection or authoritative snapshot correction,
- persist raw packet bytes, secrets, Steam tickets, provider data, or private
  account information,
- encode command truth in scene paths, node names, or local Resources,
- choose final cursor language, iconography, color semantics, animation, sound,
  accessibility behavior, or render technology without design Go,
- claim combat, cover, movement, economy, balance, live Steam, public network,
  two-machine, or release-candidate readiness from this local contract.

