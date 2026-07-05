$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\flow-field-smoke-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing flow-field smoke report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "FLOW-FIELD-SMOKE") {
    throw "Flow-field report slice mismatch: $($report.slice)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Flow-field report must remain informational/blocked until measured p95 evidence exists."
}
if ($report.claim_scope -ne "local_smoke_only") {
    throw "Flow-field report must keep claim_scope local_smoke_only."
}
if ($report.scenario_id -ne "nav_shared_objective_10k_flow_field") {
    throw "Flow-field report must target nav_shared_objective_10k_flow_field."
}
if ([int]$report.entity_count -ne 10000) {
    throw "Flow-field report must cover 10,000 entities."
}
if ([int]$report.flow_field_build_count_min -lt 1) {
    throw "Flow-field report must require a built flow field."
}
if ([int]$report.flow_field_cache_request_count_min -lt [int]$report.tick_count) {
    throw "Flow-field report must require one cache request per tick."
}
if ([int]$report.flow_field_cache_hit_count_min -lt ([int]$report.tick_count - 1)) {
    throw "Flow-field report must require cache hits after the first build."
}
if ([int]$report.flow_field_cache_eviction_count_max -ne 0) {
    throw "Flow-field report must not evict during the bounded smoke scenario."
}
if ([int]$report.flow_field_cache_entry_count_max -lt 1) {
    throw "Flow-field report must require a cache entry cap."
}
if ([int]$report.flow_field_cache_entry_count_max -gt 64) {
    throw "Flow-field report cache entry cap must stay at or below 64."
}
if ([int]$report.flow_field_query_count_min -lt ([int]$report.entity_count * [int]$report.tick_count)) {
    throw "Flow-field report must require one query per entity per tick."
}
if ([int]$report.flow_field_collision_admission_check_count_min -lt (16 * [int]$report.tick_count)) {
    throw "Flow-field report must require bounded collision admission checks every tick."
}
if ([int]$report.flow_field_collision_admission_accepted_count_min -lt 1) {
    throw "Flow-field report must require at least one accepted collision admission."
}
if ([int]$report.flow_field_collision_admission_rejected_count_min -lt 1) {
    throw "Flow-field report must require at least one rejected collision admission."
}
if ([int]$report.flow_field_collision_resolved_admission_check_count_min -lt (16 * [int]$report.tick_count)) {
    throw "Flow-field report must require bounded resolved-admission checks every tick."
}
if ([int]$report.flow_field_collision_resolved_admission_rejected_count_min -lt 1) {
    throw "Flow-field report must require at least one candidate still overlapping after bounded local resolution."
}
if ([int]$report.flow_field_collision_resolved_admission_iterations_run_count_min -lt 1) {
    throw "Flow-field report must require bounded local resolution iterations."
}
if ([int]$report.flow_field_collision_resolved_admission_correction_count_min -lt 1) {
    throw "Flow-field report must require bounded local resolution corrections."
}
if ([int]$report.flow_field_collision_resolved_admission_correction_abs_mm_total_min -lt 1) {
    throw "Flow-field report must require bounded local resolution correction distance."
}
if ([int]$report.flow_field_collision_resolved_admission_max_correction_abs_mm_min -lt 1) {
    throw "Flow-field report must require bounded local resolution max correction distance."
}
if ([int]$report.flow_field_collision_movement_probe_count_min -lt (16 * [int]$report.tick_count)) {
    throw "Flow-field report must require bounded movement/collision probes every tick."
}
if ([int]$report.flow_field_collision_movement_probe_blocked_count_min -lt 1) {
    throw "Flow-field report must require at least one blocked movement/collision probe."
}
if ([int]$report.flow_field_collision_movement_applied_delta_count_min -lt 1) {
    throw "Flow-field report must require at least one movement/collision probe-applied delta."
}
if ([int]$report.flow_field_collision_movement_blocked_delta_count_min -lt 1) {
    throw "Flow-field report must require at least one movement/collision probe-blocked delta."
}
if ([int]$report.flow_field_collision_apply_physics_candidate_count_min -lt 1) {
    throw "Flow-field report must require sampled flow-field movement candidates for local apply physics."
}
if ([int]$report.flow_field_collision_apply_physics_initial_contact_count_min -lt 1) {
    throw "Flow-field report must require local apply-physics contact pressure."
}
if ([int]$report.flow_field_collision_apply_physics_iterations_run_count_min -lt 1) {
    throw "Flow-field report must require local apply-physics iterations."
}
if ([int]$report.flow_field_collision_apply_physics_correction_count_min -lt 1) {
    throw "Flow-field report must require local apply-physics corrections."
}
if ([int]$report.flow_field_collision_apply_physics_correction_abs_mm_total_min -lt 1) {
    throw "Flow-field report must require local apply-physics correction distance."
}
if ([int]$report.flow_field_collision_apply_physics_max_correction_abs_mm_min -lt 1) {
    throw "Flow-field report must require local apply-physics max correction distance."
}
if ([int]$report.flow_field_collision_apply_physics_synced_position_count_min -lt 1) {
    throw "Flow-field report must require local apply-physics sample position syncs."
}
if ([int]$report.flow_field_static_obstacle_body_count_min -lt [int]$report.blocker_cell_count) {
    throw "Flow-field report must require static obstacle collision bodies for blocker cells."
}
if ([int]$report.flow_field_visited_cell_count_min -lt 1) {
    throw "Flow-field report must require visited flow-field cells."
}
if ([int]$report.flow_field_unreachable_count_max -ne 0) {
    throw "Flow-field report must require zero unreachable flow-field queries in the local smoke."
}
if (@($report.non_claims).Count -lt 5) {
    throw "Flow-field report must preserve conservative non-claims."
}

