# Module Boundaries

Date: 2026-07-03
Status: slice ARCH-02 server/protocol boundary contract

## Purpose

This document defines the first repository boundaries for backend work. The goal
is to let backend, Godot interface, fixture, and Steam-preparation slices move
in parallel without turning any one module into hidden gameplay authority.

## Rust Server Modules

`server/src/protocol.rs`

- Owns protocol_v0 constants, message type IDs, packet header decoding, and
  pre-payload rejection rules.
- May know about wire-level connection/session IDs as numeric fields.
- Must not know about Godot scenes, Steam SDK types, gameplay outcomes, or
  rendering concepts.

`server/src/transport.rs`

- Owns transport-facing connection envelopes and the local/mock/UDP transport
  boundary.
- May wrap a decoded packet with connection metadata.
- Must not decide command success, simulation state, combat, economy, or map
  authority.
- Real Steam calls remain outside this module until a live gate is explicitly
  opened; mock/local Steam-facing shapes are allowed.

`server/src/simulation.rs`

- Owns authoritative simulation identifiers, server tick constants, and future
  state transition types.
- Will become the place where accepted commands change durable match state.
- Must not parse raw packets directly and must not depend on Godot or Steam SDK
  concepts.

`server/src/determinism.rs`

- Owns deterministic input frame shape, canonical ordering, seed derivation, and
  checksum helpers for replay/desync slices.
- Must not execute gameplay, read wall-clock time, depend on transport arrival
  order, or include Godot/Steam/provider runtime data in replayable inputs.

`server/src/fixtures.rs`

- Owns fixture path constants and fixture inventory used by Rust tests.
- May point to `protocol/fixtures/` but should not generate or mutate live
  runtime data.
- Must keep fixtures deterministic, local, and free of secrets or provider data.

`server/src/lib.rs`

- Re-exports stable foundation types for tests and future crates.
- Contains integration-level unit tests while the crate is small.
- Should not accumulate business logic that belongs in a module.

## Repository Boundary Map

| Path | Owner | Allowed responsibility | Not allowed |
| --- | --- | --- | --- |
| `protocol/fixtures/` | Protocol/fixtures | Canonical packet bytes and descriptors | Secrets, Steam tickets, private data |
| `docs/protocol/` | Protocol docs | Wire contract and fixture interpretation | Runtime implementation decisions |
| `server/` | Backend | Authoritative server, parser, transport, sim, tests | Godot scene/UI ownership |
| `client/godot/` | Godot client | Input, UI, local adapters, rendering, fixture decode later | Durable server authority |
| `docs/architecture/` | Architecture | Boundaries, contracts, stop rules | JSON planning source of truth |

## Parallel Work Rules

- Backend protocol work may change `server/src/protocol.rs`,
  `protocol/fixtures/`, and focused Rust tests.
- Godot interface work may consume the same fixtures but must keep Godot-side
  decoding in `client/godot/scripts/net/` or a planned equivalent.
- Steam-preparation work may define facade contracts and mock/local identity
  flows, but live Steam auth, real AppID use, and two-machine smoke remain
  explicit Go gates.
- Performance/evidence work may add local harnesses and report schemas, but no
  scale claim is closed without the matching evidence.

## Stop Rules

Stop and gate the slice if a change would:

- make transport or Godot authoritative for match state,
- put Steam SDK/live-provider types into simulation,
- copy prototype gameplay behavior into the new server,
- add gameplay mechanics before infrastructure gates,
- introduce secrets, tokens, Steam tickets, or private account data,
- require destructive Git, live network mutation, installs, or deploys.

Cross-lane integration checkpoints and next-slice selection rules live in
`docs/architecture/integration-checkpoints-stop-rules.md`.
