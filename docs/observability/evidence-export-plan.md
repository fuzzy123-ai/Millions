# Observability Evidence Export Plan

Date: 2026-07-03
Slice: GOBS-04
Status: local evidence export contract

## Purpose

Observability evidence needs a repeatable local export shape so logs, counters,
debug-field snapshots, and troubleshooting handoffs can be bundled without
leaking secrets or claiming more than the underlying checks prove.

This plan defines the allowed local artifact families, required manifest fields,
redaction rules, and pass/fail semantics for future evidence-smoke tooling.

## Artifact Families

| Family | Example source | Export format | Claim scope |
| --- | --- | --- | --- |
| `log_schema` | `docs/observability/log-event-schema.md` | markdown/json summary | schema contract only |
| `counter_catalog` | `server/src/observability.rs`, `PerfLedger.gd` | manifest row/json snapshot | local diagnostic counters |
| `debug_overlay_fields` | `docs/observability/debug-overlay-fields.md` | markdown/json summary | planned display fields only |
| `troubleshooting_runbook` | `docs/runbooks/observability-troubleshooting.md` | markdown reference | operator procedure |
| `local_check_output` | focused smoke command output | redacted text/json line | result of one local check |
| `perf_report_reference` | `tests/perf/*.json` | JSON file reference | perf claim only if report passes its budget |

## Manifest Fields

Every exported observability evidence bundle should include:

- `schema_version`
- `export_id`
- `created_utc`
- `repo_state`
- `source_slice`
- `run_id`
- `artifact_family`
- `artifact_path`
- `redaction_status`
- `claim_scope`
- `checks`
- `blocked_gates`
- `notes`

## Redaction Rules

An export is blocked if it contains:

- Steam auth/session tickets,
- provider tokens,
- secrets, passwords, API keys, or private credentials,
- private account data,
- raw provider payloads,
- raw packet dumps not explicitly approved by a later hardening gate,
- real non-local endpoint details in public evidence.

Allowed evidence should use synthetic IDs, local/mock session IDs, aggregate
counts, and repo-relative paths.

## Status Semantics

| Status | Meaning |
| --- | --- |
| `pass` | Required manifest fields are present, redaction passes, and referenced checks passed. |
| `blocked` | A required field, check, gate, or redaction proof is missing. |
| `fail` | Redaction fails, referenced check fails, or artifact contradicts its claim. |
| `informational` | Artifact records a contract or local observation without closing a gate. |

## Bundle Rule

A bundle may combine multiple artifact families, but the bundle claim is only
as strong as the weakest artifact. If any required artifact is `blocked`, the
bundle cannot close release, scale, gameplay, live Steam, or security claims.

## First Manifest

The initial machine-readable manifest lives at:

```text
tests/observability/evidence-export-manifest.json
```

It is a contract sample, not a live export.
