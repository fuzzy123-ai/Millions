$ErrorActionPreference = "Stop"

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

& $godotPath --headless --path client\godot --script res://scripts/tests/gcore_intent_check.gd
