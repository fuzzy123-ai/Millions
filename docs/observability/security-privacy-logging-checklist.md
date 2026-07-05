# Security And Privacy Logging Checklist

Date: 2026-07-03
Slice: `HARDEN-04`
Status: repo-only hardening checklist

## Purpose

This checklist defines what protocol hardening, hostile-input, reconnect, Steam
facade, and release-evidence logs may record. The goal is useful local evidence
without secrets, auth material, private account data, raw provider output, or
accidental gameplay authority.

## Required For Hardening Events

Every hardening/security event must include:

- `schema=millions_log_event_v0`
- `category=security` or `category=protocol`
- `source=server` or `source=harness`
- `event` using a bounded event name such as `packet_rejected`,
  `auth_rejected`, `stale_command_ignored`, or `hostile_input_case_checked`
- `reason` from the protocol hardening reason catalog
- `reject_stage` such as `header`, `payload`, `auth`, `freshness`, or `policy`
- a redacted packet class such as `truncated_header`,
  `unsupported_protocol_version`, `oversized_packet`, or `replayed_command`
- `mutate_authoritative_state=false` for every rejected, duplicate, or
  disconnected hostile input
- `redacted_diagnostics_only=true`
- bounded counters or lengths, never raw packet bytes by default

## Forbidden Fields

Never log, persist, or copy into evidence:

- Steam auth/session tickets,
- provider tokens or API keys,
- passwords or private credentials,
- raw private account data,
- raw provider payloads,
- full raw packet dumps by default,
- real non-local endpoint details in public evidence,
- private file paths outside repo-relative artifact names,
- gameplay outcomes inferred by the client or logs.

## Allowed Redacted Fields

Allowed when needed for local evidence:

- synthetic local session labels,
- redacted `player_session_id` values,
- numeric `connection_id`, `command_id`, `snapshot_id`, and sequence numbers,
- packet length, declared payload length, and header length,
- reason/action words such as `version_mismatch`,
  `reject_no_state_mutation`, or `disconnect`,
- aggregate counts and timings,
- repo-relative fixture, harness, or evidence paths.

## Review Checklist

Before a hardening log, harness output, or evidence row is considered safe:

- It uses a known log category, severity, and reason word.
- It records the reject stage and action without raw private payloads.
- It states whether authoritative state was mutated.
- It records redaction status.
- It bounds diagnostic length to the current hardening limit.
- It avoids live Steam, real AppID, public endpoint, and provider details unless
  a future live gate explicitly allows a redacted shape.
- It can be shared as local Foundation evidence without implying release,
  broader playtest, or gameplay readiness.

## No-Go Conditions

Mark the run or slice `No-Go` if logs or evidence would:

- include secrets, tickets, tokens, private account data, or raw provider data,
- include raw packet dumps without a future explicit diagnostic gate,
- expose real endpoint details in public evidence,
- use client-side or log-side data as gameplay authority,
- claim live security coverage, broader playtest readiness, or
  release-candidate hardening from local-only evidence,
- skip the JSON plan handoff after a checklist change.

## Related Artifacts

- `docs/observability/log-event-schema.md`
- `docs/protocol/protocol-hardening-v0.md`
- `docs/evidence/hostile-input-smoke-evidence.md`
- `scripts/run_hostile_input_smoke.ps1`
- `tests/hardening/hostile-input-cases.json`
