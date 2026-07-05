param(
    [switch]$SkipGodot
)

$ErrorActionPreference = "Stop"

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )

    Write-Host "foundation_matrix step=$Name"
    & $Command
}

function Get-CargoPath {
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargoCommand) {
        return $cargoCommand.Source
    }
    $candidate = "C:\Users\nkatz\.cargo\bin\cargo.exe"
    if (Test-Path $candidate) {
        return $candidate
    }
    throw "cargo executable not found."
}

$cargo = Get-CargoPath

Invoke-Step "validate_plans" { powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1 }
Invoke-Step "check_foundation" { powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1 }
Invoke-Step "cargo_fmt_check" { & $cargo fmt --check }
Invoke-Step "cargo_test" { & $cargo test }
Invoke-Step "server_smoke" { powershell -ExecutionPolicy Bypass -File scripts\run_server_smoke.ps1 }
Invoke-Step "packet_loss_jitter" { powershell -ExecutionPolicy Bypass -File scripts\run_packet_loss_jitter_harness.ps1 }
Invoke-Step "replay_smoke" { powershell -ExecutionPolicy Bypass -File scripts\run_replay_smoke.ps1 }
Invoke-Step "overload_smoke" { powershell -ExecutionPolicy Bypass -File scripts\run_overload_smoke.ps1 }
Invoke-Step "hostile_input_smoke" { powershell -ExecutionPolicy Bypass -File scripts\run_hostile_input_smoke.ps1 }

if ($SkipGodot) {
    Invoke-Step "perf_schema_parse" {
        Get-Content tests\perf\perf-report.schema.json -Raw | ConvertFrom-Json | Out-Null
        Get-Content tests\perf\sample-perf-report-row.json -Raw | ConvertFrom-Json | Out-Null
    }
    Write-Host "foundation_matrix status=ok mode=ci_skip_godot"
    exit 0
}

Invoke-Step "godot_fixture" { powershell -ExecutionPolicy Bypass -File scripts\run_godot_fixture_check.ps1 }
Invoke-Step "godot_client_adapter" { powershell -ExecutionPolicy Bypass -File scripts\run_godot_client_adapter_check.ps1 }
Invoke-Step "godot_snapshot_render" { powershell -ExecutionPolicy Bypass -File scripts\run_godot_snapshot_render_smoke.ps1 }
Invoke-Step "godot_lobby_facade" { powershell -ExecutionPolicy Bypass -File scripts\run_godot_lobby_facade_check.ps1 }
Invoke-Step "perf_smoke" { powershell -ExecutionPolicy Bypass -File scripts\run_perf_smoke.ps1 }
Invoke-Step "local_multiclient" { powershell -ExecutionPolicy Bypass -File scripts\run_local_multiclient_harness.ps1 }

Write-Host "foundation_matrix status=ok mode=local_full"
