use crate::reliability::CommandId;
use crate::simulation::{PlayerSessionId, Tick};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationSeed(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicInput {
    pub tick: Tick,
    pub player_session_id: PlayerSessionId,
    pub command_id: CommandId,
    pub command_type: u16,
    pub target_tick: Tick,
    pub payload_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicInputFrame {
    pub tick: Tick,
    pub inputs: Vec<DeterministicInput>,
}

impl DeterministicInputFrame {
    pub fn canonical(tick: Tick, mut inputs: Vec<DeterministicInput>) -> Self {
        inputs.retain(|input| input.tick == tick);
        inputs.sort_by_key(|input| {
            (
                input.player_session_id.0,
                input.command_id.0,
                input.command_type,
                input.target_tick.0,
            )
        });
        Self { tick, inputs }
    }

    pub fn checksum(&self, seed: SimulationSeed) -> u64 {
        let mut hash = fnv1a64(seed.0, self.tick.0);
        for input in &self.inputs {
            hash = fnv1a64(hash, input.player_session_id.0);
            hash = fnv1a64(hash, input.command_id.0);
            hash = fnv1a64(hash, u64::from(input.command_type));
            hash = fnv1a64(hash, input.target_tick.0);
            hash = fnv1a64(hash, input.payload_hash);
        }
        hash
    }
}

pub fn derive_simulation_seed(
    match_id: u64,
    map_checksum: u64,
    protocol_version: u16,
) -> SimulationSeed {
    let hash = fnv1a64(fnv1a64(match_id, map_checksum), u64::from(protocol_version));
    SimulationSeed(hash)
}

fn fnv1a64(mut hash: u64, value: u64) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    if hash == 0 {
        hash = FNV_OFFSET;
    }

    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::PROTOCOL_VERSION;

    fn input(session_id: u64, command_id: u64, payload_hash: u64) -> DeterministicInput {
        DeterministicInput {
            tick: Tick(20),
            player_session_id: PlayerSessionId(session_id),
            command_id: CommandId(command_id),
            command_type: 1,
            target_tick: Tick(22),
            payload_hash,
        }
    }

    #[test]
    fn canonical_frame_sorts_inputs_by_session_and_command() {
        let frame = DeterministicInputFrame::canonical(
            Tick(20),
            vec![input(2, 1, 100), input(1, 2, 200), input(1, 1, 300)],
        );

        let order: Vec<(u64, u64)> = frame
            .inputs
            .iter()
            .map(|input| (input.player_session_id.0, input.command_id.0))
            .collect();
        assert_eq!(order, vec![(1, 1), (1, 2), (2, 1)]);
    }

    #[test]
    fn frame_checksum_is_stable_across_input_order() {
        let seed = derive_simulation_seed(7, 99, PROTOCOL_VERSION);
        let first = DeterministicInputFrame::canonical(
            Tick(20),
            vec![input(2, 1, 100), input(1, 2, 200), input(1, 1, 300)],
        );
        let second = DeterministicInputFrame::canonical(
            Tick(20),
            vec![input(1, 1, 300), input(2, 1, 100), input(1, 2, 200)],
        );

        assert_eq!(first.checksum(seed), second.checksum(seed));
    }

    #[test]
    fn frame_checksum_changes_when_payload_hash_changes() {
        let seed = derive_simulation_seed(7, 99, PROTOCOL_VERSION);
        let first = DeterministicInputFrame::canonical(Tick(20), vec![input(1, 1, 300)]);
        let second = DeterministicInputFrame::canonical(Tick(20), vec![input(1, 1, 301)]);

        assert_ne!(first.checksum(seed), second.checksum(seed));
    }

    #[test]
    fn canonical_frame_ignores_inputs_for_other_ticks() {
        let mut other_tick = input(1, 2, 200);
        other_tick.tick = Tick(21);

        let frame =
            DeterministicInputFrame::canonical(Tick(20), vec![input(1, 1, 100), other_tick]);

        assert_eq!(frame.inputs.len(), 1);
        assert_eq!(frame.inputs[0].command_id, CommandId(1));
    }
}
