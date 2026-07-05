$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\swarm-movement-loop-measurement-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing swarm movement-loop measurement report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GSWARM-05") {
    throw "Swarm movement-loop measurement slice mismatch: $($report.slice)"
}
if ($report.scenario_id -ne "swarm_configured_movement_loop_measurement") {
    throw "Swarm movement-loop measurement scenario mismatch: $($report.scenario_id)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Swarm movement-loop measurement must remain informational/blocked until a formal p95 budget exists."
}
if ($report.claim_scope -ne "local_measured_harness_only") {
    throw "Swarm movement-loop measurement claim scope must remain local_measured_harness_only."
}
if ([int]$report.sample_count -lt 3) {
    throw "Swarm movement-loop measurement must keep at least three samples for p50/p95/p99 shape."
}
if ([int]$report.tick_count_per_sample -ne 2) {
    throw "Swarm movement-loop measurement must keep a bounded two-tick sample."
}
if ([int]$report.movement_sample_limit -ne 2) {
    throw "Swarm movement-loop measurement must keep two movement samples per tick."
}
if ([int]$report.active_zombie_count -ne 1000) {
    throw "Swarm movement-loop measurement must cover 1,000 active zombies."
}
if ([int]$report.spawned_count_total -ne 0) {
    throw "Swarm movement-loop measurement must run after the active cap and not spawn extra zombies."
}

$elapsedFields = @(
    "elapsed_us_min_min",
    "elapsed_us_p50_min",
    "elapsed_us_p95_min",
    "elapsed_us_p99_min",
    "elapsed_us_max_min"
)
foreach ($field in $elapsedFields) {
    if ([int64]$report.$field -lt 1) {
        throw "Swarm movement-loop measurement must require positive elapsed-time evidence for $field."
    }
}

$expectedMovementSamples = [int]$report.sample_count * [int]$report.tick_count_per_sample * [int]$report.movement_sample_limit
if ([int]$report.movement_sample_count_total -ne $expectedMovementSamples) {
    throw "Swarm movement-loop measurement sample total must equal samples * ticks * movement limit."
}
if ([int]$report.applied_delta_count_total_min -lt 1) {
    throw "Swarm movement-loop measurement must require applied movement deltas."
}
if ([int]$report.physics_iterations_run_total_min -lt 1) {
    throw "Swarm movement-loop measurement must require local physics iterations."
}
if ([int64]$report.flow_field_cache_request_count_total -ne [int64]$report.movement_sample_count_total) {
    throw "Swarm movement-loop measurement cache requests must match movement samples."
}
if ([int64]$report.flow_field_cache_hit_count_total_min -lt 1) {
    throw "Swarm movement-loop measurement must require cache hits."
}
if ([int64]$report.flow_field_cache_eviction_count_total_max -ne 0) {
    throw "Swarm movement-loop measurement must not evict during the bounded scenario."
}
if ([int]$report.flow_field_cache_entry_count_max_max -gt 32) {
    throw "Swarm movement-loop measurement cache entry cap must stay at or below 32."
}
if ([int]$report.moved_entity_count_min_min -lt 1) {
    throw "Swarm movement-loop measurement must require position changes in every sample."
}
if ($report.non_claims.Count -lt 7) {
    throw "Swarm movement-loop measurement must keep explicit non-claims."
}

cargo test swarm_configured_movement_loop_measurement -- --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "cargo test swarm_configured_movement_loop_measurement failed with exit code $LASTEXITCODE."
}

Write-Host "swarm_movement_loop_measurement status=ok samples=$($report.sample_count) active=$($report.active_zombie_count) ticks=$($report.tick_count_per_sample) movement_samples=$($report.movement_sample_count_total) budget_result=$($report.budget_result)"
