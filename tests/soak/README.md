# Soak Tests

Soak tests cover long sessions, reconnect churn, packet loss, jitter, slow
clients, memory growth, queue depth, log volume, and graceful shutdown.

SOAK-01 artifacts:

- `tests/soak/soak-scenarios.schema.json`
- `tests/soak/soak-scenarios.json`
- `docs/perf/soak-scenarios.md`
- `docs/runbooks/soak-scenarios.md`

These files define scenarios only. They do not run a soak test or close
release-candidate, live Steam, gameplay, final render, final ECS, or stability
claims.

SOAK-02 artifacts:

- `server/src/soak_metrics.rs`
- `client/godot/scripts/perf/SoakMetrics.gd`
- `client/godot/scripts/tests/soak_metrics_check.gd`

These emit local metric snapshots for later harnesses. They are not long-run
soak execution.

SOAK-03 artifacts:

- `scripts/run_soak_smoke.ps1`
- `tests/soak/soak-smoke-manifest.json`
- `docs/evidence/soak-smoke-evidence.md`

The smoke validates contracts and emitters. It does not run a long session.
