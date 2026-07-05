# Godot Client Adapter Contract

Date: 2026-07-03
Status: slice CLNT-01 client adapter and UI ownership contract

## Purpose

The Godot client adapter converts local UI/input intent into protocol-ready
command dictionaries and converts authoritative server snapshots into
render-ready client state. It does not own match authority.

## Ownership Boundary

| Component | Owns | Does not own |
| --- | --- | --- |
| UI scenes and controls | Button state, local selection presentation, user intent collection | Command success, entity truth, match outcomes |
| `CommandQueue.gd` | Pending local intent dictionaries and client sequence numbers | Server acceptance, idempotent apply, gameplay effects |
| `ServerConnection.gd` | Connection lifecycle and transport handoff | Protocol payload semantics, simulation authority |
| `ProtocolCodec.gd` | Fixed wire field decode/encode helpers | Scene creation, UI mutation, gameplay decisions |
| `SnapshotBuffer.gd` | Ordered server snapshot dictionaries | Entity mutation outside buffer ownership |
| `ClientWorldState.gd` | Client mirror of authoritative entity facts | Server-side truth or prediction-as-truth |
| `RenderAdapter.gd` | Render records for visual proxy systems | Packet parsing, command handling, authority |
| `MainThreadGate.gd` | Future async-to-main-thread dictionary handoff | Worker-thread scene mutation |
| `ClientAdapter.gd` | Orchestration between queue, snapshot buffer, world state, and render adapter | Authority, packet parsing details, UI ownership |

`ClientAdapter.gd` exists to wire local checks and future client slices together.
It may queue ready/intent dictionaries and apply authoritative snapshot
dictionaries through the approved components. It must preserve the component
boundaries above.

## UI Rules

- UI may request an intent by calling adapter/queue methods.
- UI may show pending, accepted, rejected, or stale states only when those states
  are derived from adapter data.
- UI must not directly mutate `ClientWorldState`.
- UI must not read raw packet bytes.
- UI must not store Steam tickets, secrets, provider tokens, or private account
  data.
- UI copy/join/ready flow remains testable through local/mock session state
  until live Steam gates open.

## Adapter Data Shapes

Command intent:

```gdscript
{
	"client_seq": 1,
	"command_type": 1,
	"payload": {},
	"target_tick": 0
}
```

Snapshot dictionary:

```gdscript
{
	"server_seq": 2,
	"tick": 20,
	"message_type": 5,
	"payload": {
		"entities": [],
		"removed_entities": []
	}
}
```

Render record:

```gdscript
{
	"entity_id": 1,
	"entity_kind": 1,
	"faction_id": 1,
	"x_mm": 1000,
	"y_mm": -2000,
	"health_q8": 256
}
```

## Required Signals Later

Later UI slices should use signals or explicit method calls for:

- connection state changed,
- pending command count changed,
- authoritative entity count changed,
- snapshot tick advanced,
- command ack/reject received,
- reconnect started/restored,
- packet loss or stale snapshot warning.

## Stop Rules

Stop and gate the slice if a change would:

- let UI directly accept/reject commands,
- make prediction override an authoritative snapshot,
- add real gameplay mechanics before infrastructure gates,
- hide raw packet bytes inside UI controls,
- require live Steam, real AppID, two-machine smoke, or final render-tech choice.
