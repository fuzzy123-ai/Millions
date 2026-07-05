$ErrorActionPreference = "Stop"

$reportPath = "tests\perf\cover-combat-smoke-report.json"
if (-not (Test-Path $reportPath)) {
    throw "Missing cover combat smoke report: $reportPath"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.slice -ne "GCOV-04") {
    throw "Cover combat report slice mismatch: $($report.slice)"
}
if ($report.status -ne "informational" -or $report.budget_result -ne "blocked") {
    throw "Cover combat report must remain informational/blocked until measured p95 evidence exists."
}
if ([int]$report.query_count -ne ([int]$report.attacker_count * [int]$report.target_count)) {
    throw "Cover combat report query_count does not match attacker_count * target_count."
}
if (@($report.required_result_buckets).Count -ne 4) {
    throw "Cover combat report must require all four result buckets."
}

$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCommand) {
    $cargo = $cargoCommand.Source
} else {
    $cargo = "C:\Users\nkatz\.cargo\bin\cargo.exe"
}
if (-not (Test-Path $cargo)) {
    throw "cargo executable not found."
}

& $cargo test cover_combat_perf_smoke
if ($LASTEXITCODE -ne 0) {
    throw "cargo test cover_combat_perf_smoke failed with exit code $LASTEXITCODE."
}

Write-Host "cover_combat_perf_smoke status=ok attackers=$($report.attacker_count) targets=$($report.target_count) queries=$($report.query_count) budget_result=$($report.budget_result)"

