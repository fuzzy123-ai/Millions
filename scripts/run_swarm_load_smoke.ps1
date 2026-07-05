$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\swarm-load-smoke-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing swarm load smoke report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GSWARM-03") {
    throw "Swarm load report slice mismatch: $($report.slice)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Swarm load report must remain informational/blocked until measured p95 evidence exists."
}
if ([int]$report.active_zombie_count -ne 1000) {
    throw "Swarm load report must cover 1,000 active zombies."
}
$elapsedFields = @(
    "local_smoke_total_elapsed_us_min",
    "spawn_ticks_elapsed_us_min",
    "behavior_elapsed_us_min",
    "movement_preview_elapsed_us_min",
    "movement_tick_elapsed_us_min",
    "batch_movement_tick_elapsed_us_min",
    "configured_movement_tick_elapsed_us_min",
    "configured_batch_movement_tick_elapsed_us_min",
    "configured_movement_loop_elapsed_us_min",
    "static_obstacle_movement_elapsed_us_min",
    "snapshot_elapsed_us_min",
    "collision_diagnostics_elapsed_us_min"
)
foreach ($field in $elapsedFields) {
    if ([int64]$report.$field -lt 1) {
        throw "Swarm load report must require positive local elapsed-time evidence for $field."
    }
}
if ([int]$report.behavior_sample_count -ne [int]$report.active_zombie_count) {
    throw "Behavior sample count must match active zombie count."
}
if ([int]$report.collision_body_count -ne [int]$report.active_zombie_count) {
    throw "Collision body count must match active zombie count."
}
if ([int]$report.collision_contact_count_min -lt 1) {
    throw "Swarm load report must require at least one collision-prep contact."
}
if ([int]$report.collision_admission_check_count -lt 1) {
    throw "Swarm load report must require collision admission checks."
}
if ([int]$report.collision_admission_rejected_count_min -lt 1) {
    throw "Swarm load report must require at least one collision admission reject."
}
if ([int]$report.collision_resolved_admission_check_count -lt 1) {
    throw "Swarm load report must require resolved-admission checks."
}
if ([int]$report.collision_resolved_admission_check_count -gt 4) {
    throw "Swarm load report resolved-admission sample must stay bounded."
}
if ([int]$report.collision_resolved_admission_rejected_count_min -lt 1) {
    throw "Swarm load report must require at least one still-overlapping resolved admission."
}
if ([int]$report.collision_resolved_admission_iterations_run_count_min -lt 1) {
    throw "Swarm load report must require bounded resolved-admission iterations."
}
if ([int]$report.collision_resolved_admission_correction_count_min -lt 1) {
    throw "Swarm load report must require bounded resolved-admission corrections."
}
if ([int]$report.collision_movement_probe_count -lt 1) {
    throw "Swarm load report must require movement/collision probes."
}
if ([int]$report.collision_movement_probe_count -gt 4) {
    throw "Swarm load report movement/collision probe sample must stay bounded."
}
if ([int]$report.collision_movement_probe_blocked_count_min -lt 1) {
    throw "Swarm load report must require at least one blocked movement/collision probe."
}
if ([int]$report.collision_batch_movement_probe_count -ne [int]$report.collision_movement_probe_count) {
    throw "Swarm load report batch movement probe count must match movement probe count."
}
if ([int]$report.collision_batch_movement_probe_unknown_body_count_max -ne 0) {
    throw "Swarm load report batch movement probes must not reference unknown bodies."
}
if ([int]$report.collision_batch_movement_probe_iterations_run_count_min -lt 1) {
    throw "Swarm load report must require batch movement probe resolution iterations."
}
if ([int]$report.collision_batch_movement_probe_correction_count_min -lt 1) {
    throw "Swarm load report must require batch movement probe corrections."
}
if ([int64]$report.collision_batch_movement_probe_correction_abs_mm_total_min -lt 1) {
    throw "Swarm load report must require batch movement probe correction distance."
}
if ([int]$report.collision_batch_movement_probe_max_correction_abs_mm_min -lt 1) {
    throw "Swarm load report must require batch movement probe max correction distance."
}
if ([int]$report.movement_preview_sample_count -ne [int]$report.collision_movement_probe_count) {
    throw "Swarm load report movement preview sample count must match movement/collision probe count."
}
if ([int]$report.movement_preview_flow_field_build_count_min -lt 1) {
    throw "Swarm load report must require at least one movement-preview flow-field build."
}
if ([int]$report.movement_preview_flow_field_build_count_max -gt [int]$report.movement_preview_sample_count) {
    throw "Swarm load report movement-preview flow-field builds must stay bounded by sample count."
}
if ([int]$report.movement_preview_flow_field_query_count -ne [int]$report.movement_preview_sample_count) {
    throw "Swarm load report movement-preview flow-field query count must match sample count."
}
if ([int]$report.movement_preview_flow_field_unreachable_count_max -ne 0) {
    throw "Swarm load report movement-preview flow-field candidates must remain reachable."
}
if ([int]$report.movement_preview_physics_candidate_count_max -gt [int]$report.movement_preview_sample_count) {
    throw "Swarm load report movement-preview physics candidates must stay bounded by sample count."
}
if ([int]$report.movement_preview_physics_initial_contact_count_min -lt 1) {
    throw "Swarm load report must require local movement-preview physics contact pressure."
}
if ([int]$report.movement_preview_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require bounded local movement-preview physics iterations."
}
if ([int]$report.movement_preview_physics_applied_correction_count_min -lt 1) {
    throw "Swarm load report must require local movement-preview physics corrections."
}
if ([int]$report.movement_preview_physics_applied_correction_abs_mm_total_min -lt 1) {
    throw "Swarm load report must require local movement-preview physics correction distance."
}
if ([int]$report.movement_preview_physics_max_applied_correction_abs_mm_min -lt 1) {
    throw "Swarm load report must require local movement-preview max physics correction distance."
}
if ([int]$report.movement_preview_blocked_delta_count_min -lt 1) {
    throw "Swarm load report must require at least one blocked movement preview delta."
}
if ([int]$report.movement_apply_sample_count -ne [int]$report.collision_movement_probe_count) {
    throw "Swarm load report movement apply sample count must match movement/collision probe count."
}
if ([int]$report.movement_apply_flow_field_build_count_min -lt 1) {
    throw "Swarm load report must require at least one movement-apply flow-field build."
}
if ([int]$report.movement_apply_flow_field_build_count_max -gt [int]$report.movement_apply_sample_count) {
    throw "Swarm load report movement-apply flow-field builds must stay bounded by sample count."
}
if ([int]$report.movement_apply_flow_field_cache_request_count -ne [int]$report.movement_apply_sample_count) {
    throw "Swarm load report movement-apply flow-field cache requests must match movement sample count."
}
if ([int]$report.movement_apply_flow_field_cache_hit_count_min -lt 1) {
    throw "Swarm load report must require at least one movement-apply flow-field cache hit."
}
if ([int]$report.movement_apply_flow_field_cache_eviction_count_max -ne 0) {
    throw "Swarm load report movement-apply flow-field cache must not evict during bounded smoke."
}
if ([int]$report.movement_apply_flow_field_cache_entry_count_max -lt 1) {
    throw "Swarm load report movement-apply flow-field cache entry cap must be present."
}
if ([int]$report.movement_apply_flow_field_cache_entry_count_max -gt 32) {
    throw "Swarm load report movement-apply flow-field cache entry cap must stay at or below 32."
}
if ([int]$report.movement_apply_flow_field_query_count -ne [int]$report.movement_apply_sample_count) {
    throw "Swarm load report movement-apply flow-field query count must match sample count."
}
if ([int]$report.movement_apply_flow_field_unreachable_count_max -ne 0) {
    throw "Swarm load report movement-apply flow-field candidates must remain reachable."
}
if ([int]$report.movement_apply_physics_candidate_count_max -gt [int]$report.movement_apply_sample_count) {
    throw "Swarm load report movement-apply physics candidates must stay bounded by sample count."
}
if ([int]$report.movement_apply_physics_initial_contact_count_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply physics contact pressure."
}
if ([int]$report.movement_apply_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply physics iterations."
}
if ([int]$report.movement_apply_physics_applied_correction_count_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply physics corrections."
}
if ([int]$report.movement_apply_physics_applied_correction_abs_mm_total_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply physics correction distance."
}
if ([int]$report.movement_apply_physics_max_applied_correction_abs_mm_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply max physics correction distance."
}
if ($null -ne $report.movement_apply_movement_probe_correction_limit_abs_mm) {
    throw "Swarm load report default movement-apply probe must remain unclamped."
}
if ([int]$report.movement_apply_movement_probe_clamped_correction_count_max -ne 0) {
    throw "Swarm load report default movement-apply probe must not clamp corrections."
}
if ([int]$report.movement_apply_physics_clamped_correction_count_max -ne 0) {
    throw "Swarm load report default movement-apply physics must remain unclamped."
}
if ([int]$report.movement_apply_physics_synced_position_count_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply physics position syncs."
}
if ([int]$report.movement_apply_physics_sample_synced_position_count_min -lt 1) {
    throw "Swarm load report must require opt-in movement-apply sample final positions to be synced after physics."
}
if ([int]$report.clamped_movement_apply_sample_count -ne [int]$report.movement_apply_sample_count) {
    throw "Swarm load report clamped movement-apply sample count must match movement apply sample count."
}
if ([int]$report.clamped_movement_apply_movement_probe_correction_limit_abs_mm -ne 50) {
    throw "Swarm load report clamped movement-apply probe limit must stay at 50mm."
}
if ([int]$report.clamped_movement_apply_movement_probe_clamped_correction_count_min -lt 1) {
    throw "Swarm load report must require clamped movement-apply probe corrections."
}
if ([int]$report.clamped_movement_apply_physics_correction_limit_abs_mm -ne 50) {
    throw "Swarm load report clamped movement-apply physics limit must stay at 50mm."
}
if ([int]$report.clamped_movement_apply_physics_clamped_correction_count_min -lt 0) {
    throw "Swarm load report clamped movement-apply physics clamp count must be non-negative."
}
if ([int]$report.clamped_movement_apply_physics_max_applied_correction_abs_mm_max -gt [int]$report.clamped_movement_apply_physics_correction_limit_abs_mm) {
    throw "Swarm load report clamped movement-apply max correction must not exceed the clamp limit."
}
if ([int]$report.batch_movement_apply_sample_count -ne [int]$report.movement_apply_sample_count) {
    throw "Swarm load report batch movement-apply sample count must match movement apply sample count."
}
if ([int]$report.batch_movement_apply_flow_field_cache_request_count -ne [int]$report.batch_movement_apply_sample_count) {
    throw "Swarm load report batch movement-apply flow-field cache requests must match sample count."
}
if ([int]$report.batch_movement_apply_flow_field_cache_hit_count_min -lt 1) {
    throw "Swarm load report must require at least one batch movement-apply flow-field cache hit."
}
if ([int]$report.batch_movement_apply_flow_field_cache_eviction_count_max -ne 0) {
    throw "Swarm load report batch movement-apply flow-field cache must not evict during bounded smoke."
}
if ([int]$report.batch_movement_apply_flow_field_cache_entry_count_max -gt 32) {
    throw "Swarm load report batch movement-apply flow-field cache entry cap must stay at or below 32."
}
if ([int]$report.batch_movement_apply_applied_delta_count_min -lt 1) {
    throw "Swarm load report must require batch movement-apply to move at least one sampled zombie."
}
if ($null -ne $report.batch_movement_apply_movement_probe_correction_limit_abs_mm) {
    throw "Swarm load report default batch movement-apply probe must remain unclamped."
}
if ([int]$report.batch_movement_apply_movement_probe_clamped_correction_count_max -ne 0) {
    throw "Swarm load report default batch movement-apply probe must not clamp corrections."
}
if ([int]$report.batch_movement_apply_physics_candidate_count_max -gt [int]$report.batch_movement_apply_sample_count) {
    throw "Swarm load report batch movement-apply physics candidates must stay bounded by sample count."
}
if ([int]$report.batch_movement_apply_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require batch movement-apply physics iterations."
}
if ([int]$report.batch_movement_apply_physics_synced_position_count_min -lt 1) {
    throw "Swarm load report must require batch movement-apply physics position syncs."
}
if ([int]$report.batch_movement_apply_physics_sample_synced_position_count_min -lt 1) {
    throw "Swarm load report must require batch movement-apply sample final positions to be synced after physics."
}
if ([int]$report.batch_movement_tick_sample_count -ne [int]$report.batch_movement_apply_sample_count) {
    throw "Swarm load report batch movement tick sample count must match batch movement apply sample count."
}
if ([int]$report.batch_movement_tick_active_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report batch movement tick active count must match active zombie count."
}
if ([int]$report.batch_movement_tick_spawned_count -ne 0) {
    throw "Swarm load report final batch movement tick should not spawn extra capped zombies."
}
if ([int]$report.batch_movement_tick_applied_delta_count_min -lt 1) {
    throw "Swarm load report must require batch movement tick to move at least one sampled zombie."
}
if ([int]$report.batch_movement_tick_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require batch movement tick physics iterations."
}
if ([int]$report.batch_movement_tick_snapshot_entity_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report batch movement tick snapshot entity count must match active zombie count."
}
if ([int64]$report.batch_movement_tick_snapshot_bytes -ne [int64]$report.movement_tick_snapshot_bytes) {
    throw "Swarm load report batch movement tick snapshot bytes must match movement tick full snapshot bytes."
}
if ([int]$report.batch_movement_delta_snapshot_entity_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report batch movement delta snapshot entity count must match active zombie count."
}
if ([int64]$report.batch_movement_delta_snapshot_bytes -ne [int64]$report.movement_delta_snapshot_bytes) {
    throw "Swarm load report batch movement delta snapshot bytes must match movement delta snapshot bytes."
}
if ([int]$report.movement_tick_sample_count -ne [int]$report.movement_apply_sample_count) {
    throw "Swarm load report movement tick sample count must match movement apply sample count."
}
if ([int]$report.movement_tick_active_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report movement tick active count must match active zombie count."
}
if ([int]$report.movement_tick_spawned_count -ne 0) {
    throw "Swarm load report final movement tick should not spawn extra capped zombies."
}
if ([int]$report.movement_tick_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require opt-in movement tick physics iterations."
}
if ([int]$report.configured_movement_tick_sample_count -ne [int]$report.movement_tick_sample_count) {
    throw "Swarm load report configured movement tick sample count must match explicit movement tick sample count."
}
if ([int]$report.configured_movement_tick_active_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report configured movement tick active count must match active zombie count."
}
if ([int]$report.configured_movement_tick_spawned_count -ne 0) {
    throw "Swarm load report configured final movement tick should not spawn extra capped zombies."
}
if ([int]$report.configured_movement_tick_applied_delta_count_min -lt 1) {
    throw "Swarm load report must require configured movement tick applied deltas."
}
if ([int]$report.configured_movement_tick_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require configured movement tick physics iterations."
}
if ([int]$report.configured_batch_movement_tick_sample_count -ne [int]$report.batch_movement_tick_sample_count) {
    throw "Swarm load report configured batch movement tick sample count must match explicit batch movement tick sample count."
}
if ([int]$report.configured_batch_movement_tick_active_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report configured batch movement tick active count must match active zombie count."
}
if ([int]$report.configured_batch_movement_tick_spawned_count -ne 0) {
    throw "Swarm load report configured batch movement tick should not spawn extra capped zombies."
}
if ([int]$report.configured_batch_movement_tick_applied_delta_count_min -lt 1) {
    throw "Swarm load report must require configured batch movement tick to move at least one sampled zombie."
}
if ([int]$report.configured_batch_movement_tick_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require configured batch movement tick physics iterations."
}
if ($report.configured_batch_movement_tick_claim_scope -ne "swarm_batch_movement_apply_opt_in") {
    throw "Swarm load report configured batch movement tick must report the batch movement apply claim scope."
}
if ([int]$report.configured_clamped_movement_tick_sample_count -ne [int]$report.configured_movement_tick_sample_count) {
    throw "Swarm load report configured clamped movement tick sample count must match configured movement tick sample count."
}
if ([int]$report.configured_clamped_movement_tick_movement_probe_correction_limit_abs_mm -ne 50) {
    throw "Swarm load report configured clamped movement tick probe limit must stay at 50mm."
}
if ([int]$report.configured_clamped_movement_tick_movement_probe_clamped_correction_count_min -lt 1) {
    throw "Swarm load report must require configured clamped movement tick probe corrections."
}
if ([int]$report.configured_clamped_movement_tick_physics_correction_limit_abs_mm -ne 50) {
    throw "Swarm load report configured clamped movement tick physics limit must stay at 50mm."
}
if ([int]$report.configured_clamped_movement_tick_physics_clamped_correction_count_min -lt 0) {
    throw "Swarm load report configured clamped movement tick physics clamp count must be non-negative."
}
if ([int]$report.configured_clamped_movement_tick_physics_max_applied_correction_abs_mm_max -gt [int]$report.configured_clamped_movement_tick_physics_correction_limit_abs_mm) {
    throw "Swarm load report configured clamped movement tick max correction must not exceed the clamp limit."
}
if ([int]$report.configured_movement_loop_tick_count -ne 2) {
    throw "Swarm load report configured movement loop must remain a bounded two-tick smoke."
}
if ([int]$report.configured_movement_loop_sample_count -ne ([int]$report.configured_movement_loop_tick_count * 2)) {
    throw "Swarm load report configured movement loop sample count must be two samples per loop tick."
}
if ([int]$report.configured_movement_loop_active_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report configured movement loop active count must match active zombie count."
}
if ([int]$report.configured_movement_loop_spawned_count -ne 0) {
    throw "Swarm load report configured movement loop should not spawn extra capped zombies."
}
if ([int]$report.configured_movement_loop_applied_delta_count_min -lt 1) {
    throw "Swarm load report must require configured movement loop applied deltas."
}
if ([int]$report.configured_movement_loop_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require configured movement loop physics iterations."
}
if ([int]$report.configured_movement_loop_flow_field_cache_request_count -ne [int]$report.configured_movement_loop_sample_count) {
    throw "Swarm load report configured movement loop cache requests must match loop samples."
}
if ([int]$report.configured_movement_loop_flow_field_cache_hit_count_min -lt 1) {
    throw "Swarm load report must require configured movement loop flow-field cache hits."
}
if ([int]$report.configured_movement_loop_flow_field_cache_eviction_count_max -ne 0) {
    throw "Swarm load report configured movement loop cache must not evict during the bounded smoke."
}
if ([int]$report.configured_movement_loop_flow_field_cache_entry_count_max -gt 32) {
    throw "Swarm load report configured movement loop cache entry cap must stay at or below 32."
}
if ([int]$report.configured_movement_loop_moved_entity_count_min -lt 1) {
    throw "Swarm load report must require configured movement loop position changes."
}
if ([int]$report.static_obstacle_count -lt 1) {
    throw "Swarm load report must require static obstacle movement fixtures."
}
if ($report.static_obstacle_source -ne "map_data_import") {
    throw "Swarm load report static obstacles must come from the local map-data import bridge."
}
if ([int]$report.static_obstacle_map_obstacle_count -ne [int]$report.static_obstacle_count) {
    throw "Swarm load report map obstacle count must match static obstacle body count."
}
if ([int]$report.static_obstacle_clearance_mm -le 0) {
    throw "Swarm load report must require positive swarm-radius clearance around static obstacle blocker cells."
}
if ([int]$report.static_obstacle_blocker_cell_count_min -le [int]$report.static_obstacle_count) {
    throw "Swarm load report must require static obstacle extents to expand beyond one flow-field blocker cell each."
}
if ([int]$report.static_obstacle_movement_sample_count -ne [int]$report.movement_tick_sample_count) {
    throw "Swarm load report static-obstacle movement sample count must match movement sample count."
}
if ([int]$report.static_obstacle_movement_flow_field_build_count_min -lt 1) {
    throw "Swarm load report must require static-obstacle movement flow-field builds."
}
if ([int]$report.static_obstacle_movement_applied_delta_count_min -lt 1) {
    throw "Swarm load report must require static-obstacle movement applied deltas."
}
if ([int]$report.static_obstacle_movement_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require static-obstacle movement physics iterations."
}
if ([int]$report.movement_tick_snapshot_entity_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report movement tick snapshot entity count must match active zombie count."
}
if ([int]$report.movement_tick_snapshot_bytes -le 0) {
    throw "Swarm load report must require movement tick snapshot bytes."
}
if ([double]$report.movement_tick_snapshot_bandwidth_kb_s_per_client_min -le 0) {
    throw "Swarm load report must require positive movement tick snapshot bandwidth evidence."
}
if ([int]$report.movement_delta_snapshot_entity_count -ne [int]$report.active_zombie_count) {
    throw "Swarm load report movement delta snapshot entity count must match active zombie count."
}
if ([int]$report.movement_delta_snapshot_removed_count -ne 0) {
    throw "Swarm load report movement delta snapshot should not remove stable visible zombies."
}
if ([int]$report.movement_delta_snapshot_aggregate_far_state_count -ne 0) {
    throw "Swarm load report all-visible movement delta snapshot should not emit aggregate far-state."
}
if ([int]$report.movement_delta_snapshot_bytes -le 0) {
    throw "Swarm load report must require movement delta snapshot bytes."
}
if ([double]$report.movement_delta_snapshot_bandwidth_kb_s_per_client_min -le 0) {
    throw "Swarm load report must require positive movement delta snapshot bandwidth evidence."
}
if ([int]$report.movement_aggregate_delta_snapshot_entity_count_min -lt 1) {
    throw "Swarm load report aggregate movement delta snapshot must include visible moved entities."
}
if ([int]$report.movement_aggregate_delta_snapshot_entity_count_max -ge [int]$report.active_zombie_count) {
    throw "Swarm load report aggregate movement delta snapshot must stay below all-visible entity count."
}
if ([int]$report.movement_aggregate_delta_snapshot_removed_count -ne 0) {
    throw "Swarm load report aggregate movement delta snapshot should not remove stable visible zombies."
}
if ([int]$report.movement_aggregate_delta_snapshot_aggregate_far_state_count_min -lt 1) {
    throw "Swarm load report aggregate movement delta snapshot must include aggregate far-state evidence."
}
if ([int]$report.movement_aggregate_delta_snapshot_bytes_min -le 0) {
    throw "Swarm load report aggregate movement delta snapshot must include positive byte evidence."
}
if ([double]$report.movement_aggregate_delta_snapshot_bandwidth_kb_s_per_client_min -le 0) {
    throw "Swarm load report aggregate movement delta snapshot must include positive bandwidth evidence."
}
if ([int]$report.collision_resolution_contact_count_min -lt 1) {
    throw "Swarm load report must require collision resolution-plan contacts."
}
if ([int]$report.collision_resolution_correction_count_min -lt 1) {
    throw "Swarm load report must require collision resolution-plan corrections."
}
if ([int]$report.collision_physics_iterations_run_min -lt 1) {
    throw "Swarm load report must require at least one local collision physics iteration."
}
if ([int]$report.collision_physics_applied_correction_count_min -lt 1) {
    throw "Swarm load report must require applied local collision corrections."
}
if ([int]$report.collision_physics_applied_correction_abs_mm_total_min -lt 1) {
    throw "Swarm load report must require applied local collision correction distance."
}
if ([int]$report.collision_physics_max_applied_correction_abs_mm_min -lt 1) {
    throw "Swarm load report must require applied local collision max correction distance."
}
if (@($report.non_claims).Count -lt 5) {
    throw "Swarm load report must preserve conservative non-claims."
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

& $cargo test swarm_load_smoke
if ($LASTEXITCODE -ne 0) {
    throw "cargo test swarm_load_smoke failed with exit code $LASTEXITCODE."
}

$godotRoot = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe"
$godot = Get-ChildItem -Path $godotRoot -Filter "Godot*_console.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $godot) {
    $godot = Get-ChildItem -Path $godotRoot -Filter "Godot*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
}
if (-not $godot) {
    $godotCommand = Get-Command godot -ErrorAction SilentlyContinue
    if ($godotCommand) {
        $godotPath = $godotCommand.Source
    }
} else {
    $godotPath = $godot.FullName
}

if (-not $godotPath) {
    throw "Godot executable not found."
}

& $godotPath --headless --path client\godot --script res://scripts/tests/swarm_readability_stress_check.gd
if ($LASTEXITCODE -ne 0) {
    throw "Godot swarm readability stress failed with exit code $LASTEXITCODE."
}

Write-Host "swarm_load_smoke status=ok zombies=$($report.active_zombie_count) behavior=$($report.behavior_sample_count) collision_bodies=$($report.collision_body_count) admission_checks=$($report.collision_admission_check_count) resolved_checks=$($report.collision_resolved_admission_check_count) budget_result=$($report.budget_result)"
