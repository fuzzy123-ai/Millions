# Godot Server Bridge

Date: 2026-07-03
Status: slice GNET-01 responsibility contract

## Purpose

The Godot server bridge is the client-side boundary between local UI/input and
the authoritative Rust server. It may encode intent, decode server facts, buffer
snapshots, expose render-state dictionaries, and report diagnostics. It must not
decide durable match outcomes.

## Components

`ServerConnection.gd`

- Owns local connection state and future transport handoff.
- Calls `Online` for connection status.
- Logs connection lifecycle through `ClientLog`.
- Must not parse gameplay state directly.

`ProtocolCodec.gd`

- Owns protocol_v0 constants and packet-header decode helpers.
- Decodes only fixed wire fields into dictionaries.
- Rejects bad magic, protocol version, flags, header length, unknown message
  type, and payload length mismatch before handing payloads to later code.
- Must not instantiate scenes, Resources, or Nodes from packet data.

`CommandQueue.gd`

- Owns client intent sequence numbers and pending command dictionaries.
- Queues intent only; it does not mark commands as successful.
- Removes commands only when a server ack slice defines and validates the ack
  behavior.

`SnapshotBuffer.gd`

- Owns received snapshot dictionaries ordered by server sequence and tick.
- Keeps the latest full/delta facts for the local client.
- May discard stale buffered snapshots once later interpolation rules exist.
- Must not mutate Godot render nodes.

`ClientWorldState.gd`

- Owns the client-side mirror of authoritative entity facts after snapshot
  application.
- Reconciles local/predicted presentation toward server snapshots later.
- Must treat server snapshot facts as truth and must not invent authority.

`RenderAdapter.gd`

- Owns conversion from `ClientWorldState` dictionaries to render-proxy inputs.
- May batch, pool, or expose draw-ready records.
- Must not parse packets, accept/reject commands, or decide gameplay outcomes.

`MainThreadGate.gd`

- Owns the future handoff queue from async receive/decode work to main-thread
  Godot components.
- Stores only copied dictionaries.
- Must not carry secrets, Steam tickets, provider data, or raw live session
  tokens.

## Data Flow

```text
Input/UI
  -> CommandQueue
  -> ProtocolCodec encode path later
  -> ServerConnection transport later
  -> Rust server authority
  -> ServerConnection receives bytes later
  -> ProtocolCodec decode
  -> MainThreadGate if decode ever happens off-thread
  -> SnapshotBuffer
  -> ClientWorldState
  -> RenderAdapter
  -> Godot visuals/UI/debug overlay
```

## Required Diagnostics

The bridge must expose or log these fields as implementation arrives:

- connection state,
- message type,
- protocol version,
- client sequence,
- server sequence,
- ack sequence,
- server tick,
- snapshot age,
- pending command count,
- buffered snapshot count,
- authoritative entity count,
- decode/apply/render timing metrics.

## Stop Rules

Stop or gate the slice if a bridge change would:

- make Godot authoritative for command success, movement, damage, capture,
  economy, spawn, or win/loss,
- put scene paths, Resources, or Node names into protocol payloads,
- log Steam tickets, secrets, account data, or raw provider/session tokens,
- require live Steam/network mutation without explicit Go,
- commit to final transport or render technology before its design gate.
