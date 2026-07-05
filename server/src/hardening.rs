use crate::protocol::{PacketDecodeError, PacketHeader, PROTOCOL_HEADER_LEN};

pub const MAX_PACKET_BYTES: usize = 1200;
pub const MAX_PAYLOAD_BYTES: usize = MAX_PACKET_BYTES - PROTOCOL_HEADER_LEN;
pub const MAX_CLIENT_COMMAND_BATCH_BYTES: usize = 1024;
pub const MAX_REDACTED_DIAGNOSTIC_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolHardeningLimits {
    pub max_packet_bytes: usize,
    pub max_payload_bytes: usize,
    pub max_client_command_batch_bytes: usize,
    pub max_redacted_diagnostic_bytes: usize,
}

impl Default for ProtocolHardeningLimits {
    fn default() -> Self {
        Self {
            max_packet_bytes: MAX_PACKET_BYTES,
            max_payload_bytes: MAX_PAYLOAD_BYTES,
            max_client_command_batch_bytes: MAX_CLIENT_COMMAND_BATCH_BYTES,
            max_redacted_diagnostic_bytes: MAX_REDACTED_DIAGNOSTIC_BYTES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolRejectReason {
    PacketTooLarge,
    PayloadTooLarge,
    ClientCommandBatchTooLarge,
    VersionMismatch,
    MalformedHeader,
    MalformedPayload,
    AuthMissing,
    AuthRejected,
    StaleCommand,
    ReplayedCommand,
}

impl ProtocolRejectReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PacketTooLarge => "packet_too_large",
            Self::PayloadTooLarge => "payload_too_large",
            Self::ClientCommandBatchTooLarge => "client_command_batch_too_large",
            Self::VersionMismatch => "version_mismatch",
            Self::MalformedHeader => "malformed_header",
            Self::MalformedPayload => "malformed_payload",
            Self::AuthMissing => "auth_missing",
            Self::AuthRejected => "auth_rejected",
            Self::StaleCommand => "stale_command",
            Self::ReplayedCommand => "replayed_command",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolHardeningAction {
    Accept,
    RejectNoStateMutation,
    AckDuplicateNoStateMutation,
    Disconnect,
}

impl ProtocolHardeningAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::RejectNoStateMutation => "reject_no_state_mutation",
            Self::AckDuplicateNoStateMutation => "ack_duplicate_no_state_mutation",
            Self::Disconnect => "disconnect",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolHardeningDecision {
    pub action: ProtocolHardeningAction,
    pub reason: Option<ProtocolRejectReason>,
    pub mutate_authoritative_state: bool,
    pub redacted_diagnostics_only: bool,
}

impl ProtocolHardeningDecision {
    pub fn accept() -> Self {
        Self {
            action: ProtocolHardeningAction::Accept,
            reason: None,
            mutate_authoritative_state: true,
            redacted_diagnostics_only: false,
        }
    }

    pub fn reject(reason: ProtocolRejectReason) -> Self {
        Self {
            action: ProtocolHardeningAction::RejectNoStateMutation,
            reason: Some(reason),
            mutate_authoritative_state: false,
            redacted_diagnostics_only: true,
        }
    }

    pub fn duplicate(reason: ProtocolRejectReason) -> Self {
        Self {
            action: ProtocolHardeningAction::AckDuplicateNoStateMutation,
            reason: Some(reason),
            mutate_authoritative_state: false,
            redacted_diagnostics_only: true,
        }
    }

    pub fn disconnect(reason: ProtocolRejectReason) -> Self {
        Self {
            action: ProtocolHardeningAction::Disconnect,
            reason: Some(reason),
            mutate_authoritative_state: false,
            redacted_diagnostics_only: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthCheckState {
    LocalMockAccepted,
    MissingProof,
    RejectedProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandFreshness {
    Fresh,
    Stale,
    Replayed,
}

pub fn validate_packet_bounds(
    packet: &[u8],
    limits: ProtocolHardeningLimits,
) -> ProtocolHardeningDecision {
    if packet.len() > limits.max_packet_bytes {
        return ProtocolHardeningDecision::disconnect(ProtocolRejectReason::PacketTooLarge);
    }

    match PacketHeader::decode(packet) {
        Ok(header) => validate_header_payload_limits(header, limits),
        Err(PacketDecodeError::UnsupportedProtocol(_)) => {
            ProtocolHardeningDecision::reject(ProtocolRejectReason::VersionMismatch)
        }
        Err(_) => ProtocolHardeningDecision::reject(ProtocolRejectReason::MalformedHeader),
    }
}

pub fn validate_header_payload_limits(
    header: PacketHeader,
    limits: ProtocolHardeningLimits,
) -> ProtocolHardeningDecision {
    if header.payload_len as usize > limits.max_payload_bytes {
        return ProtocolHardeningDecision::reject(ProtocolRejectReason::PayloadTooLarge);
    }

    if header.message_type == crate::protocol::MessageType::ClientCommandBatch
        && header.payload_len as usize > limits.max_client_command_batch_bytes
    {
        return ProtocolHardeningDecision::reject(ProtocolRejectReason::ClientCommandBatchTooLarge);
    }

    ProtocolHardeningDecision::accept()
}

pub fn evaluate_auth_state(auth: AuthCheckState) -> ProtocolHardeningDecision {
    match auth {
        AuthCheckState::LocalMockAccepted => ProtocolHardeningDecision::accept(),
        AuthCheckState::MissingProof => {
            ProtocolHardeningDecision::reject(ProtocolRejectReason::AuthMissing)
        }
        AuthCheckState::RejectedProof => {
            ProtocolHardeningDecision::disconnect(ProtocolRejectReason::AuthRejected)
        }
    }
}

pub fn evaluate_command_freshness(freshness: CommandFreshness) -> ProtocolHardeningDecision {
    match freshness {
        CommandFreshness::Fresh => ProtocolHardeningDecision::accept(),
        CommandFreshness::Stale => {
            ProtocolHardeningDecision::duplicate(ProtocolRejectReason::StaleCommand)
        }
        CommandFreshness::Replayed => {
            ProtocolHardeningDecision::duplicate(ProtocolRejectReason::ReplayedCommand)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_packet() -> Vec<u8> {
        include_bytes!("../../protocol/fixtures/protocol_v0_server_hello_accept.bin").to_vec()
    }

    fn command_fixture_packet() -> Vec<u8> {
        include_bytes!("../../protocol/fixtures/protocol_v0_command_ready_batch_ok.bin").to_vec()
    }

    fn assert_safe_non_accept(decision: ProtocolHardeningDecision) {
        if decision.action != ProtocolHardeningAction::Accept {
            assert!(!decision.mutate_authoritative_state);
            assert!(decision.redacted_diagnostics_only);
        }
    }

    #[test]
    fn hardening_accepts_fixture_within_bounds() {
        let decision =
            validate_packet_bounds(&fixture_packet(), ProtocolHardeningLimits::default());

        assert_eq!(decision, ProtocolHardeningDecision::accept());
    }

    #[test]
    fn hardening_rejects_version_mismatch_without_state_mutation() {
        let mut packet = fixture_packet();
        packet[2] = 99;

        let decision = validate_packet_bounds(&packet, ProtocolHardeningLimits::default());

        assert_eq!(
            decision,
            ProtocolHardeningDecision::reject(ProtocolRejectReason::VersionMismatch)
        );
        assert_eq!(decision.reason.unwrap().as_str(), "version_mismatch");
        assert!(!decision.mutate_authoritative_state);
    }

    #[test]
    fn hardening_disconnects_oversized_packets_before_decode() {
        let mut packet = fixture_packet();
        packet.resize(MAX_PACKET_BYTES + 1, 0);

        let decision = validate_packet_bounds(&packet, ProtocolHardeningLimits::default());

        assert_eq!(
            decision,
            ProtocolHardeningDecision::disconnect(ProtocolRejectReason::PacketTooLarge)
        );
    }

    #[test]
    fn hardening_rejects_oversized_client_command_batches() {
        let mut packet = command_fixture_packet();
        packet[8..12].copy_from_slice(&(1100u32).to_le_bytes());
        packet.resize(PROTOCOL_HEADER_LEN + 1100, 0);

        let decision = validate_packet_bounds(&packet, ProtocolHardeningLimits::default());

        assert_eq!(
            decision,
            ProtocolHardeningDecision::reject(ProtocolRejectReason::ClientCommandBatchTooLarge)
        );
    }

    #[test]
    fn hardening_maps_auth_failures_to_safe_actions() {
        assert_eq!(
            evaluate_auth_state(AuthCheckState::MissingProof),
            ProtocolHardeningDecision::reject(ProtocolRejectReason::AuthMissing)
        );
        assert_eq!(
            evaluate_auth_state(AuthCheckState::RejectedProof),
            ProtocolHardeningDecision::disconnect(ProtocolRejectReason::AuthRejected)
        );
    }

    #[test]
    fn hardening_acks_stale_or_replayed_commands_without_mutation() {
        for freshness in [CommandFreshness::Stale, CommandFreshness::Replayed] {
            let decision = evaluate_command_freshness(freshness);

            assert_eq!(
                decision.action,
                ProtocolHardeningAction::AckDuplicateNoStateMutation
            );
            assert!(!decision.mutate_authoritative_state);
            assert!(decision.redacted_diagnostics_only);
        }
    }

    #[test]
    fn hardening_rejects_all_truncated_headers_without_panic() {
        let packet = fixture_packet();

        for len in 0..PROTOCOL_HEADER_LEN {
            let decision =
                validate_packet_bounds(&packet[..len], ProtocolHardeningLimits::default());

            assert_eq!(
                decision,
                ProtocolHardeningDecision::reject(ProtocolRejectReason::MalformedHeader),
                "truncated length {len} should reject as malformed_header"
            );
        }
    }

    #[test]
    fn hardening_header_byte_mutations_are_bounded() {
        let packet = fixture_packet();

        for index in 0..PROTOCOL_HEADER_LEN {
            let mut mutated = packet.clone();
            mutated[index] ^= 0xFF;

            let decision = validate_packet_bounds(&mutated, ProtocolHardeningLimits::default());
            assert_safe_non_accept(decision);
        }
    }

    #[test]
    fn hardening_never_allocates_declared_payload_when_lengths_mismatch() {
        let mut packet = fixture_packet();
        packet[8..12].copy_from_slice(&(MAX_PAYLOAD_BYTES as u32 + 1).to_le_bytes());

        let decision = validate_packet_bounds(&packet, ProtocolHardeningLimits::default());

        assert_eq!(
            decision,
            ProtocolHardeningDecision::reject(ProtocolRejectReason::MalformedHeader)
        );
        assert!(!decision.mutate_authoritative_state);
    }

    #[test]
    fn hardening_fuzz_seed_catalog_is_present() {
        let corpus = include_str!("../../tests/fuzz/protocol-hardening-corpus.json");

        for required in [
            "HARDEN-02",
            "truncated_header_range",
            "unsupported_protocol_version",
            "oversized_packet_disconnect",
            "oversized_command_batch",
            "single_byte_header_mutations",
            "auth_and_replay_state_table",
        ] {
            assert!(corpus.contains(required), "missing fuzz seed {required}");
        }
    }
}
