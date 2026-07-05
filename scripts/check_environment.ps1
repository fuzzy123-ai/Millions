$ErrorActionPreference = "Stop"

function Get-ToolStatus {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [string[]]$VersionArgs = @("--version"),
        [string[]]$CandidatePaths = @(),
        [bool]$Required = $true
    )

    $commandPath = ""
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($command) {
        $commandPath = $command.Source
        $source = "PATH"
    }

    if (-not $commandPath) {
        foreach ($candidate in $CandidatePaths) {
            if ([string]::IsNullOrWhiteSpace($candidate)) {
                continue
            }

            $expanded = [Environment]::ExpandEnvironmentVariables($candidate)
            $match = Get-ChildItem -Path $expanded -ErrorAction SilentlyContinue |
                Where-Object { -not $_.PSIsContainer } |
                Select-Object -First 1
            if ($match) {
                $commandPath = $match.FullName
                $source = "candidate"
                break
            }
        }
    }

    if (-not $commandPath) {
        return [pscustomobject]@{
            Tool = $Name
            Found = $false
            Required = $Required
            Source = ""
            Path = ""
            Version = ""
        }
    }

    $versionText = ""
    try {
        $versionText = (& $commandPath @VersionArgs 2>$null | Select-Object -First 1)
    } catch {
        $versionText = "version check failed"
    }

    return [pscustomobject]@{
        Tool = $Name
        Found = $true
        Required = $Required
        Source = $source
        Path = $commandPath
        Version = $versionText
    }
}

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$wingetGodotPackage = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe"
$godotEnvPath = $env:GODOT_BIN

$tools = @(
    (Get-ToolStatus -Name "rustc" -CandidatePaths @((Join-Path $cargoBin "rustc.exe")) -Required $true),
    (Get-ToolStatus -Name "cargo" -CandidatePaths @((Join-Path $cargoBin "cargo.exe")) -Required $true),
    (Get-ToolStatus -Name "godot" -CandidatePaths @(
        $godotEnvPath,
        (Join-Path $wingetGodotPackage "Godot*_console.exe"),
        (Join-Path $wingetGodotPackage "Godot*.exe")
    ) -Required $true),
    (Get-ToolStatus -Name "dotnet" -Required $false),
    (Get-ToolStatus -Name "pwsh" -Required $false),
    (Get-ToolStatus -Name "powershell" -VersionArgs @("-NoProfile", "-Command", "`$PSVersionTable.PSVersion.ToString()") -Required $true)
)

$tools | Format-Table Tool, Found, Required, Source, Version, Path -AutoSize -Wrap

$missingRequired = @($tools | Where-Object { $_.Required -and -not $_.Found })
if ($missingRequired.Count -gt 0) {
    Write-Host ""
    Write-Host "Missing required tools for full implementation:" -ForegroundColor Yellow
    $missingRequired | ForEach-Object { Write-Host " - $($_.Tool)" -ForegroundColor Yellow }
    Write-Host ""
    Write-Host "No tools were installed. See docs/runbooks/local-toolchain-setup.md."
    exit 2
}

Write-Host "All required foundation tools are available." -ForegroundColor Green
