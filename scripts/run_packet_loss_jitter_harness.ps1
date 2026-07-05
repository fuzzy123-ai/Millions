$ErrorActionPreference = "Stop"

$casesPath = "tests\replay\packet-loss-jitter-cases.json"
if (-not (Test-Path $casesPath)) {
    throw "Missing packet loss/jitter cases: $casesPath"
}

$suite = Get-Content -LiteralPath $casesPath -Raw | ConvertFrom-Json
if ($suite.mode -ne "local_synthetic") {
    throw "Packet loss/jitter harness refuses non-local mode '$($suite.mode)'."
}

function Join-IntArray {
    param([object[]]$Values)
    return (@($Values) | ForEach-Object { [int]$_ }) -join ","
}

foreach ($case in @($suite.cases)) {
    $lastAccepted = 0
    $accepted = @()
    $duplicates = 0
    $gaps = 0

    foreach ($rawSeq in @($case.client_seq_stream)) {
        $seq = [int]$rawSeq
        $expected = $lastAccepted + 1
        if ($seq -eq $expected) {
            $accepted += $seq
            $lastAccepted = $seq
        } elseif ($seq -le $lastAccepted) {
            $duplicates += 1
        } else {
            $gaps += 1
        }
    }

    $pending = @()
    $resendDue = @()
    foreach ($packet in @($case.server_packets)) {
        $serverSeq = [int]$packet.server_seq
        if ($serverSeq -gt [int]$case.ack_seq) {
            $pending += $serverSeq
            if (([int]$case.now_tick - [int]$packet.last_sent_tick) -ge [int]$case.resend_after_ticks) {
                $resendDue += $serverSeq
            }
        }
    }

    if ((Join-IntArray $accepted) -ne (Join-IntArray $case.expected_accepted)) {
        throw "Case '$($case.id)' accepted mismatch: got $(Join-IntArray $accepted), expected $(Join-IntArray $case.expected_accepted)"
    }
    if ($duplicates -ne [int]$case.expected_duplicate_or_stale) {
        throw "Case '$($case.id)' duplicate/stale mismatch: got $duplicates, expected $($case.expected_duplicate_or_stale)"
    }
    if ($gaps -ne [int]$case.expected_gaps) {
        throw "Case '$($case.id)' gap mismatch: got $gaps, expected $($case.expected_gaps)"
    }
    if ((Join-IntArray $pending) -ne (Join-IntArray $case.expected_pending_server_seq)) {
        throw "Case '$($case.id)' pending server seq mismatch: got $(Join-IntArray $pending), expected $(Join-IntArray $case.expected_pending_server_seq)"
    }
    if ((Join-IntArray $resendDue) -ne (Join-IntArray $case.expected_resend_due)) {
        throw "Case '$($case.id)' resend due mismatch: got $(Join-IntArray $resendDue), expected $(Join-IntArray $case.expected_resend_due)"
    }

    Write-Host "packet_loss_jitter_harness case=$($case.id) status=ok accepted=$(Join-IntArray $accepted) gaps=$gaps duplicates=$duplicates resend_due=$(Join-IntArray $resendDue)"
}

Write-Host "packet_loss_jitter_harness status=ok cases=$(@($suite.cases).Count) mode=$($suite.mode)"
