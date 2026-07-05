# Tests

Foundation test entrypoints:

- Local full matrix:
  `powershell -ExecutionPolicy Bypass -File scripts\run_foundation_matrix.ps1`
- CI-safe matrix without Godot:
  `powershell -ExecutionPolicy Bypass -File scripts\run_foundation_matrix.ps1 -SkipGodot`

The CI-safe matrix validates plans, foundation checks, Rust formatting/tests,
server smoke, packet-loss/jitter harness, replay smoke, overload smoke,
hostile-input smoke, and perf schema parsing.
The local full matrix additionally runs Godot fixture, adapter, render, lobby,
perf, and multi-client smokes.

Neither matrix calls live Steam, opens public network state, installs tools, or
claims gameplay/scale performance.
