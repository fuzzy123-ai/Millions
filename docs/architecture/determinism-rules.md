# Determinism Rules

Date: 2026-07-03
Status: slice DET-01 deterministic input contract

## Purpose

Deterministic replay starts with stable simulation inputs. This document defines
the allowed input facts for server-side replay and desync investigation before
gameplay systems exist.

## Input Facts

Each deterministic input belongs to exactly one server tick and contains only:

- server `tick`,
- stable `player_session_id`,
- stable `command_id`,
- numeric `command_type`,
- numeric `target_tick`,
- bounded `payload_hash`.

The input frame is canonicalized by `player_session_id`, `command_id`,
`command_type`, and `target_tick` before checksum or replay work. Arrival order,
transport kind, Godot node state, Steam provider state, wall-clock time, and log
text are not deterministic simulation inputs.

## Seed Rules

Simulation seed derivation is deterministic and repo-local:

- match ID,
- map/content checksum,
- protocol version.

No random OS source, current time, live Steam identifier, machine path, process
ID, thread scheduling detail, or floating-point renderer value may feed
simulation seed derivation.

## Server Module

`server/src/determinism.rs` owns the current DET-01 model:

- `DeterministicInput`,
- `DeterministicInputFrame::canonical`,
- stable frame checksum,
- `derive_simulation_seed`.

The module records input shape only. It does not execute gameplay, choose
outcomes, or serialize a replay file yet.

## Replay Handoff

Later DET slices must preserve these rules:

- `DET-02` records command streams using this canonical input shape.
- `DET-03` compares golden snapshot/checksum fixtures by tick.
- `DET-04` documents desync triage around first divergent tick.

## Stop Rules

Stop and gate determinism work if it would:

- depend on wall-clock time for replayable behavior,
- use live Steam/provider data as simulation truth,
- include Godot scene paths, Nodes, Resources, or renderer state in server
  inputs,
- use floating-point values as authoritative replay inputs,
- add gameplay mechanics before infrastructure gates are stable,
- persist secrets, raw Steam tickets, private account data, or provider tokens.
