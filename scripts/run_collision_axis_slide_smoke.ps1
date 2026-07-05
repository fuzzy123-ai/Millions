$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\collision-axis-slide-smoke-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing collision axis-slide smoke report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GSWARM-14") {
    throw "Collision axis-slide report slice mismatch: $($report.slice)"
}
if ($report.scenario_id -ne "collision_axis_slide_probe") {
    throw "Collision axis-slide scenario mismatch: $($report.scenario_id)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Collision axis-slide report must remain informational/blocked until final movement and formal budgets exist."
}
if ($report.claim_scope -ne "axis_slide_probe_only") {
    throw "Collision axis-slide report must keep claim_scope axis_slide_probe_only."
}
if ([int]$report.direct_blocked_case_count -lt 1) {
    throw "Collision axis-slide smoke must require at least one blocked direct movement case."
}
if ([int]$report.axis_slide_attempt_count_min -lt 1) {
    throw "Collision axis-slide smoke must require slide attempts."
}
if ([int]$report.axis_slide_accepted_count_min -lt 1) {
    throw "Collision axis-slide smoke must require an accepted axis-slide candidate."
}
if ([int]$report.direct_accepted_no_attempt_count -lt 1) {
    throw "Collision axis-slide smoke must require direct-accepted movement to skip slide attempts."
}
if ([int]$report.clamped_slide_attempt_count_min -lt 1) {
    throw "Collision axis-slide smoke must require clamped slide attempts."
}
if ([int]$report.clamped_correction_limit_abs_mm -ne 50) {
    throw "Collision axis-slide clamp limit must remain 50mm."
}
if ($report.non_claims.Count -lt 6) {
    throw "Collision axis-slide report must preserve conservative non-claims."
}

cargo test collision_axis_slide -- --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "cargo test collision_axis_slide failed with exit code $LASTEXITCODE."
}

Write-Host "collision_axis_slide_smoke status=ok direct_blocked=$($report.direct_blocked_case_count) slide_min=$($report.axis_slide_accepted_count_min) budget_result=$($report.budget_result)"
