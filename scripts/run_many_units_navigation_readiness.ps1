$ErrorActionPreference = "Stop"

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Script
    )

    Write-Host "readiness_step start name=$Name"
    & $Script
    Write-Host "readiness_step status=ok name=$Name"
}

function Assert-JsonField {
    param(
        [Parameter(Mandatory = $true)]$Object,
        [Parameter(Mandatory = $true)][string]$Field,
        [Parameter(Mandatory = $true)][string]$Expected
    )

    $actual = [string]$Object.$Field
    if ($actual -ne $Expected) {
        throw "JSON field $Field mismatch. Got '$actual', expected '$Expected'."
    }
}

function Invoke-NativeCommand {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Native command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
    }
}

function Resolve-Godot {
    $godotRoot = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe"
    $godot = Get-ChildItem -Path $godotRoot -Filter "Godot*_console.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $godot) {
        $godot = Get-ChildItem -Path $godotRoot -Filter "Godot*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    }
    if ($godot) {
        return $godot.FullName
    }

    $godotCommand = Get-Command godot -ErrorAction SilentlyContinue
    if ($godotCommand) {
        return $godotCommand.Source
    }

    throw "Godot executable not found."
}

Invoke-Step "readiness_json" {
    $readiness = Get-Content "tests\perf\many-units-navigation-readiness.json" -Raw | ConvertFrom-Json
    Assert-JsonField $readiness "slice" "NAV-05"
    Assert-JsonField $readiness "status" "local_readiness_smoke"
    if (@($readiness.required_components).Count -lt 5) {
        throw "Readiness JSON must list all required components."
    }
}

Invoke-Step "movement_scale_json" {
    $movement = Get-Content "tests\perf\movement-scale-scenarios.json" -Raw | ConvertFrom-Json
    Assert-JsonField $movement "slice" "NAV-02"
    Assert-JsonField $movement "status" "scenario_definition_only"
    Assert-JsonField $movement "map_checksum" "sum16:3e03"
    $flowScenario = @($movement.scenarios | Where-Object { $_.id -eq "nav_shared_objective_10k_flow_field" }) | Select-Object -First 1
    if (-not $flowScenario) {
        throw "Movement scale scenarios must include nav_shared_objective_10k_flow_field."
    }
    if ([int]$flowScenario.flow_field_build_count_min -lt 1) {
        throw "Flow-field movement scenario must require a built flow field."
    }
    if ([int]$flowScenario.flow_field_cache_request_count_min -lt [int]$flowScenario.tick_count) {
        throw "Flow-field movement scenario must require one cache request per tick."
    }
    if ([int]$flowScenario.flow_field_cache_hit_count_min -lt ([int]$flowScenario.tick_count - 1)) {
        throw "Flow-field movement scenario must require cache hits after the first build."
    }
    if ([int]$flowScenario.flow_field_cache_eviction_count_max -ne 0) {
        throw "Flow-field movement scenario must not evict during the bounded smoke scenario."
    }
    if ([int]$flowScenario.flow_field_cache_entry_count_max -lt 1) {
        throw "Flow-field movement scenario must require a cache entry cap."
    }
    if ([int]$flowScenario.flow_field_cache_entry_count_max -gt 64) {
        throw "Flow-field movement scenario cache entry cap must stay at or below 64."
    }
    if ([int]$flowScenario.flow_field_query_count_min -lt ([int]$flowScenario.entity_count * [int]$flowScenario.tick_count)) {
        throw "Flow-field movement scenario must require one query per entity per tick."
    }
    if ([int]$flowScenario.flow_field_collision_admission_check_count_min -lt (16 * [int]$flowScenario.tick_count)) {
        throw "Flow-field movement scenario must require bounded collision admission checks every tick."
    }
    if ([int]$flowScenario.flow_field_collision_admission_accepted_count_min -lt 1) {
        throw "Flow-field movement scenario must require at least one accepted collision admission."
    }
    if ([int]$flowScenario.flow_field_collision_admission_rejected_count_min -lt 1) {
        throw "Flow-field movement scenario must require at least one rejected collision admission."
    }
    if ([int]$flowScenario.flow_field_collision_resolved_admission_check_count_min -lt (16 * [int]$flowScenario.tick_count)) {
        throw "Flow-field movement scenario must require bounded resolved-admission checks every tick."
    }
    if ([int]$flowScenario.flow_field_collision_resolved_admission_rejected_count_min -lt 1) {
        throw "Flow-field movement scenario must require at least one candidate still overlapping after bounded local resolution."
    }
    if ([int]$flowScenario.flow_field_collision_resolved_admission_iterations_run_count_min -lt 1) {
        throw "Flow-field movement scenario must require bounded local resolution iterations."
    }
    if ([int]$flowScenario.flow_field_collision_resolved_admission_correction_count_min -lt 1) {
        throw "Flow-field movement scenario must require bounded local resolution corrections."
    }
    if ([int]$flowScenario.flow_field_collision_resolved_admission_correction_abs_mm_total_min -lt 1) {
        throw "Flow-field movement scenario must require bounded local resolution correction distance."
    }
    if ([int]$flowScenario.flow_field_collision_resolved_admission_max_correction_abs_mm_min -lt 1) {
        throw "Flow-field movement scenario must require bounded local resolution max correction distance."
    }
    if ([int]$flowScenario.flow_field_collision_movement_probe_count_min -lt (16 * [int]$flowScenario.tick_count)) {
        throw "Flow-field movement scenario must require bounded movement/collision probes every tick."
    }
    if ([int]$flowScenario.flow_field_collision_movement_probe_blocked_count_min -lt 1) {
        throw "Flow-field movement scenario must require at least one blocked movement/collision probe."
    }
    if ([int]$flowScenario.flow_field_collision_movement_applied_delta_count_min -lt 1) {
        throw "Flow-field movement scenario must require at least one movement/collision probe-applied delta."
    }
    if ([int]$flowScenario.flow_field_collision_movement_blocked_delta_count_min -lt 1) {
        throw "Flow-field movement scenario must require at least one movement/collision probe-blocked delta."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_candidate_count_min -lt 1) {
        throw "Flow-field movement scenario must require sampled flow-field movement candidates for local apply physics."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_initial_contact_count_min -lt 1) {
        throw "Flow-field movement scenario must require local apply-physics contact pressure."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_iterations_run_count_min -lt 1) {
        throw "Flow-field movement scenario must require local apply-physics iterations."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_correction_count_min -lt 1) {
        throw "Flow-field movement scenario must require local apply-physics corrections."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_correction_abs_mm_total_min -lt 1) {
        throw "Flow-field movement scenario must require local apply-physics correction distance."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_max_correction_abs_mm_min -lt 1) {
        throw "Flow-field movement scenario must require local apply-physics max correction distance."
    }
    if ([int]$flowScenario.flow_field_collision_apply_physics_synced_position_count_min -lt 1) {
        throw "Flow-field movement scenario must require local apply-physics sample position syncs."
    }
    if ([int]$flowScenario.flow_field_static_obstacle_body_count_min -lt [int]$flowScenario.blocker_cell_count) {
        throw "Flow-field movement scenario must require static obstacle collision bodies for blocker cells."
    }
    if ([int]$flowScenario.flow_field_unreachable_count_max -ne 0) {
        throw "Flow-field movement scenario must require zero unreachable local queries."
    }
}

