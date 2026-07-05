# Overload Tests

`LOADSHED-03` owns the local overload harness cases in this directory.

- `load-shed-overload-cases.json` is the machine-readable case catalog for command rate, bandwidth/backlog, log volume, and resend-window overload behavior.
- `scripts/run_overload_smoke.ps1` validates the catalog, refuses non-local modes, checks expected policy outcomes, and runs the Rust `load_shed` tests.

The harness is repo-only evidence. It does not start live networking, call Steam, run a soak, or claim release-candidate overload readiness.
