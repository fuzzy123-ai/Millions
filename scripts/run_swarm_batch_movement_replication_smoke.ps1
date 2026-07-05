$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\swarm-batch-movement-replication-smoke-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing swarm batch movement replication report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GSWARM-13") {
    throw "Swarm batch movement replication report slice mismatch: $($report.slice)"
}
if ($report.scenario_id -ne "swarm_batch_movement_replication_smoke") {
    throw "Swarm batch movement replication scenario mismatch: $($report.scenario_id)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Swarm batch movement replication must remain informational/blocked until live transport and formal budgets exist."
}
if ($report.claim_scope -ne "local_batch_movement_replication_smoke_only") {
    throw "Swarm batch movement replication claim scope must remain local_batch_movement_replication_smoke_only."
}
if ([int]$report.active_zombie_count -ne 1000) {
    throw "Swarm batch movement replication must cover 1,000 active zombies."
}
foreach ($field in @("baseline_snapshot_entity_count", "movement_snapshot_entity_count", "delta_visible_entity_count")) {
    if ([int]$report.$field -ne [int]$report.active_zombie_count) {
        throw "Swarm batch movement replication $field must match active zombie count."
    }
}
foreach ($field in @(
    "delta_changed_visible_entity_count_min",
    "delta_snapshot_bytes_min",
    "delta_bandwidth_kb_s_per_client_min",
    "aggregate_visible_entity_count_min",
    "aggregate_changed_visible_entity_count_min",
    "aggregate_far_state_count_min",
    "aggregate_snapshot_bytes_min",
    "movement_sample_count_min",
    "movement_applied_delta_count_min",
    "movement_physics_iterations_run_min"
)) {
    if ([double]$report.$field -lt 1) {
        throw "Swarm batch movement replication must require positive evidence for $field."
    }
}
if ([int]$report.delta_removed_entity_count -ne 0) {
    throw "Swarm batch movement replication all-visible delta should not remove stable visible zombies."
}
if ([int]$report.delta_aggregate_far_state_count -ne 0) {
    throw "Swarm batch movement replication all-visible delta should not emit aggregate far-state."
}
if ([int]$report.aggregate_visible_entity_count_max -ge [int]$report.active_zombie_count) {
    throw "Swarm batch movement replication aggregate-visible count must stay below all-visible count."
}
if ($report.non_claims.Count -lt 7) {
    throw "Swarm batch movement replication report must keep explicit non-claims."
}

cargo test swarm_batch_movement_replication_smoke -- --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "cargo test swarm_batch_movement_replication_smoke failed with exit code $LASTEXITCODE."
}

Write-Host "swarm_batch_movement_replication_smoke status=ok zombies=$($report.active_zombie_count) delta_changed_min=$($report.delta_changed_visible_entity_count_min) aggregate_far_min=$($report.aggregate_far_state_count_min) budget_result=$($report.budget_result)"