Invoke-Step "readability_report_json" {
    $readability = Get-Content "tests\perf\movement-readability-stress-report.json" -Raw | ConvertFrom-Json
    Assert-JsonField $readability "slice" "NAV-03"
    Assert-JsonField $readability "status" "informational"
    Assert-JsonField $readability "budget_result" "blocked"
    if ([int]$readability.entity_proxy_count -ne 1000) {
        throw "Readability proxy count mismatch."
    }
}

Invoke-Step "mapdata_checksum_json" {
    $checksum = Get-Content "tests\fixtures\mapdata_v0_local_contract.checksum.json" -Raw | ConvertFrom-Json
    Assert-JsonField $checksum "algorithm" "sum16_bytes"
    Assert-JsonField $checksum "expected_checksum" "sum16:3e03"
}

Invoke-Step "rust_movement_scale" {
    Invoke-NativeCommand "cargo" @("test", "movement_scale")
}

Invoke-Step "rust_map_data" {
    Invoke-NativeCommand "cargo" @("test", "map_data")
}

$godotPath = Resolve-Godot
Invoke-Step "godot_mapdata_fixture" {
    Invoke-NativeCommand $godotPath @("--headless", "--path", "client\godot", "--script", "res://scripts/tests/map_data_fixture_check.gd")
}

Invoke-Step "godot_movement_readability" {
    Invoke-NativeCommand $godotPath @("--headless", "--path", "client\godot", "--script", "res://scripts/tests/movement_readability_stress_check.gd")
}

Invoke-Step "plan_validation" {
    powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1
}

Invoke-Step "foundation_check" {
    powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1
}

Write-Host "many_units_navigation_readiness status=ok rust_movement=ok rust_mapdata=ok godot_mapdata=ok godot_readability=ok plan=ok foundation=ok"
