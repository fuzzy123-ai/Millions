param(
    [switch]$SkipGodot
)

$ErrorActionPreference = "Stop"

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

function Get-GodotConsolePath {
    $godotCommand = Get-Command godot -ErrorAction SilentlyContinue
    if ($godotCommand) {
        return $godotCommand.Source
    }

    $godotRoot = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe"
    $godot = Get-ChildItem -Path $godotRoot -Filter "Godot*_console.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($godot) {
        return $godot.FullName
    }

    throw "Godot executable not found."
}

$schema = Get-Content tests\soak\soak-scenarios.schema.json -Raw | ConvertFrom-Json
$catalog = Get-Content tests\soak\soak-scenarios.json -Raw | ConvertFrom-Json

if ($catalog.scenarios.Count -eq 0) {
    throw "No soak scenarios are defined."
}

$liveScenarios = 0
foreach ($scenario in @($catalog.scenarios)) {
    foreach ($field in @($schema.required_fields)) {
        if (-not ($scenario.PSObject.Properties.Name -contains $field)) {
            throw "Soak scenario $($scenario.scenario_id) missing required field: $field"
        }
    }
    if ($scenario.live_go_required -or $scenario.class -eq "needs_live_go") {
        $liveScenarios += 1
    }
}

$cargo = Get-CargoPath
& $cargo test soak_metrics

if (-not $SkipGodot) {
    $godot = Get-GodotConsolePath
    & $godot --headless --path client\godot --script res://scripts/tests/soak_metrics_check.gd
}

$godotStatus = if ($SkipGodot) { "skipped" } else { "ok" }
Write-Host "soak_smoke status=ok scenarios=$($catalog.scenarios.Count) live_gated=$liveScenarios rust_metrics=ok godot_metrics=$godotStatus"
