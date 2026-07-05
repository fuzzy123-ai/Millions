# Interest Bandwidth Regressions

Date: 2026-07-03
Status: slice INT-03 local Foundation evidence

## Purpose

INT-03 records the first bandwidth smoke metrics and regression gates for the
server-owned interest-management path from INT-01 and INT-02.

This is a byte-estimate smoke, not a live network measurement. It uses the
protocol v0 snapshot byte model:

- snapshot header: 24 bytes,
- entity record: 36 bytes,
- removed entity ID: 8 bytes,
- aggregate far-state cell estimate: 32 bytes.

Bandwidth is calculated as `snapshot_bytes_per_tick * 20 / 1024` KB/s per
client, matching the current 20 Hz server tick baseline.

## Evidence Files

- `tests/perf/interest-bandwidth-smoke-report.json`
- `tests/perf/interest-bandwidth-regression-thresholds.json`

Both files are local Foundation evidence. They do not close live networking,
release-candidate, soak, real Steam, gameplay visibility, or final AOI bandwidth
claims.

## Local Results

| Scenario | Visible records | Removed IDs | Aggregate cells | Bytes/tick | Bandwidth p95 | Gate |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `int_aoi_delta_steady_128_visible` | 128 | 0 | 0 | 4,632 | 90.46875 KB/s | 256 KB/s normal |
| `int_aoi_delta_churn_256_visible_32_removed` | 256 | 32 | 0 | 9,496 | 185.46875 KB/s | 256 KB/s normal |
| `int_aoi_10k_aggregate_far_state` | 512 | 0 | 200 | 24,856 | 485.46875 KB/s | 768 KB/s stress |

## Regression Gates

- Normal AOI scenarios must stay at or below 256 KB/s p95 per client.
- 10k stress aggregate scenarios must stay at or below 768 KB/s p95 per client.
- The byte-per-tick ceiling is derived from the budget key and 20 Hz tick rate.
- Any future report above the relevant ceiling is a regression until fixed or
  the JSON plan budget changes with rationale.

## Open Limits

- No socket transport, packet loss, resend, compression, fragmentation, MTU, or
  live network timing is measured.
- Aggregate far-state is still a server-side estimate and has no binary wire
  fixture.
- No gameplay visibility, fog-of-war, stealth, camera authority, or real Steam
  state is included.
- INT-03 does not make a release-candidate claim.
