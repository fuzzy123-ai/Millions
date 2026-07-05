use std::collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque};

use crate::collision::{
    CollisionAdmissionResult, CollisionBody, CollisionBodyKind, CollisionMovementDecision,
    CollisionResolvedAdmissionResult, CollisionWorld,
};
use crate::simulation::{
    EntityColumns, EntityId, EntityState, MovementDelta, SpatialCell, SpatialGrid, TickLoop,
    WorldPosition,
};

pub const MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM: &str = "sum16:3e03";
const FLOW_FIELD_CACHE_ENTRY_LIMIT: usize = 64;
const FLOW_FIELD_COLLISION_SAMPLE_COUNT: usize = 16;
const FLOW_FIELD_COLLISION_RADIUS_MM: i32 = 300;
const FLOW_FIELD_COLLISION_RESOLUTION_ITERATIONS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementOptionFamily {
    DirectTargetSteering,
    GridCorridorPathing,
    FlowFieldObjective,
    FormationAnchor,
    LocalAvoidanceLayer,
}

impl MovementOptionFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectTargetSteering => "direct_target_steering",
            Self::GridCorridorPathing => "grid_corridor_pathing",
            Self::FlowFieldObjective => "flow_field_objective",
            Self::FormationAnchor => "formation_anchor",
            Self::LocalAvoidanceLayer => "local_avoidance_layer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementScaleScenarioId {
    SharedTarget1kDirect,
    ManyGroups5kCorridor,
    SharedObjective10kFlowField,
    Choke5kAvoidance,
    Formation1kAnchor,
}

impl MovementScaleScenarioId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SharedTarget1kDirect => "nav_shared_target_1k_direct",
            Self::ManyGroups5kCorridor => "nav_many_groups_5k_corridor",
            Self::SharedObjective10kFlowField => "nav_shared_objective_10k_flow_field",
            Self::Choke5kAvoidance => "nav_choke_5k_avoidance",
            Self::Formation1kAnchor => "nav_formation_1k_anchor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementScaleScenario {
    pub id: MovementScaleScenarioId,
    pub option_family: MovementOptionFamily,
    pub entity_count: usize,
    pub tick_count: u64,
    pub group_count: usize,
    pub cell_size_mm: i32,
    pub blocker_cell_count: usize,
    pub map_checksum: &'static str,
}

pub const MOVEMENT_SCALE_SCENARIOS: [MovementScaleScenario; 5] = [
    MovementScaleScenario {
        id: MovementScaleScenarioId::SharedTarget1kDirect,
        option_family: MovementOptionFamily::DirectTargetSteering,
        entity_count: 1_000,
        tick_count: 8,
        group_count: 1,
        cell_size_mm: 2_000,
        blocker_cell_count: 0,
        map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
    },
    MovementScaleScenario {
        id: MovementScaleScenarioId::ManyGroups5kCorridor,
        option_family: MovementOptionFamily::GridCorridorPathing,
        entity_count: 5_000,
        tick_count: 8,
        group_count: 25,
        cell_size_mm: 2_000,
        blocker_cell_count: 16,
        map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
    },
    MovementScaleScenario {
        id: MovementScaleScenarioId::SharedObjective10kFlowField,
        option_family: MovementOptionFamily::FlowFieldObjective,
        entity_count: 10_000,
        tick_count: 8,
        group_count: 4,
        cell_size_mm: 2_000,
        blocker_cell_count: 32,
        map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
    },
    MovementScaleScenario {
        id: MovementScaleScenarioId::Choke5kAvoidance,
        option_family: MovementOptionFamily::LocalAvoidanceLayer,
        entity_count: 5_000,
        tick_count: 8,
        group_count: 10,
        cell_size_mm: 2_000,
        blocker_cell_count: 24,
        map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
    },
    MovementScaleScenario {
        id: MovementScaleScenarioId::Formation1kAnchor,
        option_family: MovementOptionFamily::FormationAnchor,
        entity_count: 1_000,
        tick_count: 8,
        group_count: 20,
        cell_size_mm: 2_000,
        blocker_cell_count: 8,
        map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
    },
];

