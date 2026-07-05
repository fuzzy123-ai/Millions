#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadShedLimits {
    pub max_commands_per_second: u32,
    pub max_pending_reliable_commands: u32,
    pub max_reliable_backlog_packets: u32,
    pub resend_window_ticks: u64,
    pub max_bandwidth_kb_s_per_client: u32,
    pub max_log_events_per_minute: u32,
    pub slow_client_backlog_packets: u32,
}

impl Default for LoadShedLimits {
    fn default() -> Self {
        Self {
            max_commands_per_second: 30,
            max_pending_reliable_commands: 128,
            max_reliable_backlog_packets: 256,
            resend_window_ticks: 40,
            max_bandwidth_kb_s_per_client: 256,
            max_log_events_per_minute: 120,
            slow_client_backlog_packets: 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientLoadSample {
    pub commands_per_second: u32,
    pub pending_reliable_commands: u32,
    pub reliable_backlog_packets: u32,
    pub oldest_unacked_ticks: u64,
    pub bandwidth_kb_s: u32,
    pub log_events_per_minute: u32,
}

impl ClientLoadSample {
    pub fn idle() -> Self {
        Self {
            commands_per_second: 0,
            pending_reliable_commands: 0,
            reliable_backlog_packets: 0,
            oldest_unacked_ticks: 0,
            bandwidth_kb_s: 0,
            log_events_per_minute: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadShedReason {
    CommandRate,
    PendingReliableCommands,
    ReliableBacklog,
    ResendWindow,
    Bandwidth,
    LogVolume,
}

impl LoadShedReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CommandRate => "command_rate",
            Self::PendingReliableCommands => "pending_reliable_commands",
            Self::ReliableBacklog => "reliable_backlog",
            Self::ResendWindow => "resend_window",
            Self::Bandwidth => "bandwidth",
            Self::LogVolume => "log_volume",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadShedAction {
    Accept,
    Degrade,
    DropCommand,
    Disconnect,
}

impl LoadShedAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Degrade => "degrade",
            Self::DropCommand => "drop_command",
            Self::Disconnect => "disconnect",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotDegradeMode {
    Normal,
    ReduceDeltaRate,
    AggregateFarStateOnly,
    FullSnapshotOnly,
}

impl SnapshotDegradeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::ReduceDeltaRate => "reduce_delta_rate",
            Self::AggregateFarStateOnly => "aggregate_far_state_only",
            Self::FullSnapshotOnly => "full_snapshot_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAdmission {
    Accept,
    DropNewCommands,
}

impl CommandAdmission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::DropNewCommands => "drop_new_commands",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlowClientPolicyDecision {
    pub load_shed: LoadShedDecision,
    pub snapshot_mode: SnapshotDegradeMode,
    pub command_admission: CommandAdmission,
    pub optional_diagnostics_enabled: bool,
    pub disconnect: bool,
}

impl SlowClientPolicyDecision {
    pub fn action_name(&self) -> &'static str {
        self.load_shed.action.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadShedDecision {
    pub action: LoadShedAction,
    pub reasons: Vec<LoadShedReason>,
}

impl LoadShedDecision {
    pub fn is_over_limit(&self) -> bool {
        self.action != LoadShedAction::Accept
    }
}

pub fn evaluate_load_shed(sample: ClientLoadSample, limits: LoadShedLimits) -> LoadShedDecision {
    let mut reasons = Vec::new();

    if sample.commands_per_second > limits.max_commands_per_second {
        reasons.push(LoadShedReason::CommandRate);
    }
    if sample.pending_reliable_commands > limits.max_pending_reliable_commands {
        reasons.push(LoadShedReason::PendingReliableCommands);
    }
    if sample.reliable_backlog_packets > limits.max_reliable_backlog_packets {
        reasons.push(LoadShedReason::ReliableBacklog);
    }
    if sample.oldest_unacked_ticks > limits.resend_window_ticks {
        reasons.push(LoadShedReason::ResendWindow);
    }
    if sample.bandwidth_kb_s > limits.max_bandwidth_kb_s_per_client {
        reasons.push(LoadShedReason::Bandwidth);
    }
    if sample.log_events_per_minute > limits.max_log_events_per_minute {
        reasons.push(LoadShedReason::LogVolume);
    }

    let action = choose_action(sample, limits, &reasons);
    LoadShedDecision { action, reasons }
}

pub fn apply_slow_client_policy(
    sample: ClientLoadSample,
    limits: LoadShedLimits,
) -> SlowClientPolicyDecision {
    let load_shed = evaluate_load_shed(sample, limits);
    let disconnect = load_shed.action == LoadShedAction::Disconnect;
    let command_admission = if load_shed.action == LoadShedAction::DropCommand || disconnect {
        CommandAdmission::DropNewCommands
    } else {
        CommandAdmission::Accept
    };
    let snapshot_mode = choose_snapshot_mode(sample, limits, load_shed.action);
    let optional_diagnostics_enabled =
        sample.log_events_per_minute <= limits.max_log_events_per_minute && !disconnect;

    SlowClientPolicyDecision {
        load_shed,
        snapshot_mode,
        command_admission,
        optional_diagnostics_enabled,
        disconnect,
    }
}

fn choose_action(
    sample: ClientLoadSample,
    limits: LoadShedLimits,
    reasons: &[LoadShedReason],
) -> LoadShedAction {
    if reasons.is_empty() {
        return LoadShedAction::Accept;
    }

    if sample.oldest_unacked_ticks > limits.resend_window_ticks.saturating_mul(2)
        || sample.reliable_backlog_packets > limits.max_reliable_backlog_packets
    {
        return LoadShedAction::Disconnect;
    }

    if sample.commands_per_second > limits.max_commands_per_second
        || sample.pending_reliable_commands > limits.max_pending_reliable_commands
        || sample.log_events_per_minute > limits.max_log_events_per_minute
    {
        return LoadShedAction::DropCommand;
    }

    LoadShedAction::Degrade
}

fn choose_snapshot_mode(
    sample: ClientLoadSample,
    limits: LoadShedLimits,
    action: LoadShedAction,
) -> SnapshotDegradeMode {
    match action {
        LoadShedAction::Accept => SnapshotDegradeMode::Normal,
        LoadShedAction::DropCommand => SnapshotDegradeMode::Normal,
        LoadShedAction::Disconnect => SnapshotDegradeMode::FullSnapshotOnly,
        LoadShedAction::Degrade => {
            if sample.reliable_backlog_packets >= limits.slow_client_backlog_packets
                || sample.bandwidth_kb_s > limits.max_bandwidth_kb_s_per_client
            {
                SnapshotDegradeMode::AggregateFarStateOnly
            } else {
                SnapshotDegradeMode::ReduceDeltaRate
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_shed_accepts_idle_client() {
        let decision = evaluate_load_shed(ClientLoadSample::idle(), LoadShedLimits::default());

        assert_eq!(decision.action, LoadShedAction::Accept);
        assert!(decision.reasons.is_empty());
        assert!(!decision.is_over_limit());
    }

    #[test]
    fn load_shed_drops_over_rate_commands() {
        let sample = ClientLoadSample {
            commands_per_second: 31,
            ..ClientLoadSample::idle()
        };
        let decision = evaluate_load_shed(sample, LoadShedLimits::default());

        assert_eq!(decision.action, LoadShedAction::DropCommand);
        assert_eq!(decision.reasons, vec![LoadShedReason::CommandRate]);
        assert_eq!(decision.reasons[0].as_str(), "command_rate");
    }

    #[test]
    fn load_shed_degrades_bandwidth_before_disconnect() {
        let sample = ClientLoadSample {
            bandwidth_kb_s: 300,
            reliable_backlog_packets: 65,
            ..ClientLoadSample::idle()
        };
        let decision = evaluate_load_shed(sample, LoadShedLimits::default());

        assert_eq!(decision.action, LoadShedAction::Degrade);
        assert_eq!(decision.reasons, vec![LoadShedReason::Bandwidth]);
    }

    #[test]
    fn load_shed_disconnects_when_resend_window_is_exhausted() {
        let sample = ClientLoadSample {
            oldest_unacked_ticks: 81,
            ..ClientLoadSample::idle()
        };
        let decision = evaluate_load_shed(sample, LoadShedLimits::default());

        assert_eq!(decision.action, LoadShedAction::Disconnect);
        assert_eq!(decision.reasons, vec![LoadShedReason::ResendWindow]);
    }

    #[test]
    fn slow_client_policy_reduces_snapshot_detail_before_disconnect() {
        let sample = ClientLoadSample {
            bandwidth_kb_s: 300,
            reliable_backlog_packets: 64,
            ..ClientLoadSample::idle()
        };
        let policy = apply_slow_client_policy(sample, LoadShedLimits::default());

        assert_eq!(policy.action_name(), "degrade");
        assert_eq!(
            policy.snapshot_mode,
            SnapshotDegradeMode::AggregateFarStateOnly
        );
        assert_eq!(policy.snapshot_mode.as_str(), "aggregate_far_state_only");
        assert_eq!(policy.command_admission, CommandAdmission::Accept);
        assert!(policy.optional_diagnostics_enabled);
        assert!(!policy.disconnect);
    }

    #[test]
    fn slow_client_policy_drops_new_commands_under_command_pressure() {
        let sample = ClientLoadSample {
            commands_per_second: 90,
            log_events_per_minute: 121,
            ..ClientLoadSample::idle()
        };
        let policy = apply_slow_client_policy(sample, LoadShedLimits::default());

        assert_eq!(policy.load_shed.action, LoadShedAction::DropCommand);
        assert_eq!(policy.command_admission, CommandAdmission::DropNewCommands);
        assert_eq!(policy.command_admission.as_str(), "drop_new_commands");
        assert_eq!(policy.snapshot_mode, SnapshotDegradeMode::Normal);
        assert!(!policy.optional_diagnostics_enabled);
        assert!(!policy.disconnect);
    }

    #[test]
    fn slow_client_policy_disconnects_and_requires_full_snapshot_resume() {
        let sample = ClientLoadSample {
            oldest_unacked_ticks: 100,
            reliable_backlog_packets: 257,
            ..ClientLoadSample::idle()
        };
        let policy = apply_slow_client_policy(sample, LoadShedLimits::default());

        assert_eq!(policy.load_shed.action, LoadShedAction::Disconnect);
        assert_eq!(policy.snapshot_mode, SnapshotDegradeMode::FullSnapshotOnly);
        assert_eq!(policy.command_admission, CommandAdmission::DropNewCommands);
        assert!(!policy.optional_diagnostics_enabled);
        assert!(policy.disconnect);
    }
}
