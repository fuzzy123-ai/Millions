$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\collision-physics-smoke-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing collision physics smoke report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "COLLISION-PHYSICS-SMOKE") {
    throw "Collision physics report slice mismatch: $($report.slice)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Collision physics report must remain informational/blocked until measured p95 evidence exists."
}
if ($report.claim_scope -ne "local_smoke_only") {
    throw "Collision physics report must keep claim_scope local_smoke_only."
}
if (@($report.scenarios).Count -ne 2) {
    throw "Collision physics report must contain exactly the 1k and 5k smoke scenarios."
}
if (@($report.non_claims).Count -lt 5) {
    throw "Collision physics report must preserve conservative non-claims."
}

$scenarioIds = @($report.scenarios | ForEach-Object { $_.scenario_id })
if ($scenarioIds -notcontains "collision_clustered_swarm_1k") {
    throw "Collision physics report is missing collision_clustered_swarm_1k."
}
if ($scenarioIds -notcontains "collision_clustered_swarm_5k") {
    throw "Collision physics report is missing collision_clustered_swarm_5k."
}

foreach ($scenario in @($report.scenarios)) {
    if ([int]$scenario.body_count -lt 1000) {
        throw "Collision physics scenario $($scenario.scenario_id) must cover at least 1,000 bodies."
    }
    if ([int]$scenario.cluster_count -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must include clusters."
    }
    if ([int]$scenario.initial_contact_count_min -lt [int]$scenario.body_count) {
        throw "Collision physics scenario $($scenario.scenario_id) must require contact pressure."
    }
    if ([int]$scenario.resolved_admission_check_count -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require resolved admission checks."
    }
    if ([int]$scenario.resolved_admission_accepted_after_resolution_count_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require at least one resolved admission accept after bounded resolution."
    }
    if ([int]$scenario.resolved_admission_rejected_count_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require at least one unresolved admission under pressure."
    }
    if ([int]$scenario.static_obstacle_resolution_check_count -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require a static obstacle resolution check."
    }
    if ([int]$scenario.static_obstacle_correction_count_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require at least one dynamic correction against a static obstacle."
    }
    if ([int]$scenario.clamped_resolution_check_count -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require a clamped stability check."
    }
    if ([int]$scenario.clamped_correction_limit_abs_mm -ne 50) {
        throw "Collision physics scenario $($scenario.scenario_id) must keep the clamped correction limit at 50mm."
    }
    if ([int]$scenario.clamped_correction_count_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require at least one clamped correction."
    }
    if ([int]$scenario.clamped_max_applied_correction_abs_mm_max -gt [int]$scenario.clamped_correction_limit_abs_mm) {
        throw "Collision physics scenario $($scenario.scenario_id) clamped max correction must not exceed the clamp limit."
    }
    if ([int]$scenario.iterations_run_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require at least one physics iteration."
    }
    if ([int]$scenario.applied_correction_count_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require applied corrections."
    }
    if ([int]$scenario.applied_correction_abs_mm_total_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require applied correction distance."
    }
    if ([int]$scenario.max_applied_correction_abs_mm_min -lt 1) {
        throw "Collision physics scenario $($scenario.scenario_id) must require a max applied correction distance."
    }
}

$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCommand) {
    $cargo = $cargoCommand.Source
} else {
    $cargo = "C:\Users\nkatz\.cargo\bin\cargo.exe"
}
if (-not (Test-Path $cargo)) {
    throw "cargo executable not found."
}

& $cargo test collision_perf_smoke
if ($LASTEXITCODE -ne 0) {
    throw "cargo test collision_perf_smoke failed with exit code $LASTEXITCODE."
}

Write-Host "collision_physics_perf_smoke status=ok scenarios=$(@($report.scenarios).Count) budget_result=$($report.budget_result)"