$movement = Get-Content -LiteralPath "tests\perf\movement-scale-scenarios.json" -Raw | ConvertFrom-Json
$flowScenario = @($movement.scenarios | Where-Object { $_.id -eq "nav_shared_objective_10k_flow_field" }) | Select-Object -First 1
if (-not $flowScenario) {
    throw "Movement scale scenarios must include nav_shared_objective_10k_flow_field."
}
if ([int]$flowScenario.flow_field_build_count_min -lt 1) {
    throw "Movement scale flow scenario must require flow_field_build_count_min."
}
if ([int]$flowScenario.flow_field_cache_request_count_min -lt [int]$flowScenario.tick_count) {
    throw "Movement scale flow scenario must require one cache request per tick."
}
if ([int]$flowScenario.flow_field_cache_hit_count_min -lt ([int]$flowScenario.tick_count - 1)) {
    throw "Movement scale flow scenario must require cache hits after the first build."
}
if ([int]$flowScenario.flow_field_cache_eviction_count_max -ne 0) {
    throw "Movement scale flow scenario must not evict during the bounded smoke scenario."
}
if ([int]$flowScenario.flow_field_cache_entry_count_max -lt 1) {
    throw "Movement scale flow scenario must require a cache entry cap."
}
if ([int]$flowScenario.flow_field_cache_entry_count_max -gt 64) {
    throw "Movement scale flow scenario cache entry cap must stay at or below 64."
}
if ([int]$flowScenario.flow_field_query_count_min -lt ([int]$flowScenario.entity_count * [int]$flowScenario.tick_count)) {
    throw "Movement scale flow scenario must require one query per entity per tick."
}
if ([int]$flowScenario.flow_field_collision_admission_check_count_min -lt (16 * [int]$flowScenario.tick_count)) {
    throw "Movement scale flow scenario must require bounded collision admission checks every tick."
}
if ([int]$flowScenario.flow_field_collision_admission_accepted_count_min -lt 1) {
    throw "Movement scale flow scenario must require at least one accepted collision admission."
}
if ([int]$flowScenario.flow_field_collision_admission_rejected_count_min -lt 1) {
    throw "Movement scale flow scenario must require at least one rejected collision admission."
}
if ([int]$flowScenario.flow_field_collision_resolved_admission_check_count_min -lt (16 * [int]$flowScenario.tick_count)) {
    throw "Movement scale flow scenario must require bounded resolved-admission checks every tick."
}
if ([int]$flowScenario.flow_field_collision_resolved_admission_rejected_count_min -lt 1) {
    throw "Movement scale flow scenario must require at least one candidate still overlapping after bounded local resolution."
}
if ([int]$flowScenario.flow_field_collision_resolved_admission_iterations_run_count_min -lt 1) {
    throw "Movement scale flow scenario must require bounded local resolution iterations."
}
if ([int]$flowScenario.flow_field_collision_resolved_admission_correction_count_min -lt 1) {
    throw "Movement scale flow scenario must require bounded local resolution corrections."
}
if ([int]$flowScenario.flow_field_collision_resolved_admission_correction_abs_mm_total_min -lt 1) {
    throw "Movement scale flow scenario must require bounded local resolution correction distance."
}
if ([int]$flowScenario.flow_field_collision_resolved_admission_max_correction_abs_mm_min -lt 1) {
    throw "Movement scale flow scenario must require bounded local resolution max correction distance."
}
if ([int]$flowScenario.flow_field_collision_movement_probe_count_min -lt (16 * [int]$flowScenario.tick_count)) {
    throw "Movement scale flow scenario must require bounded movement/collision probes every tick."
}
if ([int]$flowScenario.flow_field_collision_movement_probe_blocked_count_min -lt 1) {
    throw "Movement scale flow scenario must require at least one blocked movement/collision probe."
}
if ([int]$flowScenario.flow_field_collision_movement_applied_delta_count_min -lt 1) {
    throw "Movement scale flow scenario must require at least one movement/collision probe-applied delta."
}
if ([int]$flowScenario.flow_field_collision_movement_blocked_delta_count_min -lt 1) {
    throw "Movement scale flow scenario must require at least one movement/collision probe-blocked delta."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_candidate_count_min -lt 1) {
    throw "Movement scale flow scenario must require sampled flow-field movement candidates for local apply physics."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_initial_contact_count_min -lt 1) {
    throw "Movement scale flow scenario must require local apply-physics contact pressure."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_iterations_run_count_min -lt 1) {
    throw "Movement scale flow scenario must require local apply-physics iterations."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_correction_count_min -lt 1) {
    throw "Movement scale flow scenario must require local apply-physics corrections."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_correction_abs_mm_total_min -lt 1) {
    throw "Movement scale flow scenario must require local apply-physics correction distance."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_max_correction_abs_mm_min -lt 1) {
    throw "Movement scale flow scenario must require local apply-physics max correction distance."
}
if ([int]$flowScenario.flow_field_collision_apply_physics_synced_position_count_min -lt 1) {
    throw "Movement scale flow scenario must require local apply-physics sample position syncs."
}
if ([int]$flowScenario.flow_field_static_obstacle_body_count_min -lt [int]$flowScenario.blocker_cell_count) {
    throw "Movement scale flow scenario must require static obstacle collision bodies for blocker cells."
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

& $cargo test flow_field
if ($LASTEXITCODE -ne 0) {
    throw "cargo test flow_field failed with exit code $LASTEXITCODE."
}

Write-Host "flow_field_smoke status=ok scenario=$($report.scenario_id) entities=$($report.entity_count) budget_result=$($report.budget_result)"
