$ErrorActionPreference = "Stop"

$configPath = "config\local-multiclient-harness.json"
if (-not (Test-Path $configPath)) {
    throw "Missing harness config: $configPath"
}

$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
if ($config.live_network -or $config.steam_live) {
    throw "Local harness refuses live network or live Steam mode."
}

Write-Host "local_multiclient_harness step=server_smoke"
& powershell -ExecutionPolicy Bypass -File scripts\run_server_smoke.ps1

foreach ($client in @($config.clients)) {
    Write-Host "local_multiclient_harness step=client profile=$($client.profile)"
    foreach ($check in @($client.checks)) {
        switch ($check) {
            "run_godot_fixture_check" {
                & powershell -ExecutionPolicy Bypass -File scripts\run_godot_fixture_check.ps1
            }
            "run_godot_client_adapter_check" {
                & powershell -ExecutionPolicy Bypass -File scripts\run_godot_client_adapter_check.ps1
            }
            "run_godot_snapshot_render_smoke" {
                & powershell -ExecutionPolicy Bypass -File scripts\run_godot_snapshot_render_smoke.ps1
            }
            "run_godot_lobby_facade_check" {
                & powershell -ExecutionPolicy Bypass -File scripts\run_godot_lobby_facade_check.ps1
            }
            default {
                throw "Unknown harness check '$check' for profile '$($client.profile)'"
            }
        }
    }
}

$restartClient = @($config.clients) | Select-Object -First 1
if ($restartClient) {
    Write-Host "local_multiclient_harness step=reconnect_restart profile=$($restartClient.profile) phase=before_restart"
    & powershell -ExecutionPolicy Bypass -File scripts\run_godot_client_adapter_check.ps1
    Write-Host "local_multiclient_harness step=reconnect_restart profile=$($restartClient.profile) phase=after_restart"
    & powershell -ExecutionPolicy Bypass -File scripts\run_godot_client_adapter_check.ps1
    Write-Host "local_multiclient_harness reconnect_restart status=ok profile=$($restartClient.profile)"
}

Write-Host "local_multiclient_harness status=ok clients=$(@($config.clients).Count) mode=$($config.mode)"
