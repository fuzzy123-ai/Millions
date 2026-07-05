# Foundation Decisions

Date: 2026-07-02
Mode: Standard ABC
Status: initial defaults for implementation start

## Scope

This document freezes the first implementation defaults for repository
scaffolding and architecture spike work. These are starting decisions, not final
product promises.

## Decisions

| ID | Decision | Default | Reason | Revisit gate |
| --- | --- | --- | --- | --- |
| FD-01 | Server language | Rust | Dedicated authoritative simulation needs predictable performance, ownership control, and strong testability. | G-ECS-CHOICE |
| FD-02 | Godot scripting | GDScript first, no C# requirement for the first spike | Keeps local setup smaller and matches the scene/node contract. .NET is present on this machine but not required. | G-GODOT-BRIDGE |
| FD-03 | Transport spike | Custom UDP behind a transport abstraction | Allows command reliability, snapshot supersession, AOI, and backpressure to be proven without blocking on QUIC/GDExtension packaging. | G-TRANSPORT-QUIC |
| FD-04 | Protocol | `protocol_v0`, fixed little-endian binary messages | Keeps Rust/Godot fixtures simple and rejects scene/resource data in packets. | PROTO fixture tests |
| FD-05 | Tick target | 20 Hz authoritative server tick | Gives a clear 50 ms tick interval and room for large entity counts. | G-PERF-BUDGETS |
| FD-06 | Rust workspace | Root `Cargo.toml` with `server/` as first crate | Lets protocol/sim/test crates be added without reshaping the repo. | BUILD-01 |
| FD-07 | Godot project root | `client/godot/` | Keeps game client assets and scripts separate from server/protocol/test tooling. | G-GODOT-SCENE-CONTRACT |
| FD-08 | First performance baseline | Machine-specific and recorded per run | No scale claim is valid without machine, scenario, p95/p99, memory, and bandwidth data. | BUDGET-01 |

## Local Tooling Status

Installed on 2026-07-02:

- Rust toolchain through Rustup: `rustc` 1.96.1, `cargo` 1.96.1.
- Godot through WinGet: Godot 4.7 stable.

Observed on 2026-07-02:

- `.NET 8` is installed at `C:\Program Files\dotnet\dotnet.exe`.
- The first Godot client spike does not depend on C# or `dotnet`.
- `pwsh` is not installed; Windows PowerShell 5.1 is sufficient for current
  repository scripts.

## Implementation Start Order

1. `BUILD-01`: pin/check toolchain and local commands.
2. `PROTO-01`: freeze protocol v0 fields and fixtures.
3. `ARCH-01`: authority boundaries and forbidden couplings.
4. `GSCENE-01`: Godot folder/scene ownership.
5. `BUDGET-01`: performance report schema and hardware baseline.
6. `DET-01`: deterministic input/time/seed rules.

Gameplay implementation remains blocked until protocol, server, client adapter,
local harness, and basic performance gates exist.
