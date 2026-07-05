param(
    [string]$ConfigPath = "config\local-smoke-commands.json",
    [string[]]$Only = @()
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $ConfigPath)) {
    throw "Missing local smoke command config: $ConfigPath"
}

$config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
if ($config.live_network -or $config.steam_live) {
    throw "Local smoke commands refuse live network or live Steam mode."
}

$allowedScripts = @{
    "run_server_smoke.ps1" = $true
    "run_godot_fixture_check.ps1" = $true
    "run_godot_client_adapter_check.ps1" = $true
    "run_godot_snapshot_render_smoke.ps1" = $true
    "run_godot_render_batch_check.ps1" = $true
    "run_godot_render_stress_smoke.ps1" = $true
    "run_godot_lobby_facade_check.ps1" = $true
    "run_local_multiclient_harness.ps1" = $true
}

$ran = 0
foreach ($command in @($config.commands)) {
    if (-not $command.enabled) {
        continue
    }
    if ($Only.Count -gt 0 -and $Only -notcontains $command.id) {
        continue
    }
    if (-not $allowedScripts.ContainsKey([string]$command.script)) {
        throw "Unknown or disallowed local smoke script: $($command.script)"
    }

    $scriptPath = Join-Path "scripts" ([string]$command.script)
    if (-not (Test-Path $scriptPath)) {
        throw "Missing local smoke script: $scriptPath"
    }

    Write-Host "local_smoke_commands step=$($command.id) script=$($command.script)"
    & powershell -ExecutionPolicy Bypass -File $scriptPath
    $ran += 1
}

if ($ran -eq 0) {
    throw "No local smoke commands ran."
}

Write-Host "local_smoke_commands status=ok commands=$ran mode=$($config.mode)"
