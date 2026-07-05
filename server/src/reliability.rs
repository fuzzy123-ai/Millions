use std::collections::{BTreeMap, BTreeSet};

use crate::simulation::Tick;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSequenceResult {
    Accepted {
        client_seq: u32,
    },
    DuplicateOrStale {
        last_accepted_client_seq: u32,
    },
    Gap {
        expected_client_seq: u32,
        received_client_seq: u32,
    },
    InvalidZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandIdempotencyResult {
    New,
    DuplicateIgnored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingReliablePacket {
    pub server_seq: u32,
    pub payload_len: u32,
    pub first_sent_tick: Tick,
    pub last_sent_tick: Tick,
    pub resend_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandReliabilityState {
    pub last_accepted_client_seq: u32,
    accepted_command_ids: BTreeSet<CommandId>,
    pending_server_packets: BTreeMap<u32, PendingReliablePacket>,
}

impl CommandReliabilityState {
    pub fn new() -> Self {
        Self {
            last_accepted_client_seq: 0,
            accepted_command_ids: BTreeSet::new(),
            pending_server_packets: BTreeMap::new(),
        }
    }

    pub fn accept_client_seq(&mut self, client_seq: u32) -> ClientSequenceResult {
        if client_seq == 0 {
            return ClientSequenceResult::InvalidZero;
        }

        let expected_client_seq = self.last_accepted_client_seq.saturating_add(1);
        if client_seq == expected_client_seq {
            self.last_accepted_client_seq = client_seq;
            ClientSequenceResult::Accepted { client_seq }
        } else if client_seq <= self.last_accepted_client_seq {
            ClientSequenceResult::DuplicateOrStale {
                last_accepted_client_seq: self.last_accepted_client_seq,
            }
        } else {
            ClientSequenceResult::Gap {
                expected_client_seq,
                received_client_seq: client_seq,
            }
        }
    }

    pub fn accept_command_id(&mut self, command_id: CommandId) -> CommandIdempotencyResult {
        if self.accepted_command_ids.insert(command_id) {
            CommandIdempotencyResult::New
        } else {
            CommandIdempotencyResult::DuplicateIgnored
        }
    }

    pub fn queue_reliable_packet(
        &mut self,
        server_seq: u32,
        payload_len: u32,
        now: Tick,
    ) -> PendingReliablePacket {
        let packet = PendingReliablePacket {
            server_seq,
            payload_len,
            first_sent_tick: now,
            last_sent_tick: now,
            resend_count: 0,
        };
        self.pending_server_packets.insert(server_seq, packet);
        packet
    }

    pub fn ack_server_packets_through(&mut self, ack_seq: u32) -> usize {
        let acked: Vec<u32> = self
            .pending_server_packets
            .keys()
            .copied()
            .filter(|server_seq| *server_seq <= ack_seq)
            .collect();

        for server_seq in &acked {
            self.pending_server_packets.remove(server_seq);
        }

        acked.len()
    }

    pub fn resend_due(&self, now: Tick, resend_after_ticks: u64) -> Vec<PendingReliablePacket> {
        self.pending_server_packets
            .values()
            .copied()
            .filter(|packet| now.0.saturating_sub(packet.last_sent_tick.0) >= resend_after_ticks)
            .collect()
    }

    pub fn mark_resent(&mut self, server_seq: u32, now: Tick) -> Option<PendingReliablePacket> {
        let packet = self.pending_server_packets.get_mut(&server_seq)?;
        packet.last_sent_tick = now;
        packet.resend_count = packet.resend_count.saturating_add(1);
        Some(*packet)
    }

    pub fn pending_server_packet_count(&self) -> usize {
        self.pending_server_packets.len()
    }
}

impl Default for CommandReliabilityState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_sequences_accept_only_next_monotonic_value() {
        let mut state = CommandReliabilityState::new();

        assert_eq!(
            state.accept_client_seq(1),
            ClientSequenceResult::Accepted { client_seq: 1 }
        );
        assert_eq!(
            state.accept_client_seq(1),
            ClientSequenceResult::DuplicateOrStale {
                last_accepted_client_seq: 1,
            }
        );
        assert_eq!(
            state.accept_client_seq(3),
            ClientSequenceResult::Gap {
                expected_client_seq: 2,
                received_client_seq: 3,
            }
        );
        assert_eq!(
            state.accept_client_seq(2),
            ClientSequenceResult::Accepted { client_seq: 2 }
        );
    }

    #[test]
    fn command_ids_are_idempotent() {
        let mut state = CommandReliabilityState::new();

        assert_eq!(
            state.accept_command_id(CommandId(42)),
            CommandIdempotencyResult::New
        );
        assert_eq!(
            state.accept_command_id(CommandId(42)),
            CommandIdempotencyResult::DuplicateIgnored
        );
        assert_eq!(
            state.accept_command_id(CommandId(43)),
            CommandIdempotencyResult::New
        );
    }

    #[test]
    fn ack_removes_reliable_server_packets() {
        let mut state = CommandReliabilityState::new();
        state.queue_reliable_packet(1, 24, Tick(10));
        state.queue_reliable_packet(2, 32, Tick(11));
        state.queue_reliable_packet(3, 40, Tick(12));

        assert_eq!(state.ack_server_packets_through(2), 2);
        assert_eq!(state.pending_server_packet_count(), 1);
        assert_eq!(state.resend_due(Tick(20), 1)[0].server_seq, 3);
    }

    #[test]
    fn resend_due_and_mark_resent_are_bounded() {
        let mut state = CommandReliabilityState::new();
        state.queue_reliable_packet(7, 24, Tick(10));

        assert!(state.resend_due(Tick(14), 5).is_empty());
        assert_eq!(state.resend_due(Tick(15), 5)[0].server_seq, 7);

        let resent = state.mark_resent(7, Tick(15)).expect("packet exists");
        assert_eq!(resent.resend_count, 1);
        assert!(state.resend_due(Tick(19), 5).is_empty());
        assert_eq!(state.resend_due(Tick(20), 5)[0].resend_count, 1);
    }
}
