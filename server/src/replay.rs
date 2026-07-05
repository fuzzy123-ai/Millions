use std::collections::BTreeMap;

use crate::determinism::{DeterministicInput, DeterministicInputFrame, SimulationSeed};
use crate::simulation::Tick;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub frame_checksums: Vec<(Tick, u64)>,
}

impl ReplayReport {
    pub fn first_divergent_tick(&self, other: &ReplayReport) -> Option<Tick> {
        let max_len = self.frame_checksums.len().max(other.frame_checksums.len());
        for index in 0..max_len {
            let left = self.frame_checksums.get(index);
            let right = other.frame_checksums.get(index);
            if left != right {
                return left
                    .map(|(tick, _)| *tick)
                    .or_else(|| right.map(|(tick, _)| *tick));
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCommandStream {
    seed: SimulationSeed,
    inputs_by_tick: BTreeMap<u64, Vec<DeterministicInput>>,
}

impl RecordedCommandStream {
    pub fn new(seed: SimulationSeed) -> Self {
        Self {
            seed,
            inputs_by_tick: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, input: DeterministicInput) {
        self.inputs_by_tick
            .entry(input.tick.0)
            .or_default()
            .push(input);
    }

    pub fn canonical_frames(&self) -> Vec<DeterministicInputFrame> {
        self.inputs_by_tick
            .iter()
            .map(|(tick, inputs)| DeterministicInputFrame::canonical(Tick(*tick), inputs.clone()))
            .collect()
    }

    pub fn replay_report(&self) -> ReplayReport {
        let frame_checksums = self
            .canonical_frames()
            .iter()
            .map(|frame| (frame.tick, frame.checksum(self.seed)))
            .collect();
        ReplayReport { frame_checksums }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::determinism::{derive_simulation_seed, DeterministicInput};
    use crate::protocol::PROTOCOL_VERSION;
    use crate::reliability::CommandId;
    use crate::simulation::PlayerSessionId;

    fn input(tick: u64, session_id: u64, command_id: u64, payload_hash: u64) -> DeterministicInput {
        DeterministicInput {
            tick: Tick(tick),
            player_session_id: PlayerSessionId(session_id),
            command_id: CommandId(command_id),
            command_type: 1,
            target_tick: Tick(tick + 1),
            payload_hash,
        }
    }

    #[test]
    fn recorder_groups_inputs_into_canonical_frames() {
        let seed = derive_simulation_seed(1, 2, PROTOCOL_VERSION);
        let mut stream = RecordedCommandStream::new(seed);
        stream.record(input(20, 2, 1, 100));
        stream.record(input(20, 1, 1, 100));
        stream.record(input(21, 1, 2, 200));

        let frames = stream.canonical_frames();

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].tick, Tick(20));
        assert_eq!(frames[0].inputs[0].player_session_id, PlayerSessionId(1));
        assert_eq!(frames[1].tick, Tick(21));
    }

    #[test]
    fn replay_report_is_stable_across_record_order() {
        let seed = derive_simulation_seed(1, 2, PROTOCOL_VERSION);
        let mut first = RecordedCommandStream::new(seed);
        first.record(input(20, 2, 1, 100));
        first.record(input(20, 1, 1, 100));

        let mut second = RecordedCommandStream::new(seed);
        second.record(input(20, 1, 1, 100));
        second.record(input(20, 2, 1, 100));

        assert_eq!(first.replay_report(), second.replay_report());
    }

    #[test]
    fn replay_report_identifies_first_divergent_tick() {
        let seed = derive_simulation_seed(1, 2, PROTOCOL_VERSION);
        let mut first = RecordedCommandStream::new(seed);
        first.record(input(20, 1, 1, 100));
        first.record(input(21, 1, 2, 200));

        let mut second = RecordedCommandStream::new(seed);
        second.record(input(20, 1, 1, 100));
        second.record(input(21, 1, 2, 201));

        assert_eq!(
            first
                .replay_report()
                .first_divergent_tick(&second.replay_report()),
            Some(Tick(21))
        );
    }
}
