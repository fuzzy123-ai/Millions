$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\swarm-batch-vs-single-promotion-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing swarm batch-vs-single promotion report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GSWARM-12") {
    throw "Swarm batch-vs-single promotion report slice mismatch: $($report.slice)"
}
if ($report.scenario_id -ne "swarm_batch_vs_single_movement_loop_measurement") {
    throw "Swarm batch-vs-single promotion scenario mismatch: $($report.scenario_id)"
}
if ($report.scenario_family -ne "swarm_batch_vs_single") {
    throw "Swarm batch-vs-single promotion family mismatch: $($report.scenario_family)"
}
if ($report.status -ne "blocked" -or $report.decision -ne "blocked" -or $report.budget_result -ne "blocked") {
    throw "Swarm batch-vs-single promotion must remain blocked until enough comparable local rows exist."
}
if ($report.claim_scope -ne "local_regression_signal") {
    throw "Swarm batch-vs-single promotion must remain local_regression_signal."
}
if ($report.reason -ne "insufficient_comparable_rows") {
    throw "Swarm batch-vs-single promotion report must document insufficient comparable rows."
}
if ([int]$report.min_rows_required -lt 3) {
    throw "Swarm batch-vs-single promotion must require at least three comparable rows."
}
if ([int]$report.current_comparable_rows -ge [int]$report.min_rows_required) {
    throw "Static promotion report must remain a blocked single-row sample."
}
if ([int]$report.promotion_policy.max_batch_to_single_p95_bps -ne 12000) {
    throw "Swarm batch-vs-single promotion ratio threshold must match the PHIST threshold."
}
if ([int64]$report.promotion_policy.max_single_elapsed_us_p95 -ne 20000000) {
    throw "Swarm batch-vs-single promotion single-loop p95 threshold must match the PHIST threshold."
}
if ([int64]$report.promotion_policy.max_batch_elapsed_us_p95 -ne 20000000) {
    throw "Swarm batch-vs-single promotion batch-loop p95 threshold must match the PHIST threshold."
}
if ($report.non_claims.Count -lt 8) {
    throw "Swarm batch-vs-single promotion report must keep explicit non-claims."
}

cargo test swarm_batch_vs_single_promotion -- --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "cargo test swarm_batch_vs_single_promotion failed with exit code $LASTEXITCODE."
}

Write-Host "swarm_batch_vs_single_promotion status=ok decision=$($report.decision) min_rows=$($report.min_rows_required) current_rows=$($report.current_comparable_rows) budget_result=$($report.budget_result)"
