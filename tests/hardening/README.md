# Hostile Input Harness

`HARDEN-03` owns the local hostile-input harness cases in this directory.

- `hostile-input-cases.json` is the machine-readable case catalog for hostile
  parser and policy inputs.
- `scripts/run_hostile_input_smoke.ps1` validates this catalog, cross-checks
  required fuzz seed families from `tests/fuzz/protocol-hardening-corpus.json`,
  and runs the Rust `hardening` tests.

The harness is repo-only evidence. It does not open sockets, call Steam, use a
real AppID, run a public-network smoke, validate gameplay payload semantics, or
claim release-candidate hardening.
