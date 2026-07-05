$ErrorActionPreference = "Stop"

function Fail {
    param([Parameter(Mandatory = $true)][string]$Message)
    Write-Host "Plan validation failed: $Message" -ForegroundColor Red
    exit 1
}

function Read-JsonFile {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (-not (Test-Path $Path)) {
        Fail "Missing JSON file: $Path"
    }
    try {
        return Get-Content $Path -Raw | ConvertFrom-Json
    } catch {
        Fail "Invalid JSON in $Path. $($_.Exception.Message)"
    }
}

$plansDir = "docs/plans"
if (-not (Test-Path $plansDir)) {
    Fail "Missing docs/plans directory."
}

$nonJsonPlanFiles = @(
    Get-ChildItem $plansDir -File |
        Where-Object { $_.Extension.ToLowerInvariant() -ne ".json" }
)
if ($nonJsonPlanFiles.Count -gt 0) {
    $names = ($nonJsonPlanFiles | ForEach-Object { $_.FullName }) -join ", "
    Fail "Planning files must be JSON only. Non-JSON files found: $names"
}

$markdownPlanFiles = @(
    Get-ChildItem $plansDir -File -Include *.md,*.markdown
)
if ($markdownPlanFiles.Count -gt 0) {
    $names = ($markdownPlanFiles | ForEach-Object { $_.FullName }) -join ", "
    Fail "Markdown planning files are forbidden under docs/plans: $names"
}

$index = Read-JsonFile "docs/plans/planning-index.json"
if ($index.source_of_truth -ne "json") {
    Fail "planning-index.json must declare source_of_truth=json."
}

$canonicalPath = [string]$index.canonical_plan
$plan = Read-JsonFile $canonicalPath
if ($plan.source_of_truth -ne "json") {
    Fail "Canonical plan must declare source_of_truth=json."
}

$roadmaps = @($plan.roadmaps)
if ($roadmaps.Count -eq 0) {
    Fail "Canonical plan has no roadmaps."
}

$roadmapCodes = @{}
$sliceIds = @{}
foreach ($roadmap in $roadmaps) {
    $code = [string]$roadmap.code
    if ([string]::IsNullOrWhiteSpace($code)) {
        Fail "Roadmap without code."
    }
    if ($roadmapCodes.ContainsKey($code)) {
        Fail "Duplicate roadmap code: $code"
    }
    $roadmapCodes[$code] = $true

    foreach ($slice in @($roadmap.slices)) {
        $sliceId = [string]$slice.id
        if ([string]::IsNullOrWhiteSpace($sliceId)) {
            Fail "Roadmap $code has slice without id."
        }
        if (-not $sliceId.StartsWith("$code-")) {
            Fail "Slice $sliceId does not match roadmap code prefix $code."
        }
        if ($sliceIds.ContainsKey($sliceId)) {
            Fail "Duplicate slice id: $sliceId"
        }
        $sliceIds[$sliceId] = $true
    }
}

$gates = @($plan.risk_register.gates)
$gateIds = @{}
foreach ($gate in $gates) {
    $gateId = [string]$gate.id
    if ([string]::IsNullOrWhiteSpace($gateId)) {
        Fail "Gate without id."
    }
    if ($gateIds.ContainsKey($gateId)) {
        Fail "Duplicate gate id: $gateId"
    }
    $gateIds[$gateId] = $true

    $nextSafeSlice = [string]$gate.next_safe_slice
    if (-not [string]::IsNullOrWhiteSpace($nextSafeSlice) -and -not $sliceIds.ContainsKey($nextSafeSlice)) {
        Fail "Gate $gateId references missing next_safe_slice $nextSafeSlice."
    }
}

foreach ($phase in @($plan.master.phases)) {
    foreach ($code in @($phase.roadmaps)) {
        if (-not $roadmapCodes.ContainsKey([string]$code)) {
            Fail "Master phase $($phase.code) references missing roadmap $code."
        }
    }
}

foreach ($decision in @($plan.foundation_decisions)) {
    $target = [string]$decision.revisit_gate
    if ([string]::IsNullOrWhiteSpace($target)) {
        continue
    }
    if (-not $gateIds.ContainsKey($target) -and -not $roadmapCodes.ContainsKey($target)) {
        Fail "Foundation decision $($decision.id) references missing revisit_gate $target."
    }
}

foreach ($roadmap in $roadmaps) {
    $code = [string]$roadmap.code
    foreach ($dependency in @($roadmap.dependencies)) {
        $dependencyCode = [string]$dependency
        if (-not $roadmapCodes.ContainsKey($dependencyCode)) {
            Fail "Roadmap $code references missing dependency $dependencyCode."
        }
    }
    foreach ($gate in @($roadmap.gates)) {
        $gateId = [string]$gate
        if (-not $gateIds.ContainsKey($gateId)) {
            Fail "Roadmap $code references missing gate $gateId."
        }
    }
}

foreach ($risk in @($plan.risk_register.risks)) {
    if ($null -eq $risk.gate) {
        continue
    }
    $gateId = [string]$risk.gate
    if (-not $gateIds.ContainsKey($gateId)) {
        Fail "Risk $($risk.id) references missing gate $gateId."
    }
}

foreach ($dependencyGroup in @($plan.integration.dependency_order)) {
    foreach ($dependency in @($dependencyGroup)) {
        $dependencyCode = [string]$dependency
        if (-not $roadmapCodes.ContainsKey($dependencyCode)) {
            Fail "Integration dependency_order references missing roadmap $dependencyCode."
        }
    }
}

Write-Host "Plan validation passed. Roadmaps: $($roadmaps.Count), Gates: $($gates.Count), Slices: $($sliceIds.Count)." -ForegroundColor Green
