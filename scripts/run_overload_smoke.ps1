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

function Get-ReasonList {
    param(
        [Parameter(Mandatory = $true)]$Sample,
        [Parameter(Mandatory = $true)]$Limits
    )

    $reasons = @()
    if ([int]$Sample.commands_per_second -gt [int]$Limits.max_commands_per_second) {
        $reasons += "command_rate"
    }
    if ([int]$Sample.pending_reliable_commands -gt [int]$Limits.max_pending_reliable_commands) {
        $reasons += "pending_reliable_commands"
    }
    if ([int]$Sample.reliable_backlog_packets -gt [int]$Limits.max_reliable_backlog_packets) {
        $reasons += "reliable_backlog"
    }
    if ([int64]$Sample.oldest_unacked_ticks -gt [int64]$Limits.resend_window_ticks) {
        $reasons += "resend_window"
    }
    if ([int]$Sample.bandwidth_kb_s -gt [int]$Limits.max_bandwidth_kb_s_per_client) {
        $reasons += "bandwidth"
    }
    if ([int]$Sample.log_events_per_minute -gt [int]$Limits.max_log_events_per_minute) {
        $reasons += "log_volume"
    }

    return $reasons
}

function Get-Policy {
    param(
        [Parameter(Mandatory = $true)]$Sample,
        [Parameter(Mandatory = $true)]$Limits
    )

    $reasons = @(Get-ReasonList -Sample $Sample -Limits $Limits)
    $action = "accept"

    if ($reasons.Count -gt 0) {
        $resendDisconnectThreshold = [int64]$Limits.resend_window_ticks * 2
        if ([int64]$Sample.oldest_unacked_ticks -gt $resendDisconnectThreshold -or
            [int]$Sample.reliable_backlog_packets -gt [int]$Limits.max_reliable_backlog_packets) {
            $action = "disconnect"
        } elseif ([int]$Sample.commands_per_second -gt [int]$Limits.max_commands_per_second -or
            [int]$Sample.pending_reliable_commands -gt [int]$Limits.max_pending_reliable_commands -or
            [int]$Sample.log_events_per_minute -gt [int]$Limits.max_log_events_per_minute) {
            $action = "drop_command"
        } else {
            $action = "degrade"
        }
    }

    $snapshotMode = "normal"
    if ($action -eq "disconnect") {
        $snapshotMode = "full_snapshot_only"
    } elseif ($action -eq "degrade") {
        if ([int]$Sample.reliable_backlog_packets -ge [int]$Limits.slow_client_backlog_packets -or
            [int]$Sample.bandwidth_kb_s -gt [int]$Limits.max_bandwidth_kb_s_per_client) {
            $snapshotMode = "aggregate_far_state_only"
        } else {
            $snapshotMode = "reduce_delta_rate"
        }
    }

    $commandAdmission = if ($action -eq "drop_command" -or $action -eq "disconnect") {
        "drop_new_commands"
    } else {
        "accept"
    }

    $diagnosticsEnabled = [int]$Sample.log_events_per_minute -le [int]$Limits.max_log_events_per_minute -and
        $action -ne "disconnect"

    return [pscustomobject]@{
        action = $action
        reasons = $reasons
        snapshot_mode = $snapshotMode
        command_admission = $commandAdmission
        optional_diagnostics_enabled = $diagnosticsEnabled
        disconnect = ($action -eq "disconnect")
    }
}

function Join-StringArray {
    param([object[]]$Values)
    return (@($Values) | ForEach-Object { [string]$_ }) -join ","
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

$casesPath = "tests\overload\load-shed-overload-cases.json"
if (-not (Test-Path $casesPath)) {
    throw "Missing overload cases: $casesPath"
}

$catalog = Get-Content -LiteralPath $casesPath -Raw | ConvertFrom-Json
if ($catalog.mode -ne "local_contract_smoke") {
    throw "Overload smoke refuses non-local mode '$($catalog.mode)'."
}
if ($catalog.live_go_required -or $catalog.requires_live_go -or $catalog.mode -match "live|steam|release") {
    throw "Overload smoke refuses live, Steam, or release-gated catalog settings."
}
if (-not $catalog.claim_limit -or $catalog.claim_limit -match "release-candidate overload readiness") {
    throw "Overload catalog must include a bounded non-release claim_limit."
}
if (@($catalog.cases).Count -eq 0) {
    throw "No overload cases are defined."
}

foreach ($case in @($catalog.cases)) {
    foreach ($field in @("id", "sample", "expected")) {
        if (-not ($case.PSObject.Properties.Name -contains $field)) {
            throw "Overload case is missing required field: $field"
        }
    }

    $policy = Get-Policy -Sample $case.sample -Limits $catalog.limits
    Assert-Equal -CaseId $case.id -Field "action" -Actual $policy.action -Expected $case.expected.action
    Assert-Equal -CaseId $case.id -Field "reasons" -Actual (Join-StringArray $policy.reasons) -Expected (Join-StringArray $case.expected.reasons)
    Assert-Equal -CaseId $case.id -Field "snapshot_mode" -Actual $policy.snapshot_mode -Expected $case.expected.snapshot_mode
    Assert-Equal -CaseId $case.id -Field "command_admission" -Actual $policy.command_admission -Expected $case.expected.command_admission
    Assert-Equal -CaseId $case.id -Field "optional_diagnostics_enabled" -Actual ([bool]$policy.optional_diagnostics_enabled) -Expected ([bool]$case.expected.optional_diagnostics_enabled)
    Assert-Equal -CaseId $case.id -Field "disconnect" -Actual ([bool]$policy.disconnect) -Expected ([bool]$case.expected.disconnect)

    Write-Host "overload_smoke case=$($case.id) status=ok action=$($policy.action) reasons=$(Join-StringArray $policy.reasons) snapshot_mode=$($policy.snapshot_mode) command_admission=$($policy.command_admission)"
}

$cargo = Get-CargoPath
& $cargo test load_shed
if ($LASTEXITCODE -ne 0) {
    throw "cargo test load_shed failed with exit code $LASTEXITCODE."
}

Write-Host "overload_smoke status=ok cases=$(@($catalog.cases).Count) rust_load_shed=ok mode=$($catalog.mode)"
