$ErrorActionPreference = "Stop"

$requiredPaths = @(
    "Cargo.toml",
    "rust-toolchain.toml",
    "server/Cargo.toml",
    "server/src/lib.rs",
    "server/src/main.rs",
    "client/godot/project.godot",
    "client/godot/README.md",
    "client/godot/scenes/app/app_root.tscn",
    "client/godot/scripts/autoload/Online.gd",
    "client/godot/scripts/autoload/ClientLog.gd",
    "client/godot/scripts/autoload/PerfLedger.gd",
    "client/godot/scripts/net/ServerConnection.gd",
    "protocol/README.md",
    "protocol/fixtures/README.md",
    "docs/architecture/foundation-decisions.md",
    "docs/architecture/godot-scene-node-contract.md",
    "docs/plans/planning-index.json",
    "docs/plans/millions-plan.json",
    "docs/protocol/protocol-v0.md",
    "docs/perf/performance-baseline.md",
    "docs/runbooks/local-toolchain-setup.md",
    "scripts/validate_plans.ps1",
    "scripts/check_environment.ps1"
)

$missing = @()
foreach ($path in $requiredPaths) {
    if (-not (Test-Path $path)) {
        $missing += $path
    }
}

if ($missing.Count -gt 0) {
    Write-Host "Foundation check failed. Missing paths:" -ForegroundColor Red
    $missing | ForEach-Object { Write-Host " - $_" -ForegroundColor Red }
    exit 1
}

& powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1
Write-Host "Foundation check passed." -ForegroundColor Green
