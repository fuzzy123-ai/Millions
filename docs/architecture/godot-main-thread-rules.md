# Godot Main-Thread Rules

Date: 2026-07-03
Status: slice GNET-03 bridge threading contract

## Purpose

Godot scene tree, UI, render adapters, Resource access, and Node mutation must
stay on the main thread. Network receive/decode work may become asynchronous
later, but it must hand immutable dictionaries or byte buffers back to the main
thread before touching Godot objects.

## Rules

- `ProtocolCodec.gd` may decode `PackedByteArray` into dictionaries and should
  stay free of scene tree access.
- `ServerConnection.gd` owns connection lifecycle and may record decoded header
  metadata, but future socket callbacks must dispatch scene/node changes through
  main-thread handoff.
- `CommandQueue.gd`, `SnapshotBuffer.gd`, `ClientWorldState.gd`, and
  `RenderAdapter.gd` are main-thread components.
- `RenderAdapter.gd` is the only bridge component allowed to prepare render
  records for visual nodes.
- Packet bytes must not directly create Nodes, Resources, or scene instances.
- Worker threads, if introduced later, may only produce immutable packet or
  snapshot dictionaries and enqueue them for main-thread application.

## Allowed Off-Thread Later

- UDP receive buffers,
- raw packet copy,
- protocol header decode into plain dictionaries,
- fixture byte comparisons,
- checksum calculations,
- compression/decompression after a future gate defines it.

## Main-Thread Only

- `add_child`, `remove_child`, scene switching, and NodePath lookup,
- Control/UI updates,
- Render proxy creation, pooling, and visibility,
- Resource load/save,
- `PerfLedger`, `ClientLog`, and debug overlay mutation unless a later logging
  queue explicitly serializes the handoff.

## Handoff Shape

Future asynchronous code should hand data to the main thread in this shape:

```gdscript
{
	"kind": "decoded_header|snapshot|ack|disconnect|metric",
	"server_seq": 0,
	"tick": 0,
	"payload": {}
}
```

The receiving main-thread node validates the dictionary before applying it.

## Stop Rules

Stop and gate the slice if a change would:

- mutate the scene tree from a worker thread,
- pass raw packet bytes into render/UI nodes,
- store live Steam tickets or provider data in queued dictionaries,
- let Godot decide authoritative simulation outcomes.
