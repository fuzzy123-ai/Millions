use crate::metrics::estimate_snapshot_bytes;
use crate::perf_budget::ServerPerfScenario;
use crate::simulation::{
    EntityColumns, EntityId, EntityState, MovementDelta, SnapshotBuilder, SpatialGrid, TickLoop,
    WorldPosition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationScaleScenarioId {
    Sim1kSingleClient,
    Sim5kSingleClient,
    Sim10kSingleClient,
}

impl SimulationScaleScenarioId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sim1kSingleClient => "sim_1k_single_client",
            Self::Sim5kSingleClient => "sim_5k_single_client",
            Self::Sim10kSingleClient => "sim_10k_single_client",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationScaleScenario {
    pub id: SimulationScaleScenarioId,
    pub entity_count: usize,
    pub tick_count: u64,
    pub cell_size_mm: i32,
    pub world_width_entities: usize,
}

impl SimulationScaleScenario {
    pub fn budget_scenario(self) -> ServerPerfScenario {
        match self.id {
            SimulationScaleScenarioId::Sim1kSingleClient => ServerPerfScenario::Sim1kSingleClient,
            SimulationScaleScenarioId::Sim5kSingleClient => ServerPerfScenario::Sim5kSingleClient,
            SimulationScaleScenarioId::Sim10kSingleClient => ServerPerfScenario::Sim10kSingleClient,
        }
    }
}

pub const SIMULATION_SCALE_SCENARIOS: [SimulationScaleScenario; 3] = [
    SimulationScaleScenario {
        id: SimulationScaleScenarioId::Sim1kSingleClient,
        entity_count: 1_000,
        tick_count: 4,
        cell_size_mm: 2_000,
        world_width_entities: 50,
    },
    SimulationScaleScenario {
        id: SimulationScaleScenarioId::Sim5kSingleClient,
        entity_count: 5_000,
        tick_count: 4,
        cell_size_mm: 2_000,
        world_width_entities: 100,
    },
    SimulationScaleScenario {
        id: SimulationScaleScenarioId::Sim10kSingleClient,
        entity_count: 10_000,
        tick_count: 4,
        cell_size_mm: 2_000,
        world_width_entities: 100,
    },
];

pub fn simulation_scale_scenarios() -> &'static [SimulationScaleScenario] {
    &SIMULATION_SCALE_SCENARIOS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationScaleRun {
    pub scenario_id: SimulationScaleScenarioId,
    pub entity_count: usize,
    pub tick_count: u64,
    pub final_tick: u64,
    pub occupied_cells: usize,
    pub snapshot_entities: usize,
    pub snapshot_bytes: u64,
}

pub fn run_simulation_scale_scenario(scenario: SimulationScaleScenario) -> SimulationScaleRun {
    let mut entities = build_entities(scenario);
    let mut grid = SpatialGrid::new(scenario.cell_size_mm);
    let mut tick_loop = TickLoop::foundation_default();

    for tick_index in 0..scenario.tick_count {
        for entity_index in 0..entities.len() {
            let delta = movement_delta_for(entity_index, tick_index);
            entities.apply_movement_stub(entity_index, delta);
        }
        grid.rebuild(&entities);
        tick_loop.step();
    }

    let mut snapshot_builder = SnapshotBuilder::full(1, tick_loop.current_tick());
    for entity in entities.iter_entities() {
        snapshot_builder.push_entity(entity);
    }
    let snapshot = snapshot_builder.build();

    SimulationScaleRun {
        scenario_id: scenario.id,
        entity_count: entities.len(),
        tick_count: scenario.tick_count,
        final_tick: tick_loop.current_tick().0,
        occupied_cells: grid.occupied_cells().count(),
        snapshot_entities: snapshot.entities.len(),
        snapshot_bytes: estimate_snapshot_bytes(&snapshot),
    }
}

fn build_entities(scenario: SimulationScaleScenario) -> EntityColumns {
    let mut entities = EntityColumns::with_capacity(scenario.entity_count);
    for index in 0..scenario.entity_count {
        entities.push(EntityState {
            entity_id: EntityId(index as u64 + 1),
            entity_kind: (index % 8) as u16,
            faction_id: (index % 4) as u16,
            flags: 0,
            position: initial_position(index, scenario.world_width_entities),
            facing_millirad: ((index % 6283) as i32),
            health_q8: 256,
            state_id: 0,
            state_param_q8: 0,
        });
    }
    entities
}

fn initial_position(index: usize, width: usize) -> WorldPosition {
    let x = (index % width) as i32;
    let y = (index / width) as i32;
    WorldPosition {
        x_mm: x * 1_000,
        y_mm: y * 1_000,
    }
}

fn movement_delta_for(entity_index: usize, tick_index: u64) -> MovementDelta {
    let lane = ((entity_index + tick_index as usize) % 5) as i32 - 2;
    let band = (((entity_index / 5) + tick_index as usize) % 5) as i32 - 2;
    MovementDelta {
        dx_mm: lane * 10,
        dy_mm: band * 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_scenario_catalog_covers_required_counts() {
        let counts = simulation_scale_scenarios()
            .iter()
            .map(|scenario| (scenario.id.as_str(), scenario.entity_count))
            .collect::<Vec<_>>();

        assert_eq!(
            counts,
            vec![
                ("sim_1k_single_client", 1_000),
                ("sim_5k_single_client", 5_000),
                ("sim_10k_single_client", 10_000),
            ]
        );
    }

    #[test]
    fn simulation_scale_run_is_deterministic_for_10k() {
        let scenario = SIMULATION_SCALE_SCENARIOS[2];
        let first = run_simulation_scale_scenario(scenario);
        let second = run_simulation_scale_scenario(scenario);

        assert_eq!(first, second);
        assert_eq!(
            first.scenario_id,
            SimulationScaleScenarioId::Sim10kSingleClient
        );
        assert_eq!(first.entity_count, 10_000);
        assert_eq!(first.snapshot_entities, 10_000);
        assert_eq!(first.final_tick, scenario.tick_count);
        assert!(first.occupied_cells > 0);
        assert!(first.snapshot_bytes > 10_000);
    }
}
