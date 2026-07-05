$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\swarm-batch-vs-single-movement-loop-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing swarm batch-vs-single movement-loop report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GSWARM-10") {
    throw "Swarm batch-vs-single movement-loop report slice mismatch: $($report.slice)"
}
if ($report.scenario_id -ne "swarm_batch_vs_single_movement_loop_measurement") {
    throw "Swarm batch-vs-single movement-loop scenario mismatch: $($report.scenario_id)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Swarm batch-vs-single movement-loop measurement must remain informational/blocked until a formal p95 budget exists."
}
if ($report.claim_scope -ne "local_batch_vs_single_measured_harness_only") {
    throw "Swarm batch-vs-single movement-loop claim scope must remain local_batch_vs_single_measured_harness_only."
}
if ([int]$report.sample_count -lt 3) {
    throw "Swarm batch-vs-single movement-loop measurement must keep at least three samples for p50/p95/p99 shape."
}
if ([int]$report.tick_count_per_sample -ne 2) {
    throw "Swarm batch-vs-single movement-loop measurement must keep a bounded two-tick sample."
}
if ([int]$report.movement_sample_limit -ne 2) {
    throw "Swarm batch-vs-single movement-loop measurement must keep two movement samples per tick."
}
if ([int]$report.active_zombie_count -ne 1000) {
    throw "Swarm batch-vs-single movement-loop measurement must cover 1,000 active zombies."
}

$elapsedFields = @(
    "single_elapsed_us_p50_min",
    "single_elapsed_us_p95_min",
    "single_elapsed_us_p99_min",
    "batch_elapsed_us_p50_min",
    "batch_elapsed_us_p95_min",
    "batch_elapsed_us_p99_min",
    "batch_to_single_elapsed_p95_bps_min"
)
foreach ($field in $elapsedFields) {
    if ([int64]$report.$field -lt 1) {
        throw "Swarm batch-vs-single movement-loop measurement must require positive evidence for $field."
    }
}

$expectedMovementSamples = [int]$report.sample_count * [int]$report.tick_count_per_sample * [int]$report.movement_sample_limit
if ([int]$report.single_movement_sample_count_total -ne $expectedMovementSamples) {
    throw "Swarm batch-vs-single single sample total must equal samples * ticks * movement limit."
}
if ([int]$report.batch_movement_sample_count_total -ne $expectedMovementSamples) {
    throw "Swarm batch-vs-single batch sample total must equal samples * ticks * movement limit."
}
if ([int]$report.single_applied_delta_count_total_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require single-mode applied movement deltas."
}
if ([int]$report.batch_applied_delta_count_total_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require batch-mode applied movement deltas."
}
if ([int]$report.single_physics_iterations_run_total_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require single-mode local physics iterations."
}
if ([int]$report.batch_physics_iterations_run_total_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require batch-mode local physics iterations."
}
if ([int64]$report.single_flow_field_cache_hit_count_total_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require single-mode cache hits."
}
if ([int64]$report.batch_flow_field_cache_hit_count_total_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require batch-mode cache hits."
}
if ([int64]$report.single_flow_field_cache_eviction_count_total_max -ne 0) {
    throw "Swarm batch-vs-single single-mode cache must not evict during the bounded scenario."
}
if ([int64]$report.batch_flow_field_cache_eviction_count_total_max -ne 0) {
    throw "Swarm batch-vs-single batch-mode cache must not evict during the bounded scenario."
}
if ([int]$report.single_moved_entity_count_min_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require single-mode position changes in every sample."
}
if ([int]$report.batch_moved_entity_count_min_min -lt 1) {
    throw "Swarm batch-vs-single measurement must require batch-mode position changes in every sample."
}
if ($report.non_claims.Count -lt 8) {
    throw "Swarm batch-vs-single measurement must keep explicit non-claims."
}

cargo test swarm_batch_vs_single_movement_loop_measurement -- --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "cargo test swarm_batch_vs_single_movement_loop_measurement failed with exit code $LASTEXITCODE."
}

Write-Host "swarm_batch_vs_single_movement_loop_measurement status=ok samples=$($report.sample_count) active=$($report.active_zombie_count) ticks=$($report.tick_count_per_sample) movement_samples=$($report.single_movement_sample_count_total) budget_result=$($report.budget_result)"
