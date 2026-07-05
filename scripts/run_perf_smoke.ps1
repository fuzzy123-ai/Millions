$ErrorActionPreference = "Stop"

Get-Content tests\perf\perf-report.schema.json -Raw | ConvertFrom-Json | Out-Null
Get-Content tests\perf\sample-perf-report-row.json -Raw | ConvertFrom-Json | Out-Null
Get-Content tests\perf\performance-history-ledger.schema.json -Raw | ConvertFrom-Json | Out-Null
Get-Content tests\perf\perf-history-swarm-row-sample.json -Raw | ConvertFrom-Json | Out-Null
Get-Content tests\perf\perf-history-swarm-batch-vs-single-row-sample.json -Raw | ConvertFrom-Json | Out-Null
$swarmBatchReplication = Get-Content tests\perf\swarm-batch-movement-replication-smoke-report.json -Raw | ConvertFrom-Json
if ($swarmBatchReplication.slice -ne "GSWARM-13") {
    throw "Swarm batch movement replication report slice mismatch: $($swarmBatchReplication.slice)"
}
if ($swarmBatchReplication.status -ne "informational" -or $swarmBatchReplication.budget_result -ne "blocked") {
    throw "Swarm batch movement replication report must remain informational/blocked until live transport and formal budgets exist."
}
if ([int]$swarmBatchReplication.delta_changed_visible_entity_count_min -lt 1) {
    throw "Swarm batch movement replication must require changed visible delta entities."
}
if ([int]$swarmBatchReplication.aggregate_far_state_count_min -lt 1) {
    throw "Swarm batch movement replication must require aggregate far-state evidence."
}
$swarmBatchPromotion = Get-Content tests\perf\swarm-batch-vs-single-promotion-report.json -Raw | ConvertFrom-Json
if ($swarmBatchPromotion.slice -ne "GSWARM-12") {
    throw "Swarm batch-vs-single promotion report slice mismatch: $($swarmBatchPromotion.slice)"
}
if ($swarmBatchPromotion.status -ne "blocked" -or $swarmBatchPromotion.decision -ne "blocked") {
    throw "Swarm batch-vs-single promotion report must remain blocked until enough comparable rows exist."
}
if ([int]$swarmBatchPromotion.min_rows_required -lt 3) {
    throw "Swarm batch-vs-single promotion must require at least three comparable rows."
}
$historyThresholds = Get-Content docs\perf\performance-history-regression-thresholds.json -Raw | ConvertFrom-Json
$swarmThreshold = @($historyThresholds.thresholds | Where-Object {
    $_.scenario_id -eq "swarm_1k_server_behavior_collision_load" -and
    $_.scenario_family -eq "swarm_collision"
})[0]
if (-not $swarmThreshold) {
    throw "Performance history thresholds must include swarm_collision for swarm_1k_server_behavior_collision_load."
}
if ($swarmThreshold.claim_scope -ne "local_regression_signal") {
    throw "Swarm/collision PHIST threshold must remain a local regression signal."
}
foreach ($key in @("swarm_local_smoke_elapsed_us_max", "swarm_snapshot_bytes_max", "swarm_bandwidth_kb_s_per_client_max")) {
    if ($swarmThreshold.budget_keys -notcontains $key) {
        throw "Swarm/collision PHIST threshold is missing budget key $key."
    }
}
if ([double]$swarmThreshold.metrics."local_elapsed_us.total.max" -le 0) {
    throw "Swarm/collision PHIST threshold must include a positive local elapsed total max."
}
if ([double]$swarmThreshold.metrics."local_elapsed_us.configured_movement_loop.max" -le 0) {
    throw "Swarm/collision PHIST threshold must include a positive configured movement loop elapsed max."
}
if ([double]$swarmThreshold.metrics."snapshot_bytes.p95.max" -lt 36024) {
    throw "Swarm/collision PHIST threshold snapshot ceiling must cover the local smoke snapshot size."
}
$swarmBatchThreshold = @($historyThresholds.thresholds | Where-Object {
    $_.scenario_id -eq "swarm_batch_vs_single_movement_loop_measurement" -and
    $_.scenario_family -eq "swarm_batch_vs_single"
})[0]
if (-not $swarmBatchThreshold) {
    throw "Performance history thresholds must include swarm_batch_vs_single for swarm_batch_vs_single_movement_loop_measurement."
}
if ($swarmBatchThreshold.claim_scope -ne "local_regression_signal") {
    throw "Swarm batch-vs-single PHIST threshold must remain a local regression signal."
}
foreach ($key in @("swarm_batch_vs_single_p95_ratio_bps_max", "swarm_batch_movement_loop_p95_us_max", "swarm_single_movement_loop_p95_us_max")) {
    if ($swarmBatchThreshold.budget_keys -notcontains $key) {
        throw "Swarm batch-vs-single PHIST threshold is missing budget key $key."
    }
}
if ([double]$swarmBatchThreshold.metrics."server_tick_ms.p95.max" -le 0) {
    throw "Swarm batch-vs-single PHIST threshold must include a positive p95 ratio max."
}
if ([double]$swarmBatchThreshold.metrics."local_elapsed_us.movement_tick.max" -le 0) {
    throw "Swarm batch-vs-single PHIST threshold must include a positive single-loop p95 max."
}
if ([double]$swarmBatchThreshold.metrics."local_elapsed_us.configured_movement_tick.max" -le 0) {
    throw "Swarm batch-vs-single PHIST threshold must include a positive batch-loop p95 max."
}

& C:\Users\nkatz\.cargo\bin\cargo.exe test metrics::
& C:\Users\nkatz\.cargo\bin\cargo.exe test perf_budget::

$godotRoot = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe"
$godot = Get-ChildItem -Path $godotRoot -Filter "Godot*_console.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $godot) {
    throw "Godot executable not found."
}

& $godot.FullName --headless --path client\godot --script res://scripts/tests/perf_budget_check.gd

Write-Host "perf_smoke status=ok schema=ok rust_metrics=ok rust_budget=ok godot_budget=ok"