pub fn movement_scale_scenarios() -> &'static [MovementScaleScenario] {
    &MOVEMENT_SCALE_SCENARIOS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementScaleRun {
    pub scenario_id: MovementScaleScenarioId,
    pub option_family: MovementOptionFamily,
    pub entity_count: usize,
    pub tick_count: u64,
    pub final_tick: u64,
    pub movement_queries: u64,
    pub correction_count: u64,
    pub occupied_cells: usize,
    pub blocker_cell_count: usize,
    pub flow_field_build_count: u64,
    pub flow_field_cache_hit_count: u64,
    pub flow_field_cache_request_count: u64,
    pub flow_field_cache_eviction_count: u64,
    pub flow_field_cache_entry_count: usize,
    pub flow_field_query_count: u64,
    pub flow_field_visited_cell_count: usize,
    pub flow_field_blocked_cell_count: usize,
    pub flow_field_unreachable_count: u64,
    pub flow_field_collision_admission_check_count: u64,
    pub flow_field_collision_admission_accepted_count: u64,
    pub flow_field_collision_admission_rejected_count: u64,
    pub flow_field_collision_resolved_admission_check_count: u64,
    pub flow_field_collision_resolved_admission_accepted_after_resolution_count: u64,
    pub flow_field_collision_resolved_admission_rejected_count: u64,
    pub flow_field_collision_resolved_admission_iterations_run_count: u64,
    pub flow_field_collision_resolved_admission_correction_count: u64,
    pub flow_field_collision_resolved_admission_correction_abs_mm_total: u64,
    pub flow_field_collision_resolved_admission_max_correction_abs_mm: u32,
    pub flow_field_collision_movement_probe_count: u64,
    pub flow_field_collision_movement_probe_corrected_count: u64,
    pub flow_field_collision_movement_probe_blocked_count: u64,
    pub flow_field_collision_movement_applied_delta_count: u64,
    pub flow_field_collision_movement_corrected_delta_count: u64,
    pub flow_field_collision_movement_blocked_delta_count: u64,
    pub flow_field_collision_apply_physics_candidate_count: u64,
    pub flow_field_collision_apply_physics_initial_contact_count: u64,
    pub flow_field_collision_apply_physics_iterations_run_count: u64,
    pub flow_field_collision_apply_physics_correction_count: u64,
    pub flow_field_collision_apply_physics_correction_abs_mm_total: u64,
    pub flow_field_collision_apply_physics_max_correction_abs_mm: u32,
    pub flow_field_collision_apply_physics_final_contact_count: u64,
    pub flow_field_collision_apply_physics_synced_position_count: u64,
    pub flow_field_static_obstacle_body_count: usize,
    pub map_checksum: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowFieldError {
    InvalidBounds,
    GoalBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FlowFieldBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl FlowFieldBounds {
    pub fn contains(self, cell: SpatialCell) -> bool {
        cell.x >= self.min_x && cell.x <= self.max_x && cell.y >= self.min_y && cell.y <= self.max_y
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowFieldRequest {
    pub bounds: FlowFieldBounds,
    pub goal_cell: SpatialCell,
    pub blocked_cells: Vec<SpatialCell>,
    pub map_checksum: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlowFieldCacheStats {
    pub request_count: u64,
    pub build_count: u64,
    pub hit_count: u64,
    pub eviction_count: u64,
    pub invalidated_entry_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FlowFieldCacheKey {
    bounds: FlowFieldBounds,
    goal_cell: SpatialCell,
    blocked_cells: BTreeSet<SpatialCell>,
    map_checksum: &'static str,
}

impl FlowFieldRequest {
    fn cache_key(self) -> FlowFieldCacheKey {
        FlowFieldCacheKey {
            bounds: self.bounds,
            goal_cell: self.goal_cell,
            blocked_cells: self.blocked_cells.into_iter().collect(),
            map_checksum: self.map_checksum,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowFieldCache {
    fields: BTreeMap<FlowFieldCacheKey, FlowFieldMap>,
    stats: FlowFieldCacheStats,
}

impl FlowFieldCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn stats(&self) -> FlowFieldCacheStats {
        self.stats
    }

    pub fn get_or_build(
        &mut self,
        request: FlowFieldRequest,
    ) -> Result<&FlowFieldMap, FlowFieldError> {
        self.stats.request_count += 1;
        let key = request.cache_key();
        if !self.fields.contains_key(&key) && self.fields.len() >= FLOW_FIELD_CACHE_ENTRY_LIMIT {
            if let Some(evicted_key) = self.fields.keys().next().cloned() {
                self.fields.remove(&evicted_key);
                self.stats.eviction_count += 1;
            }
        }
        match self.fields.entry(key) {
            Entry::Occupied(entry) => {
                self.stats.hit_count += 1;
                Ok(entry.into_mut())
            }
            Entry::Vacant(entry) => {
                let key = entry.key();
                let field = FlowFieldMap::build(
                    key.bounds,
                    key.goal_cell,
                    key.blocked_cells.iter().copied(),
                )?;
                self.stats.build_count += 1;
                Ok(entry.insert(field))
            }
        }
    }

    pub fn invalidate_map_checksum(&mut self, map_checksum: &'static str) -> usize {
        let before = self.fields.len();
        self.fields
            .retain(|key, _| key.map_checksum != map_checksum);
        let removed = before.saturating_sub(self.fields.len());
        self.stats.invalidated_entry_count += removed as u64;
        removed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowFieldStepResult {
    AtGoal,
    NextCell,
    Blocked,
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowFieldStep {
    pub from_cell: SpatialCell,
    pub next_cell: Option<SpatialCell>,
    pub movement_delta: MovementDelta,
    pub cost: Option<u32>,
    pub result: FlowFieldStepResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowFieldMap {
    bounds: FlowFieldBounds,
    goal_cell: SpatialCell,
    blocked_cells: BTreeSet<SpatialCell>,
    costs: BTreeMap<SpatialCell, u32>,
}

impl FlowFieldMap {
    pub fn build(
        bounds: FlowFieldBounds,
        goal_cell: SpatialCell,
        blocked_cells: impl IntoIterator<Item = SpatialCell>,
    ) -> Result<Self, FlowFieldError> {
        if bounds.min_x > bounds.max_x || bounds.min_y > bounds.max_y {
            return Err(FlowFieldError::InvalidBounds);
        }

        let blocked_cells = blocked_cells
            .into_iter()
            .filter(|cell| bounds.contains(*cell))
            .collect::<BTreeSet<_>>();
        if blocked_cells.contains(&goal_cell) {
            return Err(FlowFieldError::GoalBlocked);
        }

        let mut costs = BTreeMap::new();
        let mut open = VecDeque::new();
        costs.insert(goal_cell, 0);
        open.push_back(goal_cell);

        while let Some(cell) = open.pop_front() {
            let cost = costs.get(&cell).copied().unwrap_or(0);
            for neighbor in flow_neighbors(cell) {
                if !bounds.contains(neighbor)
                    || blocked_cells.contains(&neighbor)
                    || costs.contains_key(&neighbor)
                {
                    continue;
                }
                costs.insert(neighbor, cost + 1);
                open.push_back(neighbor);
            }
        }

        Ok(Self {
            bounds,
            goal_cell,
            blocked_cells,
            costs,
        })
    }

    pub fn goal_cell(&self) -> SpatialCell {
        self.goal_cell
    }

    pub fn visited_cell_count(&self) -> usize {
        self.costs.len()
    }

    pub fn blocked_cell_count(&self) -> usize {
        self.blocked_cells.len()
    }

    pub fn cost_at(&self, cell: SpatialCell) -> Option<u32> {
        self.costs.get(&cell).copied()
    }

    pub fn step_from(&self, cell: SpatialCell, cell_size_mm: i32) -> FlowFieldStep {
        if self.blocked_cells.contains(&cell) {
            return FlowFieldStep {
                from_cell: cell,
                next_cell: None,
                movement_delta: MovementDelta { dx_mm: 0, dy_mm: 0 },
                cost: None,
                result: FlowFieldStepResult::Blocked,
            };
        }

        let Some(cost) = self.cost_at(cell) else {
            return FlowFieldStep {
                from_cell: cell,
                next_cell: None,
                movement_delta: MovementDelta { dx_mm: 0, dy_mm: 0 },
                cost: None,
                result: FlowFieldStepResult::Unreachable,
            };
        };

        if cell == self.goal_cell {
            return FlowFieldStep {
                from_cell: cell,
                next_cell: None,
                movement_delta: MovementDelta { dx_mm: 0, dy_mm: 0 },
                cost: Some(cost),
                result: FlowFieldStepResult::AtGoal,
            };
        }

        let next_cell = flow_neighbors(cell)
            .into_iter()
            .filter_map(|neighbor| {
                self.cost_at(neighbor)
                    .map(|neighbor_cost| (neighbor, neighbor_cost))
            })
            .filter(|(_, neighbor_cost)| *neighbor_cost < cost)
            .min_by_key(|(neighbor, neighbor_cost)| (*neighbor_cost, neighbor.x, neighbor.y))
            .map(|(neighbor, _)| neighbor);
        let Some(next_cell) = next_cell else {
            return FlowFieldStep {
                from_cell: cell,
                next_cell: None,
                movement_delta: MovementDelta { dx_mm: 0, dy_mm: 0 },
                cost: Some(cost),
                result: FlowFieldStepResult::Unreachable,
            };
        };

        FlowFieldStep {
            from_cell: cell,
            next_cell: Some(next_cell),
            movement_delta: MovementDelta {
                dx_mm: (next_cell.x - cell.x).signum() * cell_size_mm.min(250),
                dy_mm: (next_cell.y - cell.y).signum() * cell_size_mm.min(250),
            },
            cost: Some(cost),
            result: FlowFieldStepResult::NextCell,
        }
    }
}

fn flow_neighbors(cell: SpatialCell) -> [SpatialCell; 4] {
    [
        SpatialCell {
            x: cell.x - 1,
            y: cell.y,
        },
        SpatialCell {
            x: cell.x,
            y: cell.y - 1,
        },
        SpatialCell {
            x: cell.x,
            y: cell.y + 1,
        },
        SpatialCell {
            x: cell.x + 1,
            y: cell.y,
        },
    ]
}

pub fn run_movement_scale_scenario(scenario: MovementScaleScenario) -> MovementScaleRun {
    let mut entities = build_entities(scenario);
    let mut grid = SpatialGrid::new(scenario.cell_size_mm);
    let mut tick_loop = TickLoop::foundation_default();
    let mut movement_queries = 0u64;
    let mut correction_count = 0u64;
    let is_flow_field_scenario = scenario.option_family == MovementOptionFamily::FlowFieldObjective;
    let mut flow_field_cache = FlowFieldCache::new();
    let mut flow_field_query_count = 0u64;
    let mut flow_field_visited_cell_count = 0usize;
    let mut flow_field_blocked_cell_count = 0usize;
    let mut flow_field_unreachable_count = 0u64;
    let flow_field_collision_sample_indices = if is_flow_field_scenario {
        flow_field_collision_sample_indices(scenario.entity_count)
    } else {
        BTreeSet::new()
    };
    let mut flow_field_collision_admission_check_count = 0u64;
    let mut flow_field_collision_admission_accepted_count = 0u64;
    let mut flow_field_collision_admission_rejected_count = 0u64;
    let mut flow_field_collision_resolved_admission_check_count = 0u64;
    let mut flow_field_collision_resolved_admission_accepted_after_resolution_count = 0u64;
    let mut flow_field_collision_resolved_admission_rejected_count = 0u64;
    let mut flow_field_collision_resolved_admission_iterations_run_count = 0u64;
    let mut flow_field_collision_resolved_admission_correction_count = 0u64;
    let mut flow_field_collision_resolved_admission_correction_abs_mm_total = 0u64;
    let mut flow_field_collision_resolved_admission_max_correction_abs_mm = 0u32;
    let mut flow_field_collision_movement_probe_count = 0u64;
    let mut flow_field_collision_movement_probe_corrected_count = 0u64;
    let mut flow_field_collision_movement_probe_blocked_count = 0u64;
    let mut flow_field_collision_movement_applied_delta_count = 0u64;
    let mut flow_field_collision_movement_corrected_delta_count = 0u64;
    let mut flow_field_collision_movement_blocked_delta_count = 0u64;
    let mut flow_field_collision_apply_physics_candidate_count = 0u64;
    let mut flow_field_collision_apply_physics_initial_contact_count = 0u64;
    let mut flow_field_collision_apply_physics_iterations_run_count = 0u64;
    let mut flow_field_collision_apply_physics_correction_count = 0u64;
    let mut flow_field_collision_apply_physics_correction_abs_mm_total = 0u64;
    let mut flow_field_collision_apply_physics_max_correction_abs_mm = 0u32;
    let mut flow_field_collision_apply_physics_final_contact_count = 0u64;
    let mut flow_field_collision_apply_physics_synced_position_count = 0u64;
    let mut flow_field_static_obstacle_body_count = 0usize;

    for tick_index in 0..scenario.tick_count {
        let flow_field = if is_flow_field_scenario {
            let field = flow_field_cache
                .get_or_build(flow_field_request_for(scenario))
                .expect("movement scale flow-field scenario uses a reachable goal")
                .clone();
            flow_field_visited_cell_count = field.visited_cell_count();
            flow_field_blocked_cell_count = field.blocked_cell_count();
            Some(field)
        } else {
            None
        };
        let mut collision_world = if is_flow_field_scenario {
            let world = flow_field_collision_world_for(
                scenario,
                &entities,
                &flow_field_collision_sample_indices,
            );
            flow_field_static_obstacle_body_count = scenario.blocker_cell_count;
            Some(world)
        } else {
            None
        };
        let mut tick_apply_physics_candidate_count = 0u64;

        for entity_index in 0..entities.len() {
            movement_queries += movement_query_cost(scenario, entity_index);
            correction_count += correction_cost(scenario, entity_index, tick_index);
            let delta = if let Some(flow_field) = &flow_field {
                flow_field_query_count += 1;
                let position = entities
                    .position(entity_index)
                    .expect("entity index is valid during movement scale run");
                let step =
                    flow_field.step_from(grid.cell_for_position(position), scenario.cell_size_mm);
                if matches!(
                    step.result,
                    FlowFieldStepResult::Blocked | FlowFieldStepResult::Unreachable
                ) {
                    flow_field_unreachable_count += 1;
                }
                let mut delta = step.movement_delta;
                if flow_field_collision_sample_indices.contains(&entity_index) {
                    let candidate_position = WorldPosition {
                        x_mm: position.x_mm.saturating_add(step.movement_delta.dx_mm),
                        y_mm: position.y_mm.saturating_add(step.movement_delta.dy_mm),
                    };
                    let world = collision_world
                        .as_mut()
                        .expect("flow-field collision world is present");
                    let movement_probe = world.probe_movement_after_resolution(
                        EntityId(entity_index as u64 + 1),
                        candidate_position,
                        FLOW_FIELD_COLLISION_RESOLUTION_ITERATIONS,
                    );
                    flow_field_collision_movement_probe_count += 1;
                    match movement_probe.decision {
                        CollisionMovementDecision::Corrected => {
                            flow_field_collision_movement_probe_corrected_count += 1;
                            flow_field_collision_movement_applied_delta_count += 1;
                            flow_field_collision_movement_corrected_delta_count += 1;
                            delta = movement_probe
                                .resolved_delta
                                .expect("corrected movement probe has resolved delta");
                            if let Some(mut body) = world.body(EntityId(entity_index as u64 + 1)) {
                                body.position = movement_probe
                                    .resolved_position
                                    .expect("corrected movement probe has resolved position");
                                world
                                    .insert_or_update(body)
                                    .expect("flow-field sample body keeps a valid radius");
                                flow_field_collision_apply_physics_candidate_count += 1;
                                tick_apply_physics_candidate_count += 1;
                            }
                        }
                        CollisionMovementDecision::Blocked => {
                            flow_field_collision_movement_probe_blocked_count += 1;
                            flow_field_collision_movement_blocked_delta_count += 1;
                            delta = MovementDelta { dx_mm: 0, dy_mm: 0 };
                        }
                        CollisionMovementDecision::Accepted => {
                            flow_field_collision_movement_applied_delta_count += 1;
                            delta = movement_probe
                                .resolved_delta
                                .expect("accepted movement probe has resolved delta");
                            if let Some(mut body) = world.body(EntityId(entity_index as u64 + 1)) {
                                body.position = movement_probe
                                    .resolved_position
                                    .expect("accepted movement probe has resolved position");
                                world
                                    .insert_or_update(body)
                                    .expect("flow-field sample body keeps a valid radius");
                                flow_field_collision_apply_physics_candidate_count += 1;
                                tick_apply_physics_candidate_count += 1;
                            }
                        }
                        CollisionMovementDecision::UnknownBody => {
                            flow_field_collision_movement_blocked_delta_count += 1;
                            delta = MovementDelta { dx_mm: 0, dy_mm: 0 };
                        }
                    }
                    flow_field_collision_admission_check_count += 1;
                    match movement_probe.initial_result {
                        CollisionAdmissionResult::Accepted => {
                            flow_field_collision_admission_accepted_count += 1;
                        }
                        CollisionAdmissionResult::RejectedOverlap => {
                            flow_field_collision_admission_rejected_count += 1;
                        }
                        CollisionAdmissionResult::UnknownBody => {}
                    }
                    flow_field_collision_resolved_admission_check_count += 1;
                    flow_field_collision_resolved_admission_iterations_run_count +=
                        movement_probe.iterations_run as u64;
                    flow_field_collision_resolved_admission_correction_count +=
                        movement_probe.applied_correction_count as u64;
                    flow_field_collision_resolved_admission_correction_abs_mm_total =
                        flow_field_collision_resolved_admission_correction_abs_mm_total
                            .saturating_add(movement_probe.applied_correction_abs_mm_total);
                    flow_field_collision_resolved_admission_max_correction_abs_mm =
                        flow_field_collision_resolved_admission_max_correction_abs_mm
                            .max(movement_probe.max_applied_correction_abs_mm);
                    match movement_probe.resolved_result {
                        CollisionResolvedAdmissionResult::AcceptedAfterResolution => {
                            flow_field_collision_resolved_admission_accepted_after_resolution_count += 1;
                        }
                        CollisionResolvedAdmissionResult::RejectedStillOverlapping => {
                            flow_field_collision_resolved_admission_rejected_count += 1;
                        }
                        CollisionResolvedAdmissionResult::Accepted
                        | CollisionResolvedAdmissionResult::UnknownBody => {}
                    }
                }
                delta
            } else {
                scenario_delta(scenario, entity_index, tick_index)
            };
            entities.apply_movement_stub(entity_index, delta);
        }
        if let Some(world) = collision_world.as_mut() {
            if tick_apply_physics_candidate_count > 0 {
                let physics_step =
                    world.step_overlap_resolution(FLOW_FIELD_COLLISION_RESOLUTION_ITERATIONS);
                flow_field_collision_apply_physics_initial_contact_count +=
                    physics_step.initial_contact_count as u64;
                flow_field_collision_apply_physics_iterations_run_count +=
                    physics_step.iterations_run as u64;
                flow_field_collision_apply_physics_correction_count +=
                    physics_step.applied_correction_count as u64;
                flow_field_collision_apply_physics_correction_abs_mm_total =
                    flow_field_collision_apply_physics_correction_abs_mm_total
                        .saturating_add(physics_step.applied_correction_abs_mm_total);
                flow_field_collision_apply_physics_max_correction_abs_mm =
                    flow_field_collision_apply_physics_max_correction_abs_mm
                        .max(physics_step.max_applied_correction_abs_mm);
                flow_field_collision_apply_physics_final_contact_count +=
                    physics_step.final_contact_count as u64;
                flow_field_collision_apply_physics_synced_position_count +=
                    sync_flow_field_collision_sample_positions(
                        world,
                        &mut entities,
                        &flow_field_collision_sample_indices,
                    );
            }
        }
        grid.rebuild(&entities);
        tick_loop.step();
    }
    let flow_field_cache_stats = flow_field_cache.stats();

    MovementScaleRun {
        scenario_id: scenario.id,
        option_family: scenario.option_family,
        entity_count: entities.len(),
        tick_count: scenario.tick_count,
        final_tick: tick_loop.current_tick().0,
        movement_queries,
        correction_count,
        occupied_cells: grid.occupied_cells().count(),
        blocker_cell_count: scenario.blocker_cell_count,
        flow_field_build_count: flow_field_cache_stats.build_count,
        flow_field_cache_hit_count: flow_field_cache_stats.hit_count,
        flow_field_cache_request_count: flow_field_cache_stats.request_count,
        flow_field_cache_eviction_count: flow_field_cache_stats.eviction_count,
        flow_field_cache_entry_count: flow_field_cache.len(),
        flow_field_query_count,
        flow_field_visited_cell_count,
        flow_field_blocked_cell_count,
        flow_field_unreachable_count,
        flow_field_collision_admission_check_count,
        flow_field_collision_admission_accepted_count,
        flow_field_collision_admission_rejected_count,
        flow_field_collision_resolved_admission_check_count,
        flow_field_collision_resolved_admission_accepted_after_resolution_count,
        flow_field_collision_resolved_admission_rejected_count,
        flow_field_collision_resolved_admission_iterations_run_count,
        flow_field_collision_resolved_admission_correction_count,
        flow_field_collision_resolved_admission_correction_abs_mm_total,
        flow_field_collision_resolved_admission_max_correction_abs_mm,
        flow_field_collision_movement_probe_count,
        flow_field_collision_movement_probe_corrected_count,
        flow_field_collision_movement_probe_blocked_count,
        flow_field_collision_movement_applied_delta_count,
        flow_field_collision_movement_corrected_delta_count,
        flow_field_collision_movement_blocked_delta_count,
        flow_field_collision_apply_physics_candidate_count,
        flow_field_collision_apply_physics_initial_contact_count,
        flow_field_collision_apply_physics_iterations_run_count,
        flow_field_collision_apply_physics_correction_count,
        flow_field_collision_apply_physics_correction_abs_mm_total,
        flow_field_collision_apply_physics_max_correction_abs_mm,
        flow_field_collision_apply_physics_final_contact_count,
        flow_field_collision_apply_physics_synced_position_count,
        flow_field_static_obstacle_body_count,
        map_checksum: scenario.map_checksum,
    }
}

fn flow_field_request_for(scenario: MovementScaleScenario) -> FlowFieldRequest {
    FlowFieldRequest {
        bounds: flow_field_bounds_for(scenario),
        goal_cell: flow_field_goal_for(scenario),
        blocked_cells: flow_field_blockers_for(scenario),
        map_checksum: scenario.map_checksum,
    }
}

fn flow_field_bounds_for(scenario: MovementScaleScenario) -> FlowFieldBounds {
    let width = scenario.entity_count.isqrt().max(1) as i32;
    let height = ((scenario.entity_count + width as usize - 1) / width as usize) as i32;
    FlowFieldBounds {
        min_x: -4,
        min_y: -4,
        max_x: width / 2 + 8,
        max_y: height / 2 + 8,
    }
}

fn flow_field_goal_for(scenario: MovementScaleScenario) -> SpatialCell {
    let bounds = flow_field_bounds_for(scenario);
    SpatialCell {
        x: bounds.max_x - 2,
        y: bounds.max_y - 2,
    }
}

fn flow_field_blockers_for(scenario: MovementScaleScenario) -> Vec<SpatialCell> {
    let bounds = flow_field_bounds_for(scenario);
    let goal = flow_field_goal_for(scenario);
    (0..scenario.blocker_cell_count)
        .filter_map(|index| {
            let cell = SpatialCell {
                x: bounds.max_x - 12 + (index as i32 % 8),
                y: bounds.max_y - 12 + (index as i32 / 8),
            };
            if cell == goal {
                None
            } else {
                Some(cell)
            }
        })
        .collect()
}

fn flow_field_collision_sample_indices(entity_count: usize) -> BTreeSet<usize> {
    let mut indices = BTreeSet::new();
    if entity_count == 0 {
        return indices;
    }

    for index in 0..(FLOW_FIELD_COLLISION_SAMPLE_COUNT / 2).min(entity_count) {
        indices.insert(index);
    }

    for index in (FLOW_FIELD_COLLISION_SAMPLE_COUNT / 2)..FLOW_FIELD_COLLISION_SAMPLE_COUNT {
        let spaced =
            ((index + 1) * entity_count / FLOW_FIELD_COLLISION_SAMPLE_COUNT).min(entity_count - 1);
        indices.insert(spaced);
    }

    indices
}

fn flow_field_collision_world_for(
    scenario: MovementScaleScenario,
    entities: &EntityColumns,
    sample_indices: &BTreeSet<usize>,
) -> CollisionWorld {
    let mut bodies = Vec::with_capacity(sample_indices.len() + scenario.blocker_cell_count);
    for entity_index in sample_indices {
        let Some(position) = entities.position(*entity_index) else {
            continue;
        };
        bodies.push(CollisionBody {
            entity_id: EntityId(*entity_index as u64 + 1),
            kind: CollisionBodyKind::Unit,
            position,
            radius_mm: FLOW_FIELD_COLLISION_RADIUS_MM,
        });
    }

    for (index, blocker_cell) in flow_field_blockers_for(scenario).into_iter().enumerate() {
        bodies.push(CollisionBody {
            entity_id: EntityId(800_000 + index as u64),
            kind: CollisionBodyKind::StaticObstacle,
            position: cell_center_position(blocker_cell, scenario.cell_size_mm),
            radius_mm: scenario.cell_size_mm / 2,
        });
    }

    CollisionWorld::with_bodies(scenario.cell_size_mm, bodies)
        .expect("flow-field collision smoke bodies are valid")
}

fn sync_flow_field_collision_sample_positions(
    world: &CollisionWorld,
    entities: &mut EntityColumns,
    sample_indices: &BTreeSet<usize>,
) -> u64 {
    let mut synced_position_count = 0u64;
    for entity_index in sample_indices {
        let Some(position) = entities.position(*entity_index) else {
            continue;
        };
        let Some(body) = world.body(EntityId(*entity_index as u64 + 1)) else {
            continue;
        };
        if position == body.position {
            continue;
        }

        let delta = MovementDelta {
            dx_mm: body.position.x_mm.saturating_sub(position.x_mm),
            dy_mm: body.position.y_mm.saturating_sub(position.y_mm),
        };
        if entities.apply_movement_stub(*entity_index, delta) {
            synced_position_count += 1;
        }
    }

    synced_position_count
}

fn cell_center_position(cell: SpatialCell, cell_size_mm: i32) -> WorldPosition {
    WorldPosition {
        x_mm: cell
            .x
            .saturating_mul(cell_size_mm)
            .saturating_add(cell_size_mm / 2),
        y_mm: cell
            .y
            .saturating_mul(cell_size_mm)
            .saturating_add(cell_size_mm / 2),
    }
}

fn build_entities(scenario: MovementScaleScenario) -> EntityColumns {
    let mut entities = EntityColumns::with_capacity(scenario.entity_count);
    let width = scenario.entity_count.isqrt().max(1);

    for index in 0..scenario.entity_count {
        let group = index % scenario.group_count.max(1);
        entities.push(EntityState {
            entity_id: EntityId(index as u64 + 1),
            entity_kind: (scenario.option_family as u16) + 1,
            faction_id: (group % 4) as u16,
            flags: 0,
            position: initial_position(index, width, group),
            facing_millirad: 0,
            health_q8: 256,
            state_id: 0,
            state_param_q8: group as i16,
        });
    }

    entities
}

fn initial_position(index: usize, width: usize, group: usize) -> WorldPosition {
    let x = (index % width) as i32;
    let y = (index / width) as i32;
    WorldPosition {
        x_mm: x * 500 + (group as i32 % 5) * 100,
        y_mm: y * 500 + (group as i32 / 5) * 100,
    }
}

fn movement_query_cost(scenario: MovementScaleScenario, entity_index: usize) -> u64 {
    match scenario.option_family {
        MovementOptionFamily::DirectTargetSteering => 1,
        MovementOptionFamily::GridCorridorPathing => 2 + (entity_index % 3) as u64,
        MovementOptionFamily::FlowFieldObjective => {
            if entity_index % scenario.group_count.max(1) == 0 {
                8
            } else {
                1
            }
        }
        MovementOptionFamily::FormationAnchor => {
            if entity_index % 10 == 0 {
                3
            } else {
                1
            }
        }
        MovementOptionFamily::LocalAvoidanceLayer => 4 + (entity_index % 5) as u64,
    }
}

fn correction_cost(scenario: MovementScaleScenario, entity_index: usize, tick_index: u64) -> u64 {
    match scenario.option_family {
        MovementOptionFamily::FormationAnchor => {
            u64::from((entity_index + tick_index as usize).is_multiple_of(7))
        }
        MovementOptionFamily::LocalAvoidanceLayer => {
            u64::from((entity_index + tick_index as usize).is_multiple_of(5))
        }
        _ => u64::from(scenario.blocker_cell_count > 0 && entity_index % 97 == 0),
    }
}

fn scenario_delta(
    scenario: MovementScaleScenario,
    entity_index: usize,
    tick_index: u64,
) -> MovementDelta {
    let group_lane = (entity_index % scenario.group_count.max(1)) as i32;
    let tick_lane = tick_index as i32;
    match scenario.option_family {
        MovementOptionFamily::DirectTargetSteering => MovementDelta {
            dx_mm: 20,
            dy_mm: (group_lane % 3) - 1,
        },
        MovementOptionFamily::GridCorridorPathing => MovementDelta {
            dx_mm: 12 + (group_lane % 4),
            dy_mm: ((tick_lane + group_lane) % 5) - 2,
        },
        MovementOptionFamily::FlowFieldObjective => MovementDelta {
            dx_mm: 8 + (tick_lane % 3),
            dy_mm: 8 - (group_lane % 3),
        },
        MovementOptionFamily::FormationAnchor => MovementDelta {
            dx_mm: 10 + (group_lane % 2),
            dy_mm: (entity_index % 10) as i32 - 5,
        },
        MovementOptionFamily::LocalAvoidanceLayer => MovementDelta {
            dx_mm: 6 + (group_lane % 3),
            dy_mm: ((entity_index + tick_index as usize) % 7) as i32 - 3,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_flow_bounds() -> FlowFieldBounds {
        FlowFieldBounds {
            min_x: 0,
            min_y: 0,
            max_x: 4,
            max_y: 4,
        }
    }

    #[test]
    fn movement_scenario_catalog_covers_nav_01_option_families() {
        let families = movement_scale_scenarios()
            .iter()
            .map(|scenario| scenario.option_family.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            families,
            vec![
                "direct_target_steering",
                "grid_corridor_pathing",
                "flow_field_objective",
                "local_avoidance_layer",
                "formation_anchor",
            ]
        );
    }

    #[test]
    fn movement_scenario_catalog_includes_1k_5k_and_10k_inputs() {
        let counts = movement_scale_scenarios()
            .iter()
            .map(|scenario| scenario.entity_count)
            .collect::<Vec<_>>();

        assert!(counts.contains(&1_000));
        assert!(counts.contains(&5_000));
        assert!(counts.contains(&10_000));
    }

    #[test]
    fn flow_field_builds_deterministic_costs_around_blockers() {
        let flow = FlowFieldMap::build(
            small_flow_bounds(),
            SpatialCell { x: 4, y: 4 },
            vec![SpatialCell { x: 2, y: 4 }, SpatialCell { x: 3, y: 4 }],
        )
        .expect("valid flow field");

        assert_eq!(flow.goal_cell(), SpatialCell { x: 4, y: 4 });
        assert_eq!(flow.blocked_cell_count(), 2);
        assert_eq!(flow.cost_at(SpatialCell { x: 4, y: 4 }), Some(0));
        assert_eq!(flow.cost_at(SpatialCell { x: 4, y: 3 }), Some(1));
        assert_eq!(flow.cost_at(SpatialCell { x: 2, y: 4 }), None);
        assert!(flow.visited_cell_count() > 10);
    }

    #[test]
    fn flow_field_rejects_invalid_bounds_and_blocked_goal() {
        assert_eq!(
            FlowFieldMap::build(
                FlowFieldBounds {
                    min_x: 4,
                    min_y: 0,
                    max_x: 0,
                    max_y: 4,
                },
                SpatialCell { x: 0, y: 0 },
                Vec::new(),
            )
            .unwrap_err(),
            FlowFieldError::InvalidBounds
        );

        assert_eq!(
            FlowFieldMap::build(
                small_flow_bounds(),
                SpatialCell { x: 4, y: 4 },
                vec![SpatialCell { x: 4, y: 4 }],
            )
            .unwrap_err(),
            FlowFieldError::GoalBlocked
        );
    }

    #[test]
    fn flow_field_step_returns_next_cell_blocked_and_unreachable_results() {
        let flow = FlowFieldMap::build(
            small_flow_bounds(),
            SpatialCell { x: 4, y: 4 },
            vec![SpatialCell { x: 2, y: 2 }],
        )
        .expect("valid flow field");

        assert_eq!(
            flow.step_from(SpatialCell { x: 4, y: 4 }, 1_000),
            FlowFieldStep {
                from_cell: SpatialCell { x: 4, y: 4 },
                next_cell: None,
                movement_delta: MovementDelta { dx_mm: 0, dy_mm: 0 },
                cost: Some(0),
                result: FlowFieldStepResult::AtGoal,
            }
        );
        assert_eq!(
            flow.step_from(SpatialCell { x: 3, y: 4 }, 1_000),
            FlowFieldStep {
                from_cell: SpatialCell { x: 3, y: 4 },
                next_cell: Some(SpatialCell { x: 4, y: 4 }),
                movement_delta: MovementDelta {
                    dx_mm: 250,
                    dy_mm: 0
                },
                cost: Some(1),
                result: FlowFieldStepResult::NextCell,
            }
        );
        assert_eq!(
            flow.step_from(SpatialCell { x: 2, y: 2 }, 1_000).result,
            FlowFieldStepResult::Blocked
        );
        assert_eq!(
            flow.step_from(SpatialCell { x: 8, y: 8 }, 1_000).result,
            FlowFieldStepResult::Unreachable
        );
    }

    #[test]
    fn flow_field_cache_reuses_identical_requests_and_tracks_hits() {
        let mut cache = FlowFieldCache::new();
        let request = FlowFieldRequest {
            bounds: small_flow_bounds(),
            goal_cell: SpatialCell { x: 4, y: 4 },
            blocked_cells: vec![SpatialCell { x: 2, y: 2 }],
            map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
        };

        let first_visited = cache
            .get_or_build(request.clone())
            .expect("valid flow field")
            .visited_cell_count();
        let second_visited = cache
            .get_or_build(request)
            .expect("cached flow field")
            .visited_cell_count();

        assert_eq!(first_visited, second_visited);
        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache.stats(),
            FlowFieldCacheStats {
                request_count: 2,
                build_count: 1,
                hit_count: 1,
                eviction_count: 0,
                invalidated_entry_count: 0,
            }
        );
    }

    #[test]
    fn flow_field_cache_misses_when_blockers_change_and_invalidates_by_checksum() {
        let mut cache = FlowFieldCache::new();
        let first = FlowFieldRequest {
            bounds: small_flow_bounds(),
            goal_cell: SpatialCell { x: 4, y: 4 },
            blocked_cells: vec![SpatialCell { x: 2, y: 2 }],
            map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
        };
        let second = FlowFieldRequest {
            blocked_cells: vec![SpatialCell { x: 2, y: 3 }],
            ..first.clone()
        };

        cache.get_or_build(first).expect("valid flow field");
        cache.get_or_build(second).expect("valid changed field");

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().build_count, 2);
        assert_eq!(cache.stats().hit_count, 0);
        assert_eq!(
            cache.invalidate_map_checksum(MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM),
            2
        );
        assert!(cache.is_empty());
        assert_eq!(cache.stats().invalidated_entry_count, 2);
        assert_eq!(cache.stats().eviction_count, 0);
    }

    #[test]
    fn flow_field_cache_evicts_deterministically_at_entry_limit() {
        let mut cache = FlowFieldCache::new();
        let bounds = FlowFieldBounds {
            min_x: 0,
            min_y: 0,
            max_x: FLOW_FIELD_CACHE_ENTRY_LIMIT as i32 + 4,
            max_y: 0,
        };

        for goal_x in 0..(FLOW_FIELD_CACHE_ENTRY_LIMIT + 3) {
            cache
                .get_or_build(FlowFieldRequest {
                    bounds,
                    goal_cell: SpatialCell {
                        x: goal_x as i32,
                        y: 0,
                    },
                    blocked_cells: Vec::new(),
                    map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
                })
                .expect("goal stays inside valid bounds");
        }

        let stats = cache.stats();
        assert_eq!(cache.len(), FLOW_FIELD_CACHE_ENTRY_LIMIT);
        assert_eq!(
            stats.request_count,
            (FLOW_FIELD_CACHE_ENTRY_LIMIT + 3) as u64
        );
        assert_eq!(stats.build_count, (FLOW_FIELD_CACHE_ENTRY_LIMIT + 3) as u64);
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.eviction_count, 3);
        assert!(!cache.fields.contains_key(&FlowFieldCacheKey {
            bounds,
            goal_cell: SpatialCell { x: 0, y: 0 },
            blocked_cells: BTreeSet::new(),
            map_checksum: MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM,
        }));
    }

    #[test]
    fn movement_scale_run_is_deterministic_for_10k_flow_field() {
        let scenario = MOVEMENT_SCALE_SCENARIOS[2];
        let first = run_movement_scale_scenario(scenario);
        let second = run_movement_scale_scenario(scenario);

        assert_eq!(first, second);
        assert_eq!(
            first.scenario_id,
            MovementScaleScenarioId::SharedObjective10kFlowField
        );
        assert_eq!(first.entity_count, 10_000);
        assert_eq!(first.final_tick, scenario.tick_count);
        assert!(first.movement_queries > first.entity_count as u64);
        assert!(first.correction_count > 0);
        assert!(first.occupied_cells > 0);
        assert_eq!(first.flow_field_build_count, 1);
        assert_eq!(first.flow_field_cache_request_count, first.tick_count);
        assert_eq!(first.flow_field_cache_hit_count, first.tick_count - 1);
        assert_eq!(first.flow_field_cache_eviction_count, 0);
        assert!(first.flow_field_cache_entry_count <= FLOW_FIELD_CACHE_ENTRY_LIMIT);
        assert_eq!(
            first.flow_field_query_count,
            first.entity_count as u64 * first.tick_count
        );
        assert_eq!(
            first.flow_field_collision_admission_check_count,
            FLOW_FIELD_COLLISION_SAMPLE_COUNT as u64 * first.tick_count
        );
        assert!(first.flow_field_collision_admission_accepted_count > 0);
        assert!(first.flow_field_collision_admission_rejected_count > 0);
        assert_eq!(
            first.flow_field_collision_resolved_admission_check_count,
            FLOW_FIELD_COLLISION_SAMPLE_COUNT as u64 * first.tick_count
        );
        assert!(first.flow_field_collision_resolved_admission_rejected_count > 0);
        assert!(first.flow_field_collision_resolved_admission_iterations_run_count > 0);
        assert!(first.flow_field_collision_resolved_admission_correction_count > 0);
        assert!(first.flow_field_collision_resolved_admission_correction_abs_mm_total > 0);
        assert!(first.flow_field_collision_resolved_admission_max_correction_abs_mm > 0);
        assert_eq!(
            first.flow_field_collision_movement_probe_count,
            FLOW_FIELD_COLLISION_SAMPLE_COUNT as u64 * first.tick_count
        );
        assert_eq!(
            first.flow_field_collision_movement_probe_blocked_count,
            first.flow_field_collision_resolved_admission_rejected_count
        );
        assert!(first.flow_field_collision_movement_applied_delta_count > 0);
        assert_eq!(
            first.flow_field_collision_movement_blocked_delta_count,
            first.flow_field_collision_movement_probe_blocked_count
        );
        assert_eq!(
            first.flow_field_collision_apply_physics_candidate_count,
            first.flow_field_collision_movement_applied_delta_count
        );
        assert!(first.flow_field_collision_apply_physics_candidate_count > 0);
        assert!(first.flow_field_collision_apply_physics_initial_contact_count > 0);
        assert!(first.flow_field_collision_apply_physics_iterations_run_count > 0);
        assert!(first.flow_field_collision_apply_physics_correction_count > 0);
        assert!(first.flow_field_collision_apply_physics_correction_abs_mm_total > 0);
        assert!(first.flow_field_collision_apply_physics_max_correction_abs_mm > 0);
        assert!(
            first.flow_field_collision_apply_physics_final_contact_count
                <= first.flow_field_collision_apply_physics_initial_contact_count
        );
        assert!(first.flow_field_collision_apply_physics_synced_position_count > 0);
        assert_eq!(
            first.flow_field_static_obstacle_body_count,
            scenario.blocker_cell_count
        );
        assert!(first.flow_field_visited_cell_count > 0);
        assert_eq!(
            first.flow_field_blocked_cell_count,
            scenario.blocker_cell_count
        );
        assert_eq!(first.flow_field_unreachable_count, 0);
        assert_eq!(first.map_checksum, MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM);
    }

    #[test]
    fn local_avoidance_scenario_records_higher_query_pressure() {
        let direct = run_movement_scale_scenario(MOVEMENT_SCALE_SCENARIOS[0]);
        let avoidance = run_movement_scale_scenario(MOVEMENT_SCALE_SCENARIOS[3]);

        assert!(avoidance.movement_queries > direct.movement_queries);
        assert!(avoidance.correction_count > direct.correction_count);
        assert_eq!(direct.flow_field_build_count, 0);
        assert_eq!(direct.flow_field_cache_hit_count, 0);
        assert_eq!(direct.flow_field_cache_eviction_count, 0);
        assert_eq!(direct.flow_field_cache_entry_count, 0);
        assert_eq!(direct.flow_field_collision_admission_check_count, 0);
        assert_eq!(
            direct.flow_field_collision_resolved_admission_check_count,
            0
        );
        assert_eq!(
            direct.flow_field_collision_resolved_admission_correction_abs_mm_total,
            0
        );
        assert_eq!(
            direct.flow_field_collision_resolved_admission_max_correction_abs_mm,
            0
        );
        assert_eq!(direct.flow_field_collision_movement_probe_count, 0);
        assert_eq!(direct.flow_field_collision_movement_applied_delta_count, 0);
        assert_eq!(direct.flow_field_collision_apply_physics_candidate_count, 0);
        assert_eq!(
            direct.flow_field_collision_apply_physics_correction_abs_mm_total,
            0
        );
        assert_eq!(
            direct.flow_field_collision_apply_physics_synced_position_count,
            0
        );
        assert_eq!(avoidance.flow_field_build_count, 0);
        assert_eq!(avoidance.flow_field_cache_request_count, 0);
        assert_eq!(avoidance.flow_field_cache_eviction_count, 0);
        assert_eq!(avoidance.flow_field_cache_entry_count, 0);
        assert_eq!(
            avoidance.flow_field_collision_resolved_admission_check_count,
            0
        );
        assert_eq!(
            avoidance.flow_field_collision_resolved_admission_correction_abs_mm_total,
            0
        );
        assert_eq!(
            avoidance.flow_field_collision_resolved_admission_max_correction_abs_mm,
            0
        );
        assert_eq!(avoidance.flow_field_collision_movement_probe_count, 0);
        assert_eq!(
            avoidance.flow_field_collision_movement_applied_delta_count,
            0
        );
        assert_eq!(
            avoidance.flow_field_collision_apply_physics_candidate_count,
            0
        );
        assert_eq!(
            avoidance.flow_field_collision_apply_physics_correction_abs_mm_total,
            0
        );
        assert_eq!(
            avoidance.flow_field_collision_apply_physics_synced_position_count,
            0
        );
        assert_eq!(avoidance.flow_field_static_obstacle_body_count, 0);
    }
}
