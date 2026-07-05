# Desync Triage

Date: 2026-07-03
Status: slice DET-04 triage runbook

## Purpose

This runbook defines how to handle deterministic replay divergence once replay
reports can compare frame checksums. It is local/offline guidance and does not
claim gameplay replay coverage yet.

## Inputs

Use only repo-safe evidence:

- replay report name,
- first divergent tick,
- frame checksum before and after divergence,
- deterministic seed inputs,
- redacted command IDs and command types,
- local fixture or harness command used.

Do not include:

- Steam auth tickets,
- provider tokens,
- private account data,
- private machine paths,
- raw live provider/session output,
- Godot scene paths as server truth.

## Triage Steps

1. Re-run `powershell -ExecutionPolicy Bypass -File scripts\run_replay_smoke.ps1`.
2. Confirm plan and foundation gates with the standard checks.
3. Compare the first divergent tick across reports.
4. Inspect only deterministic inputs for that tick and the previous tick.
5. Classify the divergence:
   - `input_order`: canonical ordering changed,
   - `input_value`: command/session/tick/payload hash changed,
   - `seed`: match/map/protocol seed input changed,
   - `snapshot`: future authoritative snapshot checksum changed,
   - `unknown`: evidence is insufficient.
6. Add or update a replay fixture before changing simulation logic.
7. Stop and gate if the evidence would require live Steam, private data, or
   gameplay behavior outside the active slice.

## Status Language

`Go`: local replay smoke passes and divergence evidence is fixture-backed.

`Deferred`: a future gameplay snapshot, map checksum, or live evidence source is
needed.

`No-Go`: triage would require secrets, raw Steam tickets, private provider data,
wall-clock replay inputs, or unexplained gameplay authority.

## Handoff

Record:

- divergent tick or `none`,
- command run,
- fixture/report path,
- classification,
- open gate,
- next safe slice.
