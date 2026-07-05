# Authority Boundaries

Date: 2026-07-02
Status: slice ARCH-01 foundation contract

## Prime Rule

The dedicated Rust server owns authoritative match state. Godot may predict,
preview, render, animate, and explain state, but it must not decide durable
simulation outcomes.

## Server Authority

The server owns:

- match clock and authoritative tick,
- player session identity after handshake,
- command acceptance, rejection, ordering, and idempotency,
- entity creation, removal, position, health, faction, role, and state,
- movement validity, pathing decisions, collision, and map constraints,
- combat, damage, cover, capture, economy, spawn, production, and win/loss
  outcomes once gameplay slices begin,
- AOI visibility and snapshot contents,
- reconnect grace, session rebind, full snapshot restore, and stale command
  handling,
- overload policy, rate limits, slow-client behavior, and hardening decisions.

The server may accept client intent, never client truth. A client can ask to
move, attack, spawn, ready, or reconnect; the server decides whether the request
changes state.

## Client And Godot Authority

The Godot client owns:

- window, rendering, camera, UI, input collection, and local accessibility,
- lobby screens, copy/join UX, ready button presentation, and local validation
  hints,
- command intent construction before sending to the server,
- client-side prediction and feedback that can be corrected by snapshots,
- interpolation, smoothing, visual batching, selection overlays, and debug
  overlays,
- local logs and counters after redaction,
- scene tree organization under the Godot scene/node contract.

The client does not own:

- entity truth,
- command success,
- map/collision truth,
- combat/economy/capture/spawn outcomes,
- final player/session authority,
- replay determinism,
- performance milestone claims.

If prediction diverges from an authoritative snapshot, the snapshot wins and the
client reconciles visually.

## Protocol Boundary

Protocol v0 carries numeric, fixed-layout match facts and client intents. It
must not carry Godot scene paths, Resource paths, Node names, raw Steam tickets,
secrets, private account data, or client-computed gameplay outcomes.

Protocol fixture work is the gate between documentation and implementation:
Rust and Godot must decode the same packet bytes into the same fields before
live networking expands.

## Steam And Lobby Boundary

Steam is used for lobby/discovery/identity flow when the relevant live gates are
open. It does not become match authority.

The planned session shape is:

1. Steam or local mode establishes who wants to join.
2. The dedicated server assigns or resumes `player_session_id`.
3. The server validates readiness and match admission.
4. The server remains authoritative after match start.

Spacewar or local synthetic identity may be used for development, but release
confidence waits on the real AppID and two-machine live gates.

## Map And Content Boundary

Godot-authored map data can be the editing source, but runtime authority requires
an exported, versioned, checksum-verified data contract before gameplay depends
on cover, obstacles, spawn points, capture points, or navigation data.

Until `MAPDATA` gates are complete, visual markers and scene placeholders are
not authoritative.

The authored map data boundary lives in
`docs/architecture/authored-map-data-contract.md`; gameplay-facing notes live in
`docs/gameplay/map-data-authority-notes.md`.

Movement and pathfinding options are tracked in
`docs/architecture/movement-model-options.md`. That document is an option and
evidence contract only; it does not select or implement authoritative movement
behavior.

## Performance And Evidence Boundary

No scale, stability, or release-candidate claim is valid without matching
evidence:

- server tick metrics,
- snapshot size and bandwidth metrics,
- Godot decode/apply/render/frame metrics,
- memory and visible entity counts,
- reconnect full snapshot time where relevant,
- deterministic replay or desync evidence for simulation behavior,
- soak/chaos evidence for long-run confidence.

Performance budgets are allowed to be provisional, but claims must state the
hardware, scenario, percentile, and evidence artifact.

## Observability Boundary

Structured logs and debug overlays are diagnostic evidence only. They must not
become gameplay authority, session authority, Steam authority, or release
evidence without the matching artifact and gate. The baseline event schema,
categories, severity levels, redaction rules, and correlation IDs live in
`docs/observability/log-event-schema.md`.

Backpressure and slow-client policy remains server authority. The LOADSHED-01
contract lives in `docs/protocol/backpressure-rate-limits.md` and
`docs/architecture/backpressure-slow-client-policy.md`.

## Non-Goals Before Foundation Gates

Before BUILD, PROTO, ARCH, GSCENE, BUDGET, and DET foundation work is stable,
do not implement:

- real gameplay mechanics,
- final faction or unit balance,
- final rendering technology choices,
- Steam live auth/session validation,
- real AppID assumptions,
- art-return or final asset pipeline work,
- complex movement/pathfinding behavior,
- authoritative map gameplay,
- release-candidate packaging claims.

Prototype behavior from any earlier project may inform requirements and tests,
but must not be copied as gameplay implementation.

## Stop Conditions

Stop or gate the slice if work would:

- persist secrets, tokens, Steam auth tickets, or private provider/session data,
- make Godot authoritative for durable match state,
- add gameplay before the required infrastructure gates,
- hide protocol data in scene/resource/node payloads,
- claim scale without budget and evidence,
- require live Steam/provider/server mutation without explicit Go,
- require destructive Git or rewriting unrelated work.

Integration checkpoint and lane stop rules live in
`docs/architecture/integration-checkpoints-stop-rules.md`.
