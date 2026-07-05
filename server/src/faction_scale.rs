use crate::perf_budget::ServerPerfScenario;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionScaleScenarioId {
    StageA2pNeutralZombie1k,
    StageBAiPressure2k,
    StageC4PlusMixed5k,
    StageD10kAoiLod,
}

impl FactionScaleScenarioId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StageA2pNeutralZombie1k => "gload_stage_a_2p_neutral_zombie_1k",
            Self::StageBAiPressure2k => "gload_stage_b_ai_pressure_2k",
            Self::StageC4PlusMixed5k => "gload_stage_c_4plus_mixed_5k",
            Self::StageD10kAoiLod => "gload_stage_d_10k_aoi_lod",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleMixCounts {
    pub basic: usize,
    pub support: usize,
    pub heavy: usize,
    pub siege: usize,
    pub swarm: usize,
    pub neutral_objectives: usize,
    pub synthetic_pressure: usize,
}

impl RoleMixCounts {
    pub fn total(self) -> usize {
        self.basic
            + self.support
            + self.heavy
            + self.siege
            + self.swarm
            + self.neutral_objectives
            + self.synthetic_pressure
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactionScaleScenario {
    pub id: FactionScaleScenarioId,
    pub total_entity_count: usize,
    pub player_faction_count: u16,
    pub neutral_system_count: usize,
    pub zombie_entity_count: usize,
    pub ai_pressure_group_count: u16,
    pub simulated_extra_faction_count: u16,
    pub visible_entities_per_client: usize,
    pub aggregate_far_state_required: bool,
    pub role_mix: RoleMixCounts,
}

impl FactionScaleScenario {
    pub fn faction_count(self) -> u16 {
        self.player_faction_count
            + u16::from(self.neutral_system_count > 0)
            + u16::from(self.zombie_entity_count > 0)
            + self.ai_pressure_group_count
            + self.simulated_extra_faction_count
    }

    pub fn budget_reference(self) -> ServerPerfScenario {
        if self.total_entity_count <= 1_000 {
            ServerPerfScenario::Sim1kSingleClient
        } else if self.total_entity_count <= 5_000 {
            ServerPerfScenario::Sim5kSingleClient
        } else {
            ServerPerfScenario::Sim10kSingleClient
        }
    }

    pub fn is_shape_consistent(self) -> bool {
        self.role_mix.total() == self.total_entity_count
            && self.visible_entities_per_client <= self.total_entity_count
            && self.player_faction_count >= 2
    }
}

pub const FACTION_SCALE_SCENARIOS: [FactionScaleScenario; 4] = [
    FactionScaleScenario {
        id: FactionScaleScenarioId::StageA2pNeutralZombie1k,
        total_entity_count: 1_000,
        player_faction_count: 2,
        neutral_system_count: 8,
        zombie_entity_count: 250,
        ai_pressure_group_count: 0,
        simulated_extra_faction_count: 0,
        visible_entities_per_client: 512,
        aggregate_far_state_required: false,
        role_mix: RoleMixCounts {
            basic: 620,
            support: 80,
            heavy: 32,
            siege: 10,
            swarm: 250,
            neutral_objectives: 8,
            synthetic_pressure: 0,
        },
    },
    FactionScaleScenario {
        id: FactionScaleScenarioId::StageBAiPressure2k,
        total_entity_count: 2_000,
        player_faction_count: 2,
        neutral_system_count: 12,
        zombie_entity_count: 500,
        ai_pressure_group_count: 2,
        simulated_extra_faction_count: 0,
        visible_entities_per_client: 768,
        aggregate_far_state_required: false,
        role_mix: RoleMixCounts {
            basic: 1_050,
            support: 150,
            heavy: 80,
            siege: 30,
            swarm: 500,
            neutral_objectives: 12,
            synthetic_pressure: 178,
        },
    },
    FactionScaleScenario {
        id: FactionScaleScenarioId::StageC4PlusMixed5k,
        total_entity_count: 5_000,
        player_faction_count: 2,
        neutral_system_count: 16,
        zombie_entity_count: 900,
        ai_pressure_group_count: 0,
        simulated_extra_faction_count: 2,
        visible_entities_per_client: 1_024,
        aggregate_far_state_required: true,
        role_mix: RoleMixCounts {
            basic: 2_850,
            support: 450,
            heavy: 240,
            siege: 90,
            swarm: 900,
            neutral_objectives: 16,
            synthetic_pressure: 454,
        },
    },
    FactionScaleScenario {
        id: FactionScaleScenarioId::StageD10kAoiLod,
        total_entity_count: 10_000,
        player_faction_count: 2,
        neutral_system_count: 24,
        zombie_entity_count: 2_500,
        ai_pressure_group_count: 0,
        simulated_extra_faction_count: 4,
        visible_entities_per_client: 1_500,
        aggregate_far_state_required: true,
        role_mix: RoleMixCounts {
            basic: 5_300,
            support: 800,
            heavy: 450,
            siege: 180,
            swarm: 2_500,
            neutral_objectives: 24,
            synthetic_pressure: 746,
        },
    },
];

pub fn faction_scale_scenarios() -> &'static [FactionScaleScenario] {
    &FACTION_SCALE_SCENARIOS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_scale_catalog_matches_documented_ids() {
        let ids = faction_scale_scenarios()
            .iter()
            .map(|scenario| scenario.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "gload_stage_a_2p_neutral_zombie_1k",
                "gload_stage_b_ai_pressure_2k",
                "gload_stage_c_4plus_mixed_5k",
                "gload_stage_d_10k_aoi_lod",
            ]
        );
    }

    #[test]
    fn faction_scale_role_mixes_sum_to_total_entities() {
        for scenario in faction_scale_scenarios() {
            assert!(
                scenario.is_shape_consistent(),
                "inconsistent faction scale scenario {}",
                scenario.id.as_str()
            );
        }
    }

    #[test]
    fn faction_scale_budget_references_are_conservative() {
        assert_eq!(
            FACTION_SCALE_SCENARIOS[0].budget_reference(),
            ServerPerfScenario::Sim1kSingleClient
        );
        assert_eq!(
            FACTION_SCALE_SCENARIOS[1].budget_reference(),
            ServerPerfScenario::Sim5kSingleClient
        );
        assert_eq!(
            FACTION_SCALE_SCENARIOS[2].budget_reference(),
            ServerPerfScenario::Sim5kSingleClient
        );
        assert_eq!(
            FACTION_SCALE_SCENARIOS[3].budget_reference(),
            ServerPerfScenario::Sim10kSingleClient
        );
    }
}
