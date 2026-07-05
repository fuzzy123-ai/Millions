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

function Assert-Equal {
    param(
        [Parameter(Mandatory = $true)][string]$CaseId,
        [Parameter(Mandatory = $true)][string]$Field,
        [Parameter(Mandatory = $true)]$Actual,
        [Parameter(Mandatory = $true)]$Expected
    )

    if ($Actual -ne $Expected) {
        throw "Case '$CaseId' $Field mismatch: got '$Actual', expected '$Expected'."
    }
}

$casesPath = "tests\hardening\hostile-input-cases.json"
$corpusPath = "tests\fuzz\protocol-hardening-corpus.json"

if (-not (Test-Path $casesPath)) {
    throw "Missing hostile-input cases: $casesPath"
}
if (-not (Test-Path $corpusPath)) {
    throw "Missing protocol hardening corpus: $corpusPath"
}

$catalog = Get-Content -LiteralPath $casesPath -Raw | ConvertFrom-Json
$corpus = Get-Content -LiteralPath $corpusPath -Raw | ConvertFrom-Json

if ($catalog.mode -ne "local_hostile_input_smoke") {
    throw "Hostile-input smoke refuses non-local mode '$($catalog.mode)'."
}
if ($catalog.live_go_required -or $catalog.requires_live_go -or $catalog.mode -match "live|steam|release|public") {
    throw "Hostile-input smoke refuses live, Steam, release, or public-network catalog settings."
}
if (-not $catalog.claim_limit -or $catalog.claim_limit -notmatch "do not.*release-candidate hardening") {
    throw "Hostile-input catalog must include a bounded non-release claim_limit."
}
if (@($catalog.cases).Count -eq 0) {
    throw "No hostile-input cases are defined."
}

$seedIds = @{}
foreach ($seed in @($corpus.seeds)) {
    $seedIds[$seed.id] = $true
}

foreach ($case in @($catalog.cases)) {
    foreach ($field in @("id", "input_family", "expected_action", "expected_reason", "expected_mutates_state", "expected_redacted_diagnostics_only")) {
        if (-not ($case.PSObject.Properties.Name -contains $field)) {
            throw "Hostile-input case is missing required field: $field"
        }
    }
    if (-not $seedIds.ContainsKey($case.input_family)) {
        throw "Case '$($case.id)' references unknown fuzz seed family '$($case.input_family)'."
    }

    Assert-Equal -CaseId $case.id -Field "expected_mutates_state" -Actual ([bool]$case.expected_mutates_state) -Expected $false
    Assert-Equal -CaseId $case.id -Field "expected_redacted_diagnostics_only" -Actual ([bool]$case.expected_redacted_diagnostics_only) -Expected $true
    if ($case.expected_action -notin @("reject_no_state_mutation", "ack_duplicate_no_state_mutation", "disconnect")) {
        throw "Case '$($case.id)' has unsafe action '$($case.expected_action)'."
    }

    Write-Host "hostile_input_smoke case=$($case.id) status=ok action=$($case.expected_action) reason=$($case.expected_reason) family=$($case.input_family)"
}

$cargo = Get-CargoPath
& $cargo test hardening
if ($LASTEXITCODE -ne 0) {
    throw "cargo test hardening failed with exit code $LASTEXITCODE."
}

Write-Host "hostile_input_smoke status=ok cases=$(@($catalog.cases).Count) rust_hardening=ok mode=$($catalog.mode)"
