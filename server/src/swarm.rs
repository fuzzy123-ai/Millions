use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::collision::{
    CollisionAdmissionResult, CollisionBatchMovementCandidate, CollisionBody, CollisionBodyKind,
    CollisionMovementDecision, CollisionWorld,
};
use crate::interest::{
    build_interest_snapshot_delta, AoiRegion, InterestManager, InterestSnapshotDelta,
};
use crate::map_data::{
    validate_map_data_import, MapBounds, MapDataImport, MapDataValidationError, MapPoint, MapShape,
    MapShapeKind,
};
use crate::metrics::{
    bandwidth_kb_s, estimate_snapshot_bytes, SNAPSHOT_ENTITY_BYTES, SNAPSHOT_HEADER_BYTES,
};
use crate::movement_scale::{FlowFieldBounds, FlowFieldMap, FlowFieldStepResult};
use crate::perf_budget::{
    evaluate_server_perf_budget, BudgetResult, ServerPerfBudgets, ServerPerfReport,
    ServerPerfScenario,
};
use crate::simulation::{
    EntityColumns, EntityId, EntityState, MovementDelta, PlayerSessionId, Snapshot,
    SnapshotBuilder, SpatialCell, SpatialGrid, Tick, WorldPosition,
};

pub const SWARM_SCHEMA: &str = "millions_swarm_v0";
pub const SWARM_ENTITY_KIND: u16 = 200;
const SWARM_COLLISION_ADMISSION_SAMPLE_COUNT: usize = 16;
const SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT: usize = 4;
const SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM: i32 = 2_000;
const SWARM_MAP_OBSTACLE_ENTITY_ID_BASE: u64 = 9_000_000_000;
const SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT: usize = 32;
const SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM: u32 = 50;
const SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT: u64 = 2;
const SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT: usize = 2;
#[cfg(test)]
const SWARM_CONFIGURED_MOVEMENT_MEASUREMENT_SAMPLE_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmMovementMode {
    SpawnOnly,
    FlowFieldCollision,
    BatchFlowFieldCollision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmConfig {
    pub start_tick: Tick,
    pub spawn_interval_ticks: u64,
    pub spawn_batch_size: usize,
    pub max_active: usize,
    pub route_pressure_radius_mm: i32,
    pub collision_radius_mm: i32,
    pub direct_aggro_ticks: u64,
    pub aggro_memory_ticks: u64,
    pub full_lod_distance_mm: i32,
    pub reduced_lod_distance_mm: i32,
    pub movement_mode: SwarmMovementMode,
    pub movement_sample_limit: usize,
    pub movement_cell_size_mm: i32,
    pub movement_physics_iterations: usize,
    pub movement_correction_limit_abs_mm: Option<u32>,
}

impl SwarmConfig {
    pub fn local_scale_smoke() -> Self {
        Self {
            start_tick: Tick(20),
            spawn_interval_ticks: 5,
            spawn_batch_size: 32,
            max_active: 1_000,
            route_pressure_radius_mm: 10_000,
            collision_radius_mm: 350,
            direct_aggro_ticks: 20,
            aggro_memory_ticks: 120,
            full_lod_distance_mm: 20_000,
            reduced_lod_distance_mm: 80_000,
            movement_mode: SwarmMovementMode::SpawnOnly,
            movement_sample_limit: SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
            movement_cell_size_mm: SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            movement_physics_iterations: 1,
            movement_correction_limit_abs_mm: None,
        }
    }

    pub fn with_flow_field_collision_movement(
        mut self,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
    ) -> Self {
        self.movement_mode = SwarmMovementMode::FlowFieldCollision;
        self.movement_sample_limit = sample_limit;
        self.movement_cell_size_mm = cell_size_mm;
        self.movement_physics_iterations = physics_iterations;
        self.movement_correction_limit_abs_mm = None;
        self
    }

    pub fn with_batch_flow_field_collision_movement(
        mut self,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
    ) -> Self {
        self.movement_mode = SwarmMovementMode::BatchFlowFieldCollision;
        self.movement_sample_limit = sample_limit;
        self.movement_cell_size_mm = cell_size_mm;
        self.movement_physics_iterations = physics_iterations;
        self.movement_correction_limit_abs_mm = None;
        self
    }

    pub fn with_flow_field_collision_movement_correction_limit(
        mut self,
        correction_limit_abs_mm: u32,
    ) -> Self {
        self.movement_correction_limit_abs_mm = Some(correction_limit_abs_mm);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmSpawnPoint {
    pub spawn_id: u16,
    pub position: WorldPosition,
    pub route_target: WorldPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmEntity {
    pub entity_id: EntityId,
    pub spawned_tick: Tick,
    pub position: WorldPosition,
    pub route_target: WorldPosition,
    pub collision_radius_mm: i32,
}

impl SwarmEntity {
    pub fn collision_body(self) -> CollisionBody {
        CollisionBody {
            entity_id: self.entity_id,
            kind: CollisionBodyKind::Swarm,
            position: self.position,
            radius_mm: self.collision_radius_mm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutePressureSample {
    pub target_position: WorldPosition,
    pub active_entity_count: usize,
    pub radius_mm: i32,
    pub nearest_distance_sq_mm: u64,
    pub farthest_distance_sq_mm: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmTickReport {
    pub schema: &'static str,
    pub tick: Tick,
    pub spawned_entity_ids: Vec<EntityId>,
    pub active_count: usize,
    pub spawn_capped: bool,
    pub route_pressure: Vec<RoutePressureSample>,
    pub aggro_trails: Vec<AggroTrailSample>,
    pub movement: Option<SwarmMovementApplyReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmError {
    EmptySpawnPoints,
    InvalidSpawnInterval,
    InvalidSpawnBatchSize,
    InvalidMaxActive,
    InvalidRoutePressureRadius,
    InvalidCollisionRadius,
    InvalidAggroWindow,
    InvalidLodDistance,
    InvalidMovementSampleLimit,
    InvalidMovementCellSize,
    InvalidMovementPhysicsIterations,
    InvalidStaticObstacle,
    InvalidMapData(MapDataValidationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggroStimulus {
    pub source_entity_id: EntityId,
    pub position: WorldPosition,
    pub tick: Tick,
    pub strength: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggroTrailLane {
    Direct,
    Memory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggroTrailSample {
    pub source_entity_id: EntityId,
    pub position: WorldPosition,
    pub last_seen_tick: Tick,
    pub age_ticks: u64,
    pub strength: u16,
    pub lane: AggroTrailLane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmAiLod {
    Full,
    Reduced,
    Aggregate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmIntentKind {
    RoutePressure,
    AggroDirect,
    AggroMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmBehaviorSample {
    pub entity_id: EntityId,
    pub intent_kind: SwarmIntentKind,
    pub intent_target: WorldPosition,
    pub ai_lod: SwarmAiLod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmAiLodCounts {
    pub full: usize,
    pub reduced: usize,
    pub aggregate: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmBehaviorReport {
    pub schema: &'static str,
    pub tick: Tick,
    pub focus: WorldPosition,
    pub samples: Vec<SwarmBehaviorSample>,
    pub lod_counts: SwarmAiLodCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmMovementPreviewSample {
    pub entity_id: EntityId,
    pub intent_kind: SwarmIntentKind,
    pub from_position: WorldPosition,
    pub intent_target: WorldPosition,
    pub flow_field_result: FlowFieldStepResult,
    pub requested_delta: MovementDelta,
    pub resolved_delta: MovementDelta,
    pub collision_decision: CollisionMovementDecision,
    pub collision_iterations_run: usize,
    pub collision_applied_correction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmMovementPreviewReport {
    pub schema: &'static str,
    pub tick: Tick,
    pub sample_count: usize,
    pub flow_field_build_count: usize,
    pub flow_field_query_count: usize,
    pub flow_field_unreachable_count: usize,
    pub physics_candidate_count: usize,
    pub physics_initial_contact_count: usize,
    pub physics_iterations_run: usize,
    pub physics_applied_correction_count: usize,
    pub physics_applied_correction_abs_mm_total: u64,
    pub physics_max_applied_correction_abs_mm: u32,
    pub physics_final_contact_count: usize,
    pub applied_delta_count: usize,
    pub corrected_delta_count: usize,
    pub blocked_delta_count: usize,
    pub samples: Vec<SwarmMovementPreviewSample>,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmMovementApplySample {
    pub entity_id: EntityId,
    pub intent_kind: SwarmIntentKind,
    pub from_position: WorldPosition,
    pub intent_target: WorldPosition,
    pub flow_field_result: FlowFieldStepResult,
    pub requested_delta: MovementDelta,
    pub applied_delta: MovementDelta,
    pub collision_decision: CollisionMovementDecision,
    pub final_position: WorldPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmMovementApplyReport {
    pub schema: &'static str,
    pub tick: Tick,
    pub sample_count: usize,
    pub flow_field_build_count: usize,
    pub flow_field_cache_request_count: u64,
    pub flow_field_cache_hit_count: u64,
    pub flow_field_cache_eviction_count: u64,
    pub flow_field_cache_entry_count: usize,
    pub flow_field_query_count: usize,
    pub flow_field_unreachable_count: usize,
    pub applied_delta_count: usize,
    pub corrected_delta_count: usize,
    pub blocked_delta_count: usize,
    pub movement_probe_correction_limit_abs_mm: Option<u32>,
    pub movement_probe_clamped_correction_count: usize,
    pub physics_candidate_count: usize,
    pub physics_initial_contact_count: usize,
    pub physics_iterations_run: usize,
    pub physics_applied_correction_count: usize,
    pub physics_applied_correction_abs_mm_total: u64,
    pub physics_max_applied_correction_abs_mm: u32,
    pub physics_correction_limit_abs_mm: Option<u32>,
    pub physics_clamped_correction_count: usize,
    pub physics_final_contact_count: usize,
    pub physics_synced_position_count: usize,
    pub physics_sample_synced_position_count: usize,
    pub samples: Vec<SwarmMovementApplySample>,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmMovementTickReport {
    pub schema: &'static str,
    pub tick: Tick,
    pub tick_report: SwarmTickReport,
    pub movement_report: SwarmMovementApplyReport,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmMovementDeltaSnapshotReport {
    pub schema: &'static str,
    pub tick: Tick,
    pub snapshot: Snapshot,
    pub visible_entity_count: usize,
    pub removed_entity_count: usize,
    pub aggregate_far_state_count: usize,
    pub visible_entity_ids: BTreeSet<EntityId>,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwarmLoadSmokeRun {
    pub tick_count: u64,
    pub active_count: usize,
    pub spawned_total: usize,
    pub local_smoke_total_elapsed_us: u64,
    pub spawn_ticks_elapsed_us: u64,
    pub behavior_elapsed_us: u64,
    pub movement_preview_elapsed_us: u64,
    pub movement_tick_elapsed_us: u64,
    pub batch_movement_tick_elapsed_us: u64,
    pub configured_movement_tick_elapsed_us: u64,
    pub configured_batch_movement_tick_elapsed_us: u64,
    pub configured_movement_loop_elapsed_us: u64,
    pub static_obstacle_movement_elapsed_us: u64,
    pub snapshot_elapsed_us: u64,
    pub collision_diagnostics_elapsed_us: u64,
    pub behavior_sample_count: usize,
    pub route_pressure_bucket_count: usize,
    pub aggro_trail_count: usize,
    pub direct_aggro_count: usize,
    pub memory_aggro_count: usize,
    pub ai_lod_counts: SwarmAiLodCounts,
    pub collision_body_count: usize,
    pub collision_contact_count: usize,
    pub collision_admission_check_count: usize,
    pub collision_admission_rejected_count: usize,
    pub collision_resolved_admission_check_count: usize,
    pub collision_resolved_admission_accepted_after_resolution_count: usize,
    pub collision_resolved_admission_rejected_count: usize,
    pub collision_resolved_admission_iterations_run_count: usize,
    pub collision_resolved_admission_correction_count: usize,
    pub collision_movement_probe_count: usize,
    pub collision_movement_probe_corrected_count: usize,
    pub collision_movement_probe_blocked_count: usize,
    pub collision_batch_movement_probe_count: usize,
    pub collision_batch_movement_probe_accepted_count: usize,
    pub collision_batch_movement_probe_corrected_count: usize,
    pub collision_batch_movement_probe_blocked_count: usize,
    pub collision_batch_movement_probe_unknown_body_count: usize,
    pub collision_batch_movement_probe_iterations_run_count: usize,
    pub collision_batch_movement_probe_correction_count: usize,
    pub collision_batch_movement_probe_correction_abs_mm_total: u64,
    pub collision_batch_movement_probe_max_correction_abs_mm: u32,
    pub collision_batch_movement_probe_final_contact_count: usize,
    pub movement_preview_sample_count: usize,
    pub movement_preview_flow_field_build_count: usize,
    pub movement_preview_flow_field_query_count: usize,
    pub movement_preview_flow_field_unreachable_count: usize,
    pub movement_preview_physics_candidate_count: usize,
    pub movement_preview_physics_initial_contact_count: usize,
    pub movement_preview_physics_iterations_run: usize,
    pub movement_preview_physics_applied_correction_count: usize,
    pub movement_preview_physics_applied_correction_abs_mm_total: u64,
    pub movement_preview_physics_max_applied_correction_abs_mm: u32,
    pub movement_preview_physics_final_contact_count: usize,
    pub movement_preview_applied_delta_count: usize,
    pub movement_preview_corrected_delta_count: usize,
    pub movement_preview_blocked_delta_count: usize,
    pub movement_apply_sample_count: usize,
    pub movement_apply_flow_field_build_count: usize,
    pub movement_apply_flow_field_cache_request_count: u64,
    pub movement_apply_flow_field_cache_hit_count: u64,
    pub movement_apply_flow_field_cache_eviction_count: u64,
    pub movement_apply_flow_field_cache_entry_count: usize,
    pub movement_apply_flow_field_query_count: usize,
    pub movement_apply_flow_field_unreachable_count: usize,
    pub movement_apply_applied_delta_count: usize,
    pub movement_apply_corrected_delta_count: usize,
    pub movement_apply_blocked_delta_count: usize,
    pub movement_apply_movement_probe_correction_limit_abs_mm: Option<u32>,
    pub movement_apply_movement_probe_clamped_correction_count: usize,
    pub movement_apply_physics_candidate_count: usize,
    pub movement_apply_physics_initial_contact_count: usize,
    pub movement_apply_physics_iterations_run: usize,
    pub movement_apply_physics_applied_correction_count: usize,
    pub movement_apply_physics_applied_correction_abs_mm_total: u64,
    pub movement_apply_physics_max_applied_correction_abs_mm: u32,
    pub movement_apply_physics_correction_limit_abs_mm: Option<u32>,
    pub movement_apply_physics_clamped_correction_count: usize,
    pub movement_apply_physics_final_contact_count: usize,
    pub movement_apply_physics_synced_position_count: usize,
    pub movement_apply_physics_sample_synced_position_count: usize,
    pub clamped_movement_apply_sample_count: usize,
    pub clamped_movement_apply_movement_probe_correction_limit_abs_mm: Option<u32>,
    pub clamped_movement_apply_movement_probe_clamped_correction_count: usize,
    pub clamped_movement_apply_physics_correction_limit_abs_mm: Option<u32>,
    pub clamped_movement_apply_physics_clamped_correction_count: usize,
    pub clamped_movement_apply_physics_max_applied_correction_abs_mm: u32,
    pub batch_movement_apply_sample_count: usize,
    pub batch_movement_apply_flow_field_cache_request_count: u64,
    pub batch_movement_apply_flow_field_cache_hit_count: u64,
    pub batch_movement_apply_flow_field_cache_eviction_count: u64,
    pub batch_movement_apply_flow_field_cache_entry_count: usize,
    pub batch_movement_apply_applied_delta_count: usize,
    pub batch_movement_apply_corrected_delta_count: usize,
    pub batch_movement_apply_blocked_delta_count: usize,
    pub batch_movement_apply_movement_probe_correction_limit_abs_mm: Option<u32>,
    pub batch_movement_apply_movement_probe_clamped_correction_count: usize,
    pub batch_movement_apply_physics_candidate_count: usize,
    pub batch_movement_apply_physics_iterations_run: usize,
    pub batch_movement_apply_physics_synced_position_count: usize,
    pub batch_movement_apply_physics_sample_synced_position_count: usize,
    pub batch_movement_tick_sample_count: usize,
    pub batch_movement_tick_active_count: usize,
    pub batch_movement_tick_spawned_count: usize,
    pub batch_movement_tick_applied_delta_count: usize,
    pub batch_movement_tick_physics_iterations_run: usize,
    pub batch_movement_tick_snapshot_entity_count: usize,
    pub batch_movement_tick_snapshot_bytes: u64,
    pub batch_movement_delta_snapshot_entity_count: usize,
    pub batch_movement_delta_snapshot_bytes: u64,
    pub movement_tick_sample_count: usize,
    pub movement_tick_active_count: usize,
    pub movement_tick_spawned_count: usize,
    pub movement_tick_applied_delta_count: usize,
    pub movement_tick_physics_iterations_run: usize,
    pub configured_movement_tick_sample_count: usize,
    pub configured_movement_tick_active_count: usize,
    pub configured_movement_tick_spawned_count: usize,
    pub configured_movement_tick_applied_delta_count: usize,
    pub configured_movement_tick_physics_iterations_run: usize,
    pub configured_batch_movement_tick_sample_count: usize,
    pub configured_batch_movement_tick_active_count: usize,
    pub configured_batch_movement_tick_spawned_count: usize,
    pub configured_batch_movement_tick_applied_delta_count: usize,
    pub configured_batch_movement_tick_physics_iterations_run: usize,
    pub configured_batch_movement_tick_claim_scope: &'static str,
    pub configured_clamped_movement_tick_sample_count: usize,
    pub configured_clamped_movement_tick_movement_probe_correction_limit_abs_mm: Option<u32>,
    pub configured_clamped_movement_tick_movement_probe_clamped_correction_count: usize,
    pub configured_clamped_movement_tick_physics_correction_limit_abs_mm: Option<u32>,
    pub configured_clamped_movement_tick_physics_clamped_correction_count: usize,
    pub configured_clamped_movement_tick_physics_max_applied_correction_abs_mm: u32,
    pub configured_movement_loop_tick_count: u64,
    pub configured_movement_loop_active_count: usize,
    pub configured_movement_loop_spawned_count: usize,
    pub configured_movement_loop_sample_count: usize,
    pub configured_movement_loop_applied_delta_count: usize,
    pub configured_movement_loop_physics_iterations_run: usize,
    pub configured_movement_loop_flow_field_cache_request_count: u64,
    pub configured_movement_loop_flow_field_cache_hit_count: u64,
    pub configured_movement_loop_flow_field_cache_eviction_count: u64,
    pub configured_movement_loop_flow_field_cache_entry_count: usize,
    pub configured_movement_loop_moved_entity_count: usize,
    pub static_obstacle_count: usize,
    pub static_obstacle_source: &'static str,
    pub static_obstacle_map_obstacle_count: usize,
    pub static_obstacle_clearance_mm: i32,
    pub static_obstacle_blocker_cell_count: usize,
    pub static_obstacle_movement_sample_count: usize,
    pub static_obstacle_movement_flow_field_build_count: usize,
    pub static_obstacle_movement_applied_delta_count: usize,
    pub static_obstacle_movement_blocked_delta_count: usize,
    pub static_obstacle_movement_physics_iterations_run: usize,
    pub movement_tick_snapshot_entity_count: usize,
    pub movement_tick_snapshot_bytes: u64,
    pub movement_tick_snapshot_bandwidth_kb_s_per_client: f64,
    pub movement_delta_snapshot_entity_count: usize,
    pub movement_delta_snapshot_removed_count: usize,
    pub movement_delta_snapshot_aggregate_far_state_count: usize,
    pub movement_delta_snapshot_bytes: u64,
    pub movement_delta_snapshot_bandwidth_kb_s_per_client: f64,
    pub movement_aggregate_delta_snapshot_entity_count: usize,
    pub movement_aggregate_delta_snapshot_removed_count: usize,
    pub movement_aggregate_delta_snapshot_aggregate_far_state_count: usize,
    pub movement_aggregate_delta_snapshot_bytes: u64,
    pub movement_aggregate_delta_snapshot_bandwidth_kb_s_per_client: f64,
    pub collision_resolution_contact_count: usize,
    pub collision_resolution_correction_count: usize,
    pub collision_physics_iterations_run: usize,
    pub collision_physics_applied_correction_count: usize,
    pub collision_physics_applied_correction_abs_mm_total: u64,
    pub collision_physics_max_applied_correction_abs_mm: u32,
    pub collision_physics_final_contact_count: usize,
    pub estimated_snapshot_bytes: u64,
    pub estimated_bandwidth_kb_s_per_client: f64,
    pub budget_result: BudgetResult,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmConfiguredMovementLoopMeasurementRun {
    pub sample_count: usize,
    pub tick_count_per_sample: u64,
    pub movement_sample_limit: usize,
    pub active_count: usize,
    pub spawned_count_total: usize,
    pub elapsed_us_min: u64,
    pub elapsed_us_p50: u64,
    pub elapsed_us_p95: u64,
    pub elapsed_us_p99: u64,
    pub elapsed_us_max: u64,
    pub movement_sample_count_total: usize,
    pub applied_delta_count_total: usize,
    pub physics_iterations_run_total: usize,
    pub flow_field_cache_request_count_total: u64,
    pub flow_field_cache_hit_count_total: u64,
    pub flow_field_cache_eviction_count_total: u64,
    pub flow_field_cache_entry_count_max: usize,
    pub moved_entity_count_min: usize,
    pub budget_result: BudgetResult,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmBatchVsSingleMovementLoopMeasurementRun {
    pub sample_count: usize,
    pub tick_count_per_sample: u64,
    pub movement_sample_limit: usize,
    pub active_count: usize,
    pub single_elapsed_us_p50: u64,
    pub single_elapsed_us_p95: u64,
    pub single_elapsed_us_p99: u64,
    pub batch_elapsed_us_p50: u64,
    pub batch_elapsed_us_p95: u64,
    pub batch_elapsed_us_p99: u64,
    pub batch_to_single_elapsed_p95_bps: u32,
    pub single_movement_sample_count_total: usize,
    pub batch_movement_sample_count_total: usize,
    pub single_applied_delta_count_total: usize,
    pub batch_applied_delta_count_total: usize,
    pub single_physics_iterations_run_total: usize,
    pub batch_physics_iterations_run_total: usize,
    pub single_flow_field_cache_hit_count_total: u64,
    pub batch_flow_field_cache_hit_count_total: u64,
    pub single_flow_field_cache_eviction_count_total: u64,
    pub batch_flow_field_cache_eviction_count_total: u64,
    pub single_moved_entity_count_min: usize,
    pub batch_moved_entity_count_min: usize,
    pub budget_result: BudgetResult,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwarmBatchMovementReplicationSmokeRun {
    pub tick: Tick,
    pub active_count: usize,
    pub baseline_snapshot_entity_count: usize,
    pub movement_sample_count: usize,
    pub movement_applied_delta_count: usize,
    pub movement_physics_iterations_run: usize,
    pub movement_snapshot_entity_count: usize,
    pub delta_visible_entity_count: usize,
    pub delta_changed_visible_entity_count: usize,
    pub delta_removed_entity_count: usize,
    pub delta_aggregate_far_state_count: usize,
    pub delta_snapshot_bytes: u64,
    pub delta_bandwidth_kb_s_per_client: f64,
    pub aggregate_visible_entity_count: usize,
    pub aggregate_changed_visible_entity_count: usize,
    pub aggregate_far_state_count: usize,
    pub aggregate_snapshot_bytes: u64,
    pub budget_result: BudgetResult,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmState {
    config: SwarmConfig,
    spawn_points: Vec<SwarmSpawnPoint>,
    active_entities: BTreeMap<EntityId, SwarmEntity>,
    static_obstacles: BTreeMap<EntityId, CollisionBody>,
    static_obstacle_shapes: Vec<MapShape>,
    flow_field_cache: SwarmFlowFieldCache,
    aggro_trails: BTreeMap<EntityId, AggroStimulus>,
    next_entity_id: EntityId,
    next_spawn_point_index: usize,
    spawned_total: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SwarmFlowFieldCacheStats {
    request_count: u64,
    build_count: u64,
    hit_count: u64,
    eviction_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SwarmFlowFieldCacheKey {
    bounds: FlowFieldBounds,
    target_cell: SpatialCell,
    blocked_cells: BTreeSet<SpatialCell>,
    cell_size_mm: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SwarmFlowFieldCache {
    fields: BTreeMap<SwarmFlowFieldCacheKey, FlowFieldMap>,
    stats: SwarmFlowFieldCacheStats,
}

impl SwarmFlowFieldCache {
    fn clear(&mut self) {
        self.fields.clear();
    }

    fn stats(&self) -> SwarmFlowFieldCacheStats {
        self.stats
    }

    fn entry_count(&self) -> usize {
        self.fields.len()
    }

    fn get_or_build(
        &mut self,
        bounds: FlowFieldBounds,
        target_cell: SpatialCell,
        blocked_cells: BTreeSet<SpatialCell>,
        cell_size_mm: i32,
    ) -> &FlowFieldMap {
        self.stats.request_count += 1;
        let key = SwarmFlowFieldCacheKey {
            bounds,
            target_cell,
            blocked_cells,
            cell_size_mm,
        };
        if !self.fields.contains_key(&key)
            && self.fields.len() >= SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT
        {
            if let Some(evicted_key) = self.fields.keys().next().cloned() {
                self.fields.remove(&evicted_key);
                self.stats.eviction_count += 1;
            }
        }
        match self.fields.entry(key) {
            std::collections::btree_map::Entry::Occupied(entry) => {
                self.stats.hit_count += 1;
                entry.into_mut()
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                let key = entry.key();
                let flow_field = FlowFieldMap::build(
                    key.bounds,
                    key.target_cell,
                    key.blocked_cells.iter().copied(),
                )
                .expect("swarm movement flow bounds include target cell");
                self.stats.build_count += 1;
                entry.insert(flow_field)
            }
        }
    }
}

impl SwarmState {
    pub fn new(
        config: SwarmConfig,
        spawn_points: Vec<SwarmSpawnPoint>,
        first_entity_id: EntityId,
    ) -> Result<Self, SwarmError> {
        if spawn_points.is_empty() {
            return Err(SwarmError::EmptySpawnPoints);
        }
        if config.spawn_interval_ticks == 0 {
            return Err(SwarmError::InvalidSpawnInterval);
        }
        if config.spawn_batch_size == 0 {
            return Err(SwarmError::InvalidSpawnBatchSize);
        }
        if config.max_active == 0 {
            return Err(SwarmError::InvalidMaxActive);
        }
        if config.route_pressure_radius_mm <= 0 {
            return Err(SwarmError::InvalidRoutePressureRadius);
        }
        if config.collision_radius_mm <= 0 {
            return Err(SwarmError::InvalidCollisionRadius);
        }
        if config.direct_aggro_ticks > config.aggro_memory_ticks {
            return Err(SwarmError::InvalidAggroWindow);
        }
        if config.full_lod_distance_mm <= 0
            || config.reduced_lod_distance_mm <= config.full_lod_distance_mm
        {
            return Err(SwarmError::InvalidLodDistance);
        }
        if config.movement_mode != SwarmMovementMode::SpawnOnly {
            if config.movement_sample_limit == 0 {
                return Err(SwarmError::InvalidMovementSampleLimit);
            }
            if config.movement_cell_size_mm <= 0 {
                return Err(SwarmError::InvalidMovementCellSize);
            }
            if config.movement_physics_iterations == 0 {
                return Err(SwarmError::InvalidMovementPhysicsIterations);
            }
        }

        Ok(Self {
            config,
            spawn_points,
            active_entities: BTreeMap::new(),
            static_obstacles: BTreeMap::new(),
            static_obstacle_shapes: Vec::new(),
            flow_field_cache: SwarmFlowFieldCache::default(),
            aggro_trails: BTreeMap::new(),
            next_entity_id: first_entity_id,
            next_spawn_point_index: 0,
            spawned_total: 0,
        })
    }

    pub fn config(&self) -> SwarmConfig {
        self.config
    }

    pub fn active_count(&self) -> usize {
        self.active_entities.len()
    }

    pub fn spawned_total(&self) -> usize {
        self.spawned_total
    }

    pub fn active_entities(&self) -> impl Iterator<Item = &SwarmEntity> {
        self.active_entities.values()
    }

    pub fn set_static_obstacles(
        &mut self,
        obstacles: impl IntoIterator<Item = CollisionBody>,
    ) -> Result<(), SwarmError> {
        let mut static_obstacles = BTreeMap::new();
        for obstacle in obstacles {
            if obstacle.kind != CollisionBodyKind::StaticObstacle || obstacle.radius_mm <= 0 {
                return Err(SwarmError::InvalidStaticObstacle);
            }
            static_obstacles.insert(obstacle.entity_id, obstacle);
        }

        self.static_obstacles = static_obstacles;
        self.static_obstacle_shapes.clear();
        self.flow_field_cache.clear();
        Ok(())
    }

    pub fn set_static_obstacles_from_map_data(
        &mut self,
        map: &MapDataImport,
    ) -> Result<usize, SwarmError> {
        validate_map_data_import(map).map_err(SwarmError::InvalidMapData)?;
        let obstacles = map
            .obstacles
            .iter()
            .enumerate()
            .map(|(index, shape)| swarm_static_obstacle_from_map_shape(index, shape));
        self.set_static_obstacles(obstacles)?;
        self.static_obstacle_shapes = map.obstacles.clone();
        Ok(self.static_obstacle_count())
    }

    pub fn static_obstacle_count(&self) -> usize {
        self.static_obstacles.len()
    }

    pub fn record_aggro_stimulus(&mut self, stimulus: AggroStimulus) {
        self.aggro_trails
            .insert(stimulus.source_entity_id, stimulus);
    }

    pub fn collision_bodies(&self) -> Vec<CollisionBody> {
        self.active_entities
            .values()
            .copied()
            .map(SwarmEntity::collision_body)
            .collect()
    }

    fn movement_collision_bodies(&self) -> Vec<CollisionBody> {
        let mut bodies = self.collision_bodies();
        bodies.extend(self.static_obstacles.values().copied());
        bodies
    }

    fn static_obstacle_blocker_cells(
        &self,
        cell_size_mm: i32,
        bounds: FlowFieldBounds,
        clearance_mm: i32,
    ) -> BTreeSet<SpatialCell> {
        if !self.static_obstacle_shapes.is_empty() {
            return self
                .static_obstacle_shapes
                .iter()
                .flat_map(|shape| {
                    swarm_blocker_cells_for_map_shape(shape, cell_size_mm, bounds, clearance_mm)
                })
                .collect();
        }

        self.static_obstacles
            .values()
            .map(|obstacle| swarm_preview_cell_for_position(obstacle.position, cell_size_mm))
            .filter(|cell| bounds.contains(*cell))
            .collect()
    }

    pub fn snapshot_entities(&self) -> Vec<EntityState> {
        self.active_entities
            .values()
            .map(|entity| EntityState {
                entity_id: entity.entity_id,
                entity_kind: SWARM_ENTITY_KIND,
                faction_id: 0,
                flags: 1,
                position: entity.position,
                facing_millirad: 0,
                health_q8: 256,
                state_id: 1,
                state_param_q8: 0,
            })
            .collect()
    }

    pub fn build_full_snapshot(&self, snapshot_id: u64, tick: Tick) -> Snapshot {
        let mut builder = SnapshotBuilder::full(snapshot_id, tick);
        for entity in self.snapshot_entities() {
            builder.push_entity(entity);
        }
        builder.build()
    }

    pub fn build_interest_delta_snapshot(
        &self,
        session_id: PlayerSessionId,
        region: AoiRegion,
        snapshot_id: u64,
        baseline_snapshot_id: u64,
        tick: Tick,
        cell_size_mm: i32,
    ) -> SwarmMovementDeltaSnapshotReport {
        let entities = self.snapshot_entities();
        let grid = swarm_grid_for_snapshot_entities(&entities, cell_size_mm);
        let mut interest = InterestManager::new();
        interest.upsert_subscription(session_id, region);
        let update = interest
            .refresh_subscription(session_id, &grid)
            .expect("swarm movement delta snapshot subscription exists");
        let visible_entity_ids = update.visible_entities.clone();
        let InterestSnapshotDelta {
            snapshot,
            aggregate_far_state,
        } = build_interest_snapshot_delta(
            &update,
            entities.clone(),
            &grid,
            snapshot_id,
            baseline_snapshot_id,
            tick,
        );

        SwarmMovementDeltaSnapshotReport {
            schema: SWARM_SCHEMA,
            tick,
            visible_entity_count: snapshot.entities.len(),
            removed_entity_count: snapshot.removed_entities.len(),
            aggregate_far_state_count: aggregate_far_state.len(),
            snapshot,
            visible_entity_ids,
            claim_scope: "swarm_movement_delta_snapshot_local",
        }
    }

    pub fn evaluate_behavior(&self, tick: Tick, focus: WorldPosition) -> SwarmBehaviorReport {
        let active_aggro = self.best_aggro_sample(tick);
        let mut lod_counts = SwarmAiLodCounts {
            full: 0,
            reduced: 0,
            aggregate: 0,
        };
        let samples = self
            .active_entities
            .values()
            .map(|entity| {
                let ai_lod = self.ai_lod(entity.position, focus);
                match ai_lod {
                    SwarmAiLod::Full => lod_counts.full += 1,
                    SwarmAiLod::Reduced => lod_counts.reduced += 1,
                    SwarmAiLod::Aggregate => lod_counts.aggregate += 1,
                }

                let (intent_kind, intent_target) = active_aggro
                    .map(|sample| {
                        let intent_kind = match sample.lane {
                            AggroTrailLane::Direct => SwarmIntentKind::AggroDirect,
                            AggroTrailLane::Memory => SwarmIntentKind::AggroMemory,
                        };
                        (intent_kind, sample.position)
                    })
                    .unwrap_or((SwarmIntentKind::RoutePressure, entity.route_target));

                SwarmBehaviorSample {
                    entity_id: entity.entity_id,
                    intent_kind,
                    intent_target,
                    ai_lod,
                }
            })
            .collect();

        SwarmBehaviorReport {
            schema: SWARM_SCHEMA,
            tick,
            focus,
            samples,
            lod_counts,
        }
    }

    pub fn movement_preview(
        &self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
    ) -> SwarmMovementPreviewReport {
        let cell_size_mm = cell_size_mm.max(1);
        let behavior = self.evaluate_behavior(tick, focus);
        let collision_world = CollisionWorld::with_bodies(2_000, self.movement_collision_bodies())
            .expect("swarm collision bodies are valid");
        let mut preview_physics_world = collision_world.clone();
        let preview_behaviors = behavior
            .samples
            .iter()
            .take(sample_limit)
            .copied()
            .collect::<Vec<_>>();
        let flow_bounds =
            swarm_preview_flow_bounds(&preview_behaviors, &self.active_entities, cell_size_mm);
        let blocker_cells = self.static_obstacle_blocker_cells(
            cell_size_mm,
            flow_bounds,
            self.config.collision_radius_mm,
        );
        let mut flow_fields = BTreeMap::<SpatialCell, FlowFieldMap>::new();
        let mut flow_field_query_count = 0usize;
        let mut flow_field_unreachable_count = 0usize;
        let mut physics_candidate_count = 0usize;
        let mut applied_delta_count = 0usize;
        let mut corrected_delta_count = 0usize;
        let mut blocked_delta_count = 0usize;
        let samples = preview_behaviors
            .iter()
            .filter_map(|sample| {
                let entity = self.active_entities.get(&sample.entity_id)?;
                let entity_cell = swarm_preview_cell_for_position(entity.position, cell_size_mm);
                let target_cell =
                    swarm_preview_cell_for_position(sample.intent_target, cell_size_mm);
                let flow_field = flow_fields.entry(target_cell).or_insert_with(|| {
                    FlowFieldMap::build(
                        flow_bounds,
                        target_cell,
                        blocker_cells
                            .iter()
                            .copied()
                            .filter(|cell| *cell != target_cell),
                    )
                    .expect("swarm movement preview flow bounds include target cell")
                });
                let flow_step = flow_field.step_from(entity_cell, cell_size_mm);
                flow_field_query_count += 1;
                if matches!(
                    flow_step.result,
                    FlowFieldStepResult::Blocked | FlowFieldStepResult::Unreachable
                ) {
                    flow_field_unreachable_count += 1;
                }
                let requested_delta = flow_step.movement_delta;
                let requested_position = WorldPosition {
                    x_mm: entity.position.x_mm.saturating_add(requested_delta.dx_mm),
                    y_mm: entity.position.y_mm.saturating_add(requested_delta.dy_mm),
                };
                let probe = collision_world.probe_movement_after_resolution(
                    entity.entity_id,
                    requested_position,
                    1,
                );
                let resolved_delta = match probe.decision {
                    CollisionMovementDecision::Accepted | CollisionMovementDecision::Corrected => {
                        applied_delta_count += 1;
                        if probe.decision == CollisionMovementDecision::Corrected {
                            corrected_delta_count += 1;
                        }
                        let resolved_delta = probe
                            .resolved_delta
                            .expect("accepted or corrected movement probe has a resolved delta");
                        if let Some(resolved_position) = probe.resolved_position {
                            physics_candidate_count += 1;
                            let mut body = entity.collision_body();
                            body.position = resolved_position;
                            preview_physics_world
                                .insert_or_update(body)
                                .expect("swarm preview body keeps a valid radius");
                        }
                        resolved_delta
                    }
                    CollisionMovementDecision::Blocked | CollisionMovementDecision::UnknownBody => {
                        blocked_delta_count += 1;
                        MovementDelta { dx_mm: 0, dy_mm: 0 }
                    }
                };

                Some(SwarmMovementPreviewSample {
                    entity_id: entity.entity_id,
                    intent_kind: sample.intent_kind,
                    from_position: entity.position,
                    intent_target: sample.intent_target,
                    flow_field_result: flow_step.result,
                    requested_delta,
                    resolved_delta,
                    collision_decision: probe.decision,
                    collision_iterations_run: probe.iterations_run,
                    collision_applied_correction_count: probe.applied_correction_count,
                })
            })
            .collect::<Vec<_>>();
        let preview_physics_step = preview_physics_world.step_overlap_resolution(1);

        SwarmMovementPreviewReport {
            schema: SWARM_SCHEMA,
            tick,
            sample_count: samples.len(),
            flow_field_build_count: flow_fields.len(),
            flow_field_query_count,
            flow_field_unreachable_count,
            physics_candidate_count,
            physics_initial_contact_count: preview_physics_step.initial_contact_count,
            physics_iterations_run: preview_physics_step.iterations_run,
            physics_applied_correction_count: preview_physics_step.applied_correction_count,
            physics_applied_correction_abs_mm_total: preview_physics_step
                .applied_correction_abs_mm_total,
            physics_max_applied_correction_abs_mm: preview_physics_step
                .max_applied_correction_abs_mm,
            physics_final_contact_count: preview_physics_step.final_contact_count,
            applied_delta_count,
            corrected_delta_count,
            blocked_delta_count,
            samples,
            claim_scope: "movement_preview_only",
        }
    }

    pub fn apply_flow_field_movement_step(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
    ) -> SwarmMovementApplyReport {
        self.apply_flow_field_movement_step_with_optional_correction_limit(
            tick,
            focus,
            sample_limit,
            cell_size_mm,
            physics_iterations,
            None,
        )
    }

    pub fn apply_flow_field_movement_step_with_correction_limit(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> SwarmMovementApplyReport {
        self.apply_flow_field_movement_step_with_optional_correction_limit(
            tick,
            focus,
            sample_limit,
            cell_size_mm,
            physics_iterations,
            Some(correction_limit_abs_mm),
        )
    }

    pub fn apply_flow_field_batch_movement_step(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
    ) -> SwarmMovementApplyReport {
        self.apply_flow_field_batch_movement_step_with_optional_correction_limit(
            tick,
            focus,
            sample_limit,
            cell_size_mm,
            physics_iterations,
            None,
        )
    }

    pub fn apply_flow_field_batch_movement_step_with_correction_limit(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> SwarmMovementApplyReport {
        self.apply_flow_field_batch_movement_step_with_optional_correction_limit(
            tick,
            focus,
            sample_limit,
            cell_size_mm,
            physics_iterations,
            Some(correction_limit_abs_mm),
        )
    }

    fn apply_flow_field_batch_movement_step_with_optional_correction_limit(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> SwarmMovementApplyReport {
        #[derive(Debug, Clone, Copy)]
        struct PreparedBatchMovement {
            entity_id: EntityId,
            intent_kind: SwarmIntentKind,
            from_position: WorldPosition,
            intent_target: WorldPosition,
            flow_field_result: FlowFieldStepResult,
            requested_delta: MovementDelta,
            requested_position: WorldPosition,
        }

        let cell_size_mm = cell_size_mm.max(1);
        let behavior = self.evaluate_behavior(tick, focus);
        let movement_behaviors = behavior
            .samples
            .iter()
            .take(sample_limit)
            .copied()
            .collect::<Vec<_>>();
        let flow_bounds =
            swarm_preview_flow_bounds(&movement_behaviors, &self.active_entities, cell_size_mm);
        let blocker_cells = self.static_obstacle_blocker_cells(
            cell_size_mm,
            flow_bounds,
            self.config.collision_radius_mm,
        );
        let mut collision_world =
            CollisionWorld::with_bodies(2_000, self.movement_collision_bodies())
                .expect("swarm collision bodies are valid");
        let cache_before = self.flow_field_cache.stats();
        let mut flow_field_query_count = 0usize;
        let mut flow_field_unreachable_count = 0usize;
        let mut prepared = Vec::with_capacity(movement_behaviors.len());

        for sample in movement_behaviors {
            let Some(entity) = self.active_entities.get(&sample.entity_id).copied() else {
                continue;
            };
            let entity_cell = swarm_preview_cell_for_position(entity.position, cell_size_mm);
            let target_cell = swarm_preview_cell_for_position(sample.intent_target, cell_size_mm);
            let field_blocker_cells = blocker_cells
                .iter()
                .copied()
                .filter(|cell| *cell != target_cell)
                .collect::<BTreeSet<_>>();
            let flow_step = {
                let flow_field = self.flow_field_cache.get_or_build(
                    flow_bounds,
                    target_cell,
                    field_blocker_cells,
                    cell_size_mm,
                );
                flow_field.step_from(entity_cell, cell_size_mm)
            };
            flow_field_query_count += 1;
            if matches!(
                flow_step.result,
                FlowFieldStepResult::Blocked | FlowFieldStepResult::Unreachable
            ) {
                flow_field_unreachable_count += 1;
            }

            let requested_delta = flow_step.movement_delta;
            let requested_position = WorldPosition {
                x_mm: entity.position.x_mm.saturating_add(requested_delta.dx_mm),
                y_mm: entity.position.y_mm.saturating_add(requested_delta.dy_mm),
            };
            prepared.push(PreparedBatchMovement {
                entity_id: entity.entity_id,
                intent_kind: sample.intent_kind,
                from_position: entity.position,
                intent_target: sample.intent_target,
                flow_field_result: flow_step.result,
                requested_delta,
                requested_position,
            });
        }

        let candidates = prepared
            .iter()
            .map(|sample| CollisionBatchMovementCandidate {
                entity_id: sample.entity_id,
                target_position: sample.requested_position,
            });
        let batch_probe = match correction_limit_abs_mm {
            Some(limit_abs_mm) => collision_world
                .probe_batch_movements_after_resolution_with_correction_limit(
                    candidates,
                    1,
                    limit_abs_mm,
                ),
            None => collision_world.probe_batch_movements_after_resolution(candidates, 1),
        };

        let mut applied_delta_count = 0usize;
        let mut corrected_delta_count = 0usize;
        let mut blocked_delta_count = 0usize;
        let mut physics_candidate_count = 0usize;
        let mut samples = Vec::with_capacity(prepared.len());

        for (prepared_sample, probe_sample) in prepared.iter().zip(batch_probe.samples.iter()) {
            let (applied_delta, final_position) = match probe_sample.decision {
                CollisionMovementDecision::Accepted | CollisionMovementDecision::Corrected => {
                    applied_delta_count += 1;
                    if probe_sample.decision == CollisionMovementDecision::Corrected {
                        corrected_delta_count += 1;
                    }
                    let resolved_delta = probe_sample
                        .resolved_delta
                        .expect("accepted or corrected batch movement probe has a resolved delta");
                    let resolved_position = probe_sample.resolved_position.expect(
                        "accepted or corrected batch movement probe has a resolved position",
                    );
                    physics_candidate_count += 1;
                    if let Some(active) = self.active_entities.get_mut(&prepared_sample.entity_id) {
                        active.position = resolved_position;
                        collision_world
                            .insert_or_update(active.collision_body())
                            .expect("swarm movement body keeps a valid radius");
                    }
                    (resolved_delta, resolved_position)
                }
                CollisionMovementDecision::Blocked | CollisionMovementDecision::UnknownBody => {
                    blocked_delta_count += 1;
                    (
                        MovementDelta { dx_mm: 0, dy_mm: 0 },
                        prepared_sample.from_position,
                    )
                }
            };

            samples.push(SwarmMovementApplySample {
                entity_id: prepared_sample.entity_id,
                intent_kind: prepared_sample.intent_kind,
                from_position: prepared_sample.from_position,
                intent_target: prepared_sample.intent_target,
                flow_field_result: prepared_sample.flow_field_result,
                requested_delta: prepared_sample.requested_delta,
                applied_delta,
                collision_decision: probe_sample.decision,
                final_position,
            });
        }

        let physics_step = if physics_candidate_count > 0 {
            match correction_limit_abs_mm {
                Some(limit_abs_mm) => collision_world
                    .step_overlap_resolution_with_correction_limit(
                        physics_iterations,
                        limit_abs_mm,
                    ),
                None => collision_world.step_overlap_resolution(physics_iterations),
            }
        } else {
            crate::CollisionPhysicsStep {
                initial_contact_count: collision_world.detect_overlaps().len(),
                iterations_requested: physics_iterations,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm,
                clamped_correction_count: 0,
                final_contact_count: collision_world.detect_overlaps().len(),
                resolved: true,
                claim_scope: "physics_step_only",
            }
        };
        let physics_synced_position_count = if physics_candidate_count > 0 {
            self.sync_positions_from_collision_world(&collision_world)
        } else {
            0
        };
        let physics_sample_synced_position_count =
            sync_apply_samples_from_active_entities(&mut samples, &self.active_entities);
        let cache_after = self.flow_field_cache.stats();

        SwarmMovementApplyReport {
            schema: SWARM_SCHEMA,
            tick,
            sample_count: samples.len(),
            flow_field_build_count: (cache_after.build_count - cache_before.build_count) as usize,
            flow_field_cache_request_count: cache_after.request_count - cache_before.request_count,
            flow_field_cache_hit_count: cache_after.hit_count - cache_before.hit_count,
            flow_field_cache_eviction_count: cache_after.eviction_count
                - cache_before.eviction_count,
            flow_field_cache_entry_count: self.flow_field_cache.entry_count(),
            flow_field_query_count,
            flow_field_unreachable_count,
            applied_delta_count,
            corrected_delta_count,
            blocked_delta_count,
            movement_probe_correction_limit_abs_mm: batch_probe.correction_limit_abs_mm,
            movement_probe_clamped_correction_count: batch_probe.clamped_correction_count,
            physics_candidate_count,
            physics_initial_contact_count: physics_step.initial_contact_count,
            physics_iterations_run: physics_step.iterations_run,
            physics_applied_correction_count: physics_step.applied_correction_count,
            physics_applied_correction_abs_mm_total: physics_step.applied_correction_abs_mm_total,
            physics_max_applied_correction_abs_mm: physics_step.max_applied_correction_abs_mm,
            physics_correction_limit_abs_mm: physics_step.correction_limit_abs_mm,
            physics_clamped_correction_count: physics_step.clamped_correction_count,
            physics_final_contact_count: physics_step.final_contact_count,
            physics_synced_position_count,
            physics_sample_synced_position_count,
            samples,
            claim_scope: "swarm_batch_movement_apply_opt_in",
        }
    }

    fn apply_flow_field_movement_step_with_optional_correction_limit(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> SwarmMovementApplyReport {
        let cell_size_mm = cell_size_mm.max(1);
        let behavior = self.evaluate_behavior(tick, focus);
        let movement_behaviors = behavior
            .samples
            .iter()
            .take(sample_limit)
            .copied()
            .collect::<Vec<_>>();
        let flow_bounds =
            swarm_preview_flow_bounds(&movement_behaviors, &self.active_entities, cell_size_mm);
        let blocker_cells = self.static_obstacle_blocker_cells(
            cell_size_mm,
            flow_bounds,
            self.config.collision_radius_mm,
        );
        let mut collision_world =
            CollisionWorld::with_bodies(2_000, self.movement_collision_bodies())
                .expect("swarm collision bodies are valid");
        let cache_before = self.flow_field_cache.stats();
        let mut flow_field_query_count = 0usize;
        let mut flow_field_unreachable_count = 0usize;
        let mut applied_delta_count = 0usize;
        let mut corrected_delta_count = 0usize;
        let mut blocked_delta_count = 0usize;
        let mut movement_probe_clamped_correction_count = 0usize;
        let mut physics_candidate_count = 0usize;
        let mut samples = Vec::with_capacity(movement_behaviors.len());

        for sample in movement_behaviors {
            let Some(entity) = self.active_entities.get(&sample.entity_id).copied() else {
                continue;
            };
            let entity_cell = swarm_preview_cell_for_position(entity.position, cell_size_mm);
            let target_cell = swarm_preview_cell_for_position(sample.intent_target, cell_size_mm);
            let field_blocker_cells = blocker_cells
                .iter()
                .copied()
                .filter(|cell| *cell != target_cell)
                .collect::<BTreeSet<_>>();
            let flow_step = {
                let flow_field = self.flow_field_cache.get_or_build(
                    flow_bounds,
                    target_cell,
                    field_blocker_cells,
                    cell_size_mm,
                );
                flow_field.step_from(entity_cell, cell_size_mm)
            };
            flow_field_query_count += 1;
            if matches!(
                flow_step.result,
                FlowFieldStepResult::Blocked | FlowFieldStepResult::Unreachable
            ) {
                flow_field_unreachable_count += 1;
            }

            let requested_delta = flow_step.movement_delta;
            let requested_position = WorldPosition {
                x_mm: entity.position.x_mm.saturating_add(requested_delta.dx_mm),
                y_mm: entity.position.y_mm.saturating_add(requested_delta.dy_mm),
            };
            let probe = match correction_limit_abs_mm {
                Some(limit_abs_mm) => collision_world
                    .probe_movement_after_resolution_with_correction_limit(
                        entity.entity_id,
                        requested_position,
                        1,
                        limit_abs_mm,
                    ),
                None => collision_world.probe_movement_after_resolution(
                    entity.entity_id,
                    requested_position,
                    1,
                ),
            };
            movement_probe_clamped_correction_count += probe.clamped_correction_count;
            let (applied_delta, final_position) = match probe.decision {
                CollisionMovementDecision::Accepted | CollisionMovementDecision::Corrected => {
                    applied_delta_count += 1;
                    if probe.decision == CollisionMovementDecision::Corrected {
                        corrected_delta_count += 1;
                    }
                    let resolved_delta = probe
                        .resolved_delta
                        .expect("accepted or corrected movement probe has a resolved delta");
                    let resolved_position = probe
                        .resolved_position
                        .expect("accepted or corrected movement probe has a resolved position");
                    physics_candidate_count += 1;
                    if let Some(active) = self.active_entities.get_mut(&entity.entity_id) {
                        active.position = resolved_position;
                        collision_world
                            .insert_or_update(active.collision_body())
                            .expect("swarm movement body keeps a valid radius");
                    }
                    (resolved_delta, resolved_position)
                }
                CollisionMovementDecision::Blocked | CollisionMovementDecision::UnknownBody => {
                    blocked_delta_count += 1;
                    (MovementDelta { dx_mm: 0, dy_mm: 0 }, entity.position)
                }
            };

            samples.push(SwarmMovementApplySample {
                entity_id: entity.entity_id,
                intent_kind: sample.intent_kind,
                from_position: entity.position,
                intent_target: sample.intent_target,
                flow_field_result: flow_step.result,
                requested_delta,
                applied_delta,
                collision_decision: probe.decision,
                final_position,
            });
        }

        let physics_step = if physics_candidate_count > 0 {
            match correction_limit_abs_mm {
                Some(limit_abs_mm) => collision_world
                    .step_overlap_resolution_with_correction_limit(
                        physics_iterations,
                        limit_abs_mm,
                    ),
                None => collision_world.step_overlap_resolution(physics_iterations),
            }
        } else {
            crate::CollisionPhysicsStep {
                initial_contact_count: collision_world.detect_overlaps().len(),
                iterations_requested: physics_iterations,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm,
                clamped_correction_count: 0,
                final_contact_count: collision_world.detect_overlaps().len(),
                resolved: true,
                claim_scope: "physics_step_only",
            }
        };
        let physics_synced_position_count = if physics_candidate_count > 0 {
            self.sync_positions_from_collision_world(&collision_world)
        } else {
            0
        };
        let physics_sample_synced_position_count =
            sync_apply_samples_from_active_entities(&mut samples, &self.active_entities);
        let cache_after = self.flow_field_cache.stats();

        SwarmMovementApplyReport {
            schema: SWARM_SCHEMA,
            tick,
            sample_count: samples.len(),
            flow_field_build_count: (cache_after.build_count - cache_before.build_count) as usize,
            flow_field_cache_request_count: cache_after.request_count - cache_before.request_count,
            flow_field_cache_hit_count: cache_after.hit_count - cache_before.hit_count,
            flow_field_cache_eviction_count: cache_after.eviction_count
                - cache_before.eviction_count,
            flow_field_cache_entry_count: self.flow_field_cache.entry_count(),
            flow_field_query_count,
            flow_field_unreachable_count,
            applied_delta_count,
            corrected_delta_count,
            blocked_delta_count,
            movement_probe_correction_limit_abs_mm: correction_limit_abs_mm,
            movement_probe_clamped_correction_count,
            physics_candidate_count,
            physics_initial_contact_count: physics_step.initial_contact_count,
            physics_iterations_run: physics_step.iterations_run,
            physics_applied_correction_count: physics_step.applied_correction_count,
            physics_applied_correction_abs_mm_total: physics_step.applied_correction_abs_mm_total,
            physics_max_applied_correction_abs_mm: physics_step.max_applied_correction_abs_mm,
            physics_correction_limit_abs_mm: physics_step.correction_limit_abs_mm,
            physics_clamped_correction_count: physics_step.clamped_correction_count,
            physics_final_contact_count: physics_step.final_contact_count,
            physics_synced_position_count,
            physics_sample_synced_position_count,
            samples,
            claim_scope: "swarm_movement_apply_opt_in",
        }
    }

    pub fn tick(&mut self, tick: Tick) -> SwarmTickReport {
        self.tick_with_focus(tick, WorldPosition { x_mm: 0, y_mm: 0 })
    }

    pub fn tick_with_focus(&mut self, tick: Tick, focus: WorldPosition) -> SwarmTickReport {
        let spawned_entity_ids = self.spawn_due_on_tick(tick);
        let movement = match self.config.movement_mode {
            SwarmMovementMode::SpawnOnly => None,
            SwarmMovementMode::FlowFieldCollision => Some(
                self.apply_flow_field_movement_step_with_optional_correction_limit(
                    tick,
                    focus,
                    self.config.movement_sample_limit,
                    self.config.movement_cell_size_mm,
                    self.config.movement_physics_iterations,
                    self.config.movement_correction_limit_abs_mm,
                ),
            ),
            SwarmMovementMode::BatchFlowFieldCollision => Some(
                self.apply_flow_field_batch_movement_step_with_optional_correction_limit(
                    tick,
                    focus,
                    self.config.movement_sample_limit,
                    self.config.movement_cell_size_mm,
                    self.config.movement_physics_iterations,
                    self.config.movement_correction_limit_abs_mm,
                ),
            ),
        };

        self.tick_report(tick, spawned_entity_ids, movement)
    }

    fn spawn_due_on_tick(&mut self, tick: Tick) -> Vec<EntityId> {
        let mut spawned_entity_ids = Vec::new();

        if self.should_spawn_on_tick(tick) {
            let available_slots = self.config.max_active.saturating_sub(self.active_count());
            let spawn_count = self.config.spawn_batch_size.min(available_slots);

            for _ in 0..spawn_count {
                let entity = self.spawn_one(tick);
                spawned_entity_ids.push(entity.entity_id);
            }
        }

        spawned_entity_ids
    }

    fn tick_report(
        &self,
        tick: Tick,
        spawned_entity_ids: Vec<EntityId>,
        movement: Option<SwarmMovementApplyReport>,
    ) -> SwarmTickReport {
        SwarmTickReport {
            schema: SWARM_SCHEMA,
            tick,
            spawned_entity_ids,
            active_count: self.active_count(),
            spawn_capped: self.active_count() >= self.config.max_active,
            route_pressure: self.route_pressure(),
            aggro_trails: self.aggro_trail_samples(tick),
            movement,
        }
    }

    pub fn tick_with_flow_field_movement(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
    ) -> SwarmMovementTickReport {
        let spawned_entity_ids = self.spawn_due_on_tick(tick);
        let movement_report = self.apply_flow_field_movement_step(
            tick,
            focus,
            sample_limit,
            cell_size_mm,
            physics_iterations,
        );
        let tick_report = self.tick_report(tick, spawned_entity_ids, Some(movement_report.clone()));

        SwarmMovementTickReport {
            schema: SWARM_SCHEMA,
            tick,
            tick_report,
            movement_report,
            claim_scope: "swarm_movement_tick_opt_in",
        }
    }

    pub fn tick_with_batch_flow_field_movement(
        &mut self,
        tick: Tick,
        focus: WorldPosition,
        sample_limit: usize,
        cell_size_mm: i32,
        physics_iterations: usize,
    ) -> SwarmMovementTickReport {
        let spawned_entity_ids = self.spawn_due_on_tick(tick);
        let movement_report = self.apply_flow_field_batch_movement_step(
            tick,
            focus,
            sample_limit,
            cell_size_mm,
            physics_iterations,
        );
        let tick_report = self.tick_report(tick, spawned_entity_ids, Some(movement_report.clone()));

        SwarmMovementTickReport {
            schema: SWARM_SCHEMA,
            tick,
            tick_report,
            movement_report,
            claim_scope: "swarm_batch_movement_tick_opt_in",
        }
    }

    fn should_spawn_on_tick(&self, tick: Tick) -> bool {
        tick.0 >= self.config.start_tick.0
            && (tick.0 - self.config.start_tick.0) % self.config.spawn_interval_ticks == 0
            && self.active_count() < self.config.max_active
    }

    fn spawn_one(&mut self, tick: Tick) -> SwarmEntity {
        let spawn_point = self.spawn_points[self.next_spawn_point_index];
        self.next_spawn_point_index = (self.next_spawn_point_index + 1) % self.spawn_points.len();

        let entity = SwarmEntity {
            entity_id: self.next_entity_id,
            spawned_tick: tick,
            position: spawn_point.position,
            route_target: spawn_point.route_target,
            collision_radius_mm: self.config.collision_radius_mm,
        };

        self.next_entity_id.0 = self.next_entity_id.0.saturating_add(1);
        self.spawned_total += 1;
        self.active_entities.insert(entity.entity_id, entity);
        entity
    }

    fn route_pressure(&self) -> Vec<RoutePressureSample> {
        let mut buckets: BTreeMap<(i32, i32), RoutePressureAccumulator> = BTreeMap::new();

        for entity in self.active_entities.values() {
            let key = (entity.route_target.x_mm, entity.route_target.y_mm);
            buckets
                .entry(key)
                .or_insert_with(|| RoutePressureAccumulator::new(entity.route_target))
                .add(distance_sq_mm(entity.position, entity.route_target));
        }

        buckets
            .into_values()
            .map(|bucket| bucket.finish(self.config.route_pressure_radius_mm))
            .collect()
    }

    fn aggro_trail_samples(&self, tick: Tick) -> Vec<AggroTrailSample> {
        self.aggro_trails
            .values()
            .filter_map(|stimulus| self.aggro_sample_for(tick, *stimulus))
            .collect()
    }

    fn best_aggro_sample(&self, tick: Tick) -> Option<AggroTrailSample> {
        self.aggro_trail_samples(tick)
            .into_iter()
            .min_by_key(|sample| {
                let lane_rank = match sample.lane {
                    AggroTrailLane::Direct => 0u8,
                    AggroTrailLane::Memory => 1u8,
                };
                (
                    lane_rank,
                    u16::MAX.saturating_sub(sample.strength),
                    sample.age_ticks,
                    sample.source_entity_id,
                )
            })
    }

    fn aggro_sample_for(&self, tick: Tick, stimulus: AggroStimulus) -> Option<AggroTrailSample> {
        if tick.0 < stimulus.tick.0 {
            return None;
        }

        let age_ticks = tick.0 - stimulus.tick.0;
        let lane = if age_ticks <= self.config.direct_aggro_ticks {
            AggroTrailLane::Direct
        } else if age_ticks <= self.config.aggro_memory_ticks {
            AggroTrailLane::Memory
        } else {
            return None;
        };

        Some(AggroTrailSample {
            source_entity_id: stimulus.source_entity_id,
            position: stimulus.position,
            last_seen_tick: stimulus.tick,
            age_ticks,
            strength: stimulus.strength,
            lane,
        })
    }

    fn ai_lod(&self, position: WorldPosition, focus: WorldPosition) -> SwarmAiLod {
        let distance_sq_mm = distance_sq_mm(position, focus);
        let full_sq = i128::from(self.config.full_lod_distance_mm)
            * i128::from(self.config.full_lod_distance_mm);
        let reduced_sq = i128::from(self.config.reduced_lod_distance_mm)
            * i128::from(self.config.reduced_lod_distance_mm);

        if distance_sq_mm <= full_sq as u64 {
            SwarmAiLod::Full
        } else if distance_sq_mm <= reduced_sq as u64 {
            SwarmAiLod::Reduced
        } else {
            SwarmAiLod::Aggregate
        }
    }

    fn sync_positions_from_collision_world(&mut self, collision_world: &CollisionWorld) -> usize {
        let entity_ids = self.active_entities.keys().copied().collect::<Vec<_>>();
        let mut synced_position_count = 0usize;
        for entity_id in entity_ids {
            let Some(body) = collision_world.body(entity_id) else {
                continue;
            };
            let Some(entity) = self.active_entities.get_mut(&entity_id) else {
                continue;
            };
            if entity.position != body.position {
                entity.position = body.position;
                synced_position_count += 1;
            }
        }
        synced_position_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RoutePressureAccumulator {
    target_position: WorldPosition,
    active_entity_count: usize,
    nearest_distance_sq_mm: u64,
    farthest_distance_sq_mm: u64,
}

impl RoutePressureAccumulator {
    fn new(target_position: WorldPosition) -> Self {
        Self {
            target_position,
            active_entity_count: 0,
            nearest_distance_sq_mm: u64::MAX,
            farthest_distance_sq_mm: 0,
        }
    }

    fn add(&mut self, distance_sq_mm: u64) {
        self.active_entity_count += 1;
        self.nearest_distance_sq_mm = self.nearest_distance_sq_mm.min(distance_sq_mm);
        self.farthest_distance_sq_mm = self.farthest_distance_sq_mm.max(distance_sq_mm);
    }

    fn finish(self, radius_mm: i32) -> RoutePressureSample {
        RoutePressureSample {
            target_position: self.target_position,
            active_entity_count: self.active_entity_count,
            radius_mm,
            nearest_distance_sq_mm: self.nearest_distance_sq_mm,
            farthest_distance_sq_mm: self.farthest_distance_sq_mm,
        }
    }
}

fn distance_sq_mm(from: WorldPosition, to: WorldPosition) -> u64 {
    let dx = i128::from(to.x_mm) - i128::from(from.x_mm);
    let dy = i128::from(to.y_mm) - i128::from(from.y_mm);
    (dx * dx + dy * dy) as u64
}

fn swarm_preview_cell_for_position(position: WorldPosition, cell_size_mm: i32) -> SpatialCell {
    let cell_size_mm = cell_size_mm.max(1);
    SpatialCell {
        x: position.x_mm.div_euclid(cell_size_mm),
        y: position.y_mm.div_euclid(cell_size_mm),
    }
}

fn swarm_static_obstacle_from_map_shape(index: usize, shape: &MapShape) -> CollisionBody {
    CollisionBody {
        entity_id: EntityId(SWARM_MAP_OBSTACLE_ENTITY_ID_BASE.saturating_add(index as u64)),
        kind: CollisionBodyKind::StaticObstacle,
        position: WorldPosition {
            x_mm: shape.center.x_mm,
            y_mm: shape.center.y_mm,
        },
        radius_mm: shape.half_extents_mm.x_mm.max(shape.half_extents_mm.y_mm),
    }
}

fn swarm_blocker_cells_for_map_shape(
    shape: &MapShape,
    cell_size_mm: i32,
    bounds: FlowFieldBounds,
    clearance_mm: i32,
) -> BTreeSet<SpatialCell> {
    let cell_size_mm = cell_size_mm.max(1);
    let clearance_mm = clearance_mm.max(0);
    let min_cell = swarm_preview_cell_for_position(
        WorldPosition {
            x_mm: shape
                .center
                .x_mm
                .saturating_sub(shape.half_extents_mm.x_mm.saturating_add(clearance_mm)),
            y_mm: shape
                .center
                .y_mm
                .saturating_sub(shape.half_extents_mm.y_mm.saturating_add(clearance_mm)),
        },
        cell_size_mm,
    );
    let max_cell = swarm_preview_cell_for_position(
        WorldPosition {
            x_mm: shape
                .center
                .x_mm
                .saturating_add(shape.half_extents_mm.x_mm.saturating_add(clearance_mm)),
            y_mm: shape
                .center
                .y_mm
                .saturating_add(shape.half_extents_mm.y_mm.saturating_add(clearance_mm)),
        },
        cell_size_mm,
    );

    let min_x = min_cell.x.max(bounds.min_x);
    let max_x = max_cell.x.min(bounds.max_x);
    let min_y = min_cell.y.max(bounds.min_y);
    let max_y = max_cell.y.min(bounds.max_y);
    if min_x > max_x || min_y > max_y {
        return BTreeSet::new();
    }

    let mut cells = BTreeSet::new();
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            cells.insert(SpatialCell { x, y });
        }
    }
    cells
}

fn swarm_preview_flow_bounds(
    samples: &[SwarmBehaviorSample],
    active_entities: &BTreeMap<EntityId, SwarmEntity>,
    cell_size_mm: i32,
) -> FlowFieldBounds {
    let cell_size_mm = cell_size_mm.max(1);
    let mut cells = Vec::with_capacity(samples.len().saturating_mul(2));
    for sample in samples {
        if let Some(entity) = active_entities.get(&sample.entity_id) {
            cells.push(swarm_preview_cell_for_position(
                entity.position,
                cell_size_mm,
            ));
        }
        cells.push(swarm_preview_cell_for_position(
            sample.intent_target,
            cell_size_mm,
        ));
    }

    let Some(first) = cells.first().copied() else {
        return FlowFieldBounds {
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
        };
    };

    let mut bounds = FlowFieldBounds {
        min_x: first.x,
        min_y: first.y,
        max_x: first.x,
        max_y: first.y,
    };
    for cell in cells.into_iter().skip(1) {
        bounds.min_x = bounds.min_x.min(cell.x);
        bounds.min_y = bounds.min_y.min(cell.y);
        bounds.max_x = bounds.max_x.max(cell.x);
        bounds.max_y = bounds.max_y.max(cell.y);
    }

    FlowFieldBounds {
        min_x: bounds.min_x.saturating_sub(2),
        min_y: bounds.min_y.saturating_sub(2),
        max_x: bounds.max_x.saturating_add(2),
        max_y: bounds.max_y.saturating_add(2),
    }
}

fn swarm_grid_for_snapshot_entities(entities: &[EntityState], cell_size_mm: i32) -> SpatialGrid {
    let mut columns = EntityColumns::with_capacity(entities.len());
    for entity in entities {
        columns.push(*entity);
    }
    let mut grid = SpatialGrid::new(cell_size_mm.max(1));
    grid.rebuild(&columns);
    grid
}

fn snapshot_position_map(snapshot: &Snapshot) -> BTreeMap<EntityId, WorldPosition> {
    snapshot
        .entities
        .iter()
        .map(|entity| (entity.entity_id, entity.position))
        .collect()
}

fn changed_visible_entity_count(
    baseline_positions: &BTreeMap<EntityId, WorldPosition>,
    snapshot: &Snapshot,
) -> usize {
    snapshot
        .entities
        .iter()
        .filter(|entity| {
            baseline_positions
                .get(&entity.entity_id)
                .is_some_and(|baseline_position| *baseline_position != entity.position)
        })
        .count()
}

fn sync_apply_samples_from_active_entities(
    samples: &mut [SwarmMovementApplySample],
    active_entities: &BTreeMap<EntityId, SwarmEntity>,
) -> usize {
    let mut synced_count = 0usize;
    for sample in samples {
        let Some(entity) = active_entities.get(&sample.entity_id) else {
            continue;
        };
        if sample.final_position == entity.position {
            continue;
        }
        sample.final_position = entity.position;
        sample.applied_delta = MovementDelta {
            dx_mm: entity
                .position
                .x_mm
                .saturating_sub(sample.from_position.x_mm),
            dy_mm: entity
                .position
                .y_mm
                .saturating_sub(sample.from_position.y_mm),
        };
        synced_count += 1;
    }
    synced_count
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().max(1) as u64
}

fn percentile_nearest_rank(sorted_values: &[u64], percentile: usize) -> u64 {
    if sorted_values.is_empty() {
        return 0;
    }
    let numerator = sorted_values.len() * percentile;
    let index = numerator.div_ceil(100).saturating_sub(1);
    sorted_values[index.min(sorted_values.len() - 1)]
}

fn run_configured_movement_loop_sample(
    base_swarm: &SwarmState,
    final_tick: Tick,
    focus: WorldPosition,
    movement_mode: SwarmMovementMode,
) -> SwarmConfiguredMovementLoopMeasurementRun {
    let mut swarm = base_swarm.clone();
    swarm.config = match movement_mode {
        SwarmMovementMode::FlowFieldCollision => swarm.config.with_flow_field_collision_movement(
            SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        ),
        SwarmMovementMode::BatchFlowFieldCollision => {
            swarm.config.with_batch_flow_field_collision_movement(
                SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT,
                SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
                1,
            )
        }
        SwarmMovementMode::SpawnOnly => {
            panic!("configured movement loop measurement requires a movement mode")
        }
    };
    let start_positions = swarm
        .active_entities
        .iter()
        .map(|(entity_id, entity)| (*entity_id, entity.position))
        .collect::<BTreeMap<_, _>>();

    let started = Instant::now();
    let mut spawned_count_total = 0usize;
    let mut movement_sample_count_total = 0usize;
    let mut applied_delta_count_total = 0usize;
    let mut physics_iterations_run_total = 0usize;
    let mut flow_field_cache_request_count_total = 0u64;
    let mut flow_field_cache_hit_count_total = 0u64;
    let mut flow_field_cache_eviction_count_total = 0u64;
    let mut flow_field_cache_entry_count_max = 0usize;
    for tick_offset in 0..SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT {
        let report = swarm.tick_with_focus(Tick(final_tick.0 + 1 + tick_offset), focus);
        spawned_count_total += report.spawned_entity_ids.len();
        let movement = report
            .movement
            .as_ref()
            .expect("configured movement measurement tick runs movement");
        movement_sample_count_total += movement.sample_count;
        applied_delta_count_total += movement.applied_delta_count;
        physics_iterations_run_total += movement.physics_iterations_run;
        flow_field_cache_request_count_total += movement.flow_field_cache_request_count;
        flow_field_cache_hit_count_total += movement.flow_field_cache_hit_count;
        flow_field_cache_eviction_count_total += movement.flow_field_cache_eviction_count;
        flow_field_cache_entry_count_max =
            flow_field_cache_entry_count_max.max(movement.flow_field_cache_entry_count);
    }
    let elapsed_us = elapsed_us(started);
    let moved_entity_count = swarm
        .active_entities
        .iter()
        .filter(|(entity_id, entity)| {
            start_positions
                .get(entity_id)
                .copied()
                .is_some_and(|position| position != entity.position)
        })
        .count();

    SwarmConfiguredMovementLoopMeasurementRun {
        sample_count: 1,
        tick_count_per_sample: SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT,
        movement_sample_limit: SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT,
        active_count: swarm.active_count(),
        spawned_count_total,
        elapsed_us_min: elapsed_us,
        elapsed_us_p50: elapsed_us,
        elapsed_us_p95: elapsed_us,
        elapsed_us_p99: elapsed_us,
        elapsed_us_max: elapsed_us,
        movement_sample_count_total,
        applied_delta_count_total,
        physics_iterations_run_total,
        flow_field_cache_request_count_total,
        flow_field_cache_hit_count_total,
        flow_field_cache_eviction_count_total,
        flow_field_cache_entry_count_max,
        moved_entity_count_min: moved_entity_count,
        budget_result: BudgetResult::Blocked,
        claim_scope: "local_measured_harness_only",
    }
}

pub fn run_swarm_configured_movement_loop_measurement(
    sample_count: usize,
) -> SwarmConfiguredMovementLoopMeasurementRun {
    let config = SwarmConfig::local_scale_smoke();
    let mut base_swarm = SwarmState::new(config, load_smoke_spawn_points(), EntityId(50_000))
        .expect("local smoke config is valid");
    prepare_swarm_movement_measurement_base(&mut base_swarm);
    collect_configured_movement_loop_measurement(
        &base_swarm,
        sample_count,
        SwarmMovementMode::FlowFieldCollision,
    )
}

fn prepare_swarm_movement_measurement_base(base_swarm: &mut SwarmState) {
    for tick in 0..250 {
        if tick == 180 {
            base_swarm.record_aggro_stimulus(AggroStimulus {
                source_entity_id: EntityId(1_800),
                position: WorldPosition {
                    x_mm: 35_000,
                    y_mm: 0,
                },
                tick: Tick(tick),
                strength: 40,
            });
        }
        if tick == 240 {
            base_swarm.record_aggro_stimulus(AggroStimulus {
                source_entity_id: EntityId(2_400),
                position: WorldPosition {
                    x_mm: 15_000,
                    y_mm: 5_000,
                },
                tick: Tick(tick),
                strength: 80,
            });
        }
        base_swarm.tick(Tick(tick));
    }
}

fn collect_configured_movement_loop_measurement(
    base_swarm: &SwarmState,
    sample_count: usize,
    movement_mode: SwarmMovementMode,
) -> SwarmConfiguredMovementLoopMeasurementRun {
    let sample_count = sample_count.max(1);

    let final_tick = Tick(249);
    let focus = WorldPosition { x_mm: 0, y_mm: 0 };
    let mut elapsed_samples = Vec::with_capacity(sample_count);
    let mut active_count = base_swarm.active_count();
    let mut spawned_count_total = 0usize;
    let mut movement_sample_count_total = 0usize;
    let mut applied_delta_count_total = 0usize;
    let mut physics_iterations_run_total = 0usize;
    let mut flow_field_cache_request_count_total = 0u64;
    let mut flow_field_cache_hit_count_total = 0u64;
    let mut flow_field_cache_eviction_count_total = 0u64;
    let mut flow_field_cache_entry_count_max = 0usize;
    let mut moved_entity_count_min = usize::MAX;

    for _ in 0..sample_count {
        let sample =
            run_configured_movement_loop_sample(base_swarm, final_tick, focus, movement_mode);
        elapsed_samples.push(sample.elapsed_us_min);
        active_count = sample.active_count;
        spawned_count_total += sample.spawned_count_total;
        movement_sample_count_total += sample.movement_sample_count_total;
        applied_delta_count_total += sample.applied_delta_count_total;
        physics_iterations_run_total += sample.physics_iterations_run_total;
        flow_field_cache_request_count_total += sample.flow_field_cache_request_count_total;
        flow_field_cache_hit_count_total += sample.flow_field_cache_hit_count_total;
        flow_field_cache_eviction_count_total += sample.flow_field_cache_eviction_count_total;
        flow_field_cache_entry_count_max =
            flow_field_cache_entry_count_max.max(sample.flow_field_cache_entry_count_max);
        moved_entity_count_min = moved_entity_count_min.min(sample.moved_entity_count_min);
    }

    elapsed_samples.sort_unstable();
    SwarmConfiguredMovementLoopMeasurementRun {
        sample_count,
        tick_count_per_sample: SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT,
        movement_sample_limit: SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT,
        active_count,
        spawned_count_total,
        elapsed_us_min: elapsed_samples[0],
        elapsed_us_p50: percentile_nearest_rank(&elapsed_samples, 50),
        elapsed_us_p95: percentile_nearest_rank(&elapsed_samples, 95),
        elapsed_us_p99: percentile_nearest_rank(&elapsed_samples, 99),
        elapsed_us_max: *elapsed_samples
            .last()
            .expect("sample count is clamped to at least one"),
        movement_sample_count_total,
        applied_delta_count_total,
        physics_iterations_run_total,
        flow_field_cache_request_count_total,
        flow_field_cache_hit_count_total,
        flow_field_cache_eviction_count_total,
        flow_field_cache_entry_count_max,
        moved_entity_count_min,
        budget_result: BudgetResult::Blocked,
        claim_scope: "local_measured_harness_only",
    }
}

pub fn run_swarm_batch_vs_single_movement_loop_measurement(
    sample_count: usize,
) -> SwarmBatchVsSingleMovementLoopMeasurementRun {
    let sample_count = sample_count.max(1);
    let config = SwarmConfig::local_scale_smoke();
    let mut base_swarm = SwarmState::new(config, load_smoke_spawn_points(), EntityId(50_000))
        .expect("local smoke config is valid");
    prepare_swarm_movement_measurement_base(&mut base_swarm);

    let single = collect_configured_movement_loop_measurement(
        &base_swarm,
        sample_count,
        SwarmMovementMode::FlowFieldCollision,
    );
    let batch = collect_configured_movement_loop_measurement(
        &base_swarm,
        sample_count,
        SwarmMovementMode::BatchFlowFieldCollision,
    );
    let batch_to_single_elapsed_p95_bps = if single.elapsed_us_p95 == 0 {
        0
    } else {
        ((batch.elapsed_us_p95 as u128 * 10_000) / single.elapsed_us_p95 as u128)
            .min(u32::MAX as u128) as u32
    };

    SwarmBatchVsSingleMovementLoopMeasurementRun {
        sample_count,
        tick_count_per_sample: SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT,
        movement_sample_limit: SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT,
        active_count: single.active_count,
        single_elapsed_us_p50: single.elapsed_us_p50,
        single_elapsed_us_p95: single.elapsed_us_p95,
        single_elapsed_us_p99: single.elapsed_us_p99,
        batch_elapsed_us_p50: batch.elapsed_us_p50,
        batch_elapsed_us_p95: batch.elapsed_us_p95,
        batch_elapsed_us_p99: batch.elapsed_us_p99,
        batch_to_single_elapsed_p95_bps,
        single_movement_sample_count_total: single.movement_sample_count_total,
        batch_movement_sample_count_total: batch.movement_sample_count_total,
        single_applied_delta_count_total: single.applied_delta_count_total,
        batch_applied_delta_count_total: batch.applied_delta_count_total,
        single_physics_iterations_run_total: single.physics_iterations_run_total,
        batch_physics_iterations_run_total: batch.physics_iterations_run_total,
        single_flow_field_cache_hit_count_total: single.flow_field_cache_hit_count_total,
        batch_flow_field_cache_hit_count_total: batch.flow_field_cache_hit_count_total,
        single_flow_field_cache_eviction_count_total: single.flow_field_cache_eviction_count_total,
        batch_flow_field_cache_eviction_count_total: batch.flow_field_cache_eviction_count_total,
        single_moved_entity_count_min: single.moved_entity_count_min,
        batch_moved_entity_count_min: batch.moved_entity_count_min,
        budget_result: BudgetResult::Blocked,
        claim_scope: "local_batch_vs_single_measured_harness_only",
    }
}

pub fn run_swarm_batch_movement_replication_smoke() -> SwarmBatchMovementReplicationSmokeRun {
    let tick = Tick(1_500);
    let focus = WorldPosition { x_mm: 0, y_mm: 0 };
    let config = SwarmConfig::local_scale_smoke().with_batch_flow_field_collision_movement(
        SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        1,
    );
    let mut swarm = SwarmState::new(config, load_smoke_spawn_points(), EntityId(50_000))
        .expect("local smoke config is valid");
    prepare_swarm_movement_measurement_base(&mut swarm);

    let baseline_snapshot = swarm.build_full_snapshot(20_000, tick);
    let baseline_positions = snapshot_position_map(&baseline_snapshot);
    let movement = swarm.tick_with_focus(tick, focus);
    let movement_snapshot = swarm.build_full_snapshot(20_001, tick);
    let delta = swarm.build_interest_delta_snapshot(
        PlayerSessionId(80),
        AoiRegion::new(SpatialCell { x: 0, y: 0 }, 128),
        20_002,
        baseline_snapshot.snapshot_id,
        tick,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let aggregate_delta = swarm.build_interest_delta_snapshot(
        PlayerSessionId(81),
        AoiRegion::new(SpatialCell { x: -6, y: -6 }, 1),
        20_003,
        baseline_snapshot.snapshot_id,
        tick,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let delta_snapshot_bytes = estimate_snapshot_bytes(&delta.snapshot);
    let aggregate_snapshot_bytes = estimate_snapshot_bytes(&aggregate_delta.snapshot);
    let movement_report = movement
        .movement
        .as_ref()
        .expect("batch movement config emits a movement report");

    SwarmBatchMovementReplicationSmokeRun {
        tick,
        active_count: swarm.active_count(),
        baseline_snapshot_entity_count: baseline_snapshot.entities.len(),
        movement_sample_count: movement_report.sample_count,
        movement_applied_delta_count: movement_report.applied_delta_count,
        movement_physics_iterations_run: movement_report.physics_iterations_run,
        movement_snapshot_entity_count: movement_snapshot.entities.len(),
        delta_visible_entity_count: delta.visible_entity_count,
        delta_changed_visible_entity_count: changed_visible_entity_count(
            &baseline_positions,
            &delta.snapshot,
        ),
        delta_removed_entity_count: delta.removed_entity_count,
        delta_aggregate_far_state_count: delta.aggregate_far_state_count,
        delta_snapshot_bytes,
        delta_bandwidth_kb_s_per_client: bandwidth_kb_s(delta_snapshot_bytes, 20.0),
        aggregate_visible_entity_count: aggregate_delta.visible_entity_count,
        aggregate_changed_visible_entity_count: changed_visible_entity_count(
            &baseline_positions,
            &aggregate_delta.snapshot,
        ),
        aggregate_far_state_count: aggregate_delta.aggregate_far_state_count,
        aggregate_snapshot_bytes,
        budget_result: BudgetResult::Blocked,
        claim_scope: "local_batch_movement_replication_smoke_only",
    }
}

pub fn run_swarm_load_smoke() -> SwarmLoadSmokeRun {
    let total_started = Instant::now();
    let config = SwarmConfig::local_scale_smoke();
    let mut swarm = SwarmState::new(config, load_smoke_spawn_points(), EntityId(50_000))
        .expect("local smoke config is valid");

    let spawn_ticks_started = Instant::now();
    for tick in 0..250 {
        if tick == 180 {
            swarm.record_aggro_stimulus(AggroStimulus {
                source_entity_id: EntityId(1_800),
                position: WorldPosition {
                    x_mm: 35_000,
                    y_mm: 0,
                },
                tick: Tick(tick),
                strength: 40,
            });
        }
        if tick == 240 {
            swarm.record_aggro_stimulus(AggroStimulus {
                source_entity_id: EntityId(2_400),
                position: WorldPosition {
                    x_mm: 15_000,
                    y_mm: 5_000,
                },
                tick: Tick(tick),
                strength: 80,
            });
        }
        swarm.tick(Tick(tick));
    }
    let spawn_ticks_elapsed_us = elapsed_us(spawn_ticks_started);

    let final_tick = Tick(249);
    let focus = WorldPosition { x_mm: 0, y_mm: 0 };
    let behavior_started = Instant::now();
    let behavior = swarm.evaluate_behavior(final_tick, focus);
    let behavior_elapsed_us = elapsed_us(behavior_started);
    let movement_preview_started = Instant::now();
    let movement_preview = swarm.movement_preview(
        final_tick,
        focus,
        SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let movement_preview_elapsed_us = elapsed_us(movement_preview_started);
    let mut movement_tick_swarm = swarm.clone();
    let movement_tick_started = Instant::now();
    let movement_tick = movement_tick_swarm.tick_with_flow_field_movement(
        final_tick,
        focus,
        SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        1,
    );
    let movement_tick_elapsed_us = elapsed_us(movement_tick_started);
    let movement_apply = &movement_tick.movement_report;
    let mut batch_movement_tick_swarm = swarm.clone();
    let batch_movement_tick_started = Instant::now();
    let batch_movement_tick = batch_movement_tick_swarm.tick_with_batch_flow_field_movement(
        final_tick,
        focus,
        SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        1,
    );
    let batch_movement_tick_elapsed_us = elapsed_us(batch_movement_tick_started);
    let mut clamped_movement_apply_swarm = swarm.clone();
    let clamped_movement_apply = clamped_movement_apply_swarm
        .apply_flow_field_movement_step_with_correction_limit(
            final_tick,
            focus,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
            SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM,
        );
    let mut batch_movement_apply_swarm = swarm.clone();
    let batch_movement_apply = batch_movement_apply_swarm.apply_flow_field_batch_movement_step(
        final_tick,
        focus,
        SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        1,
    );
    let mut configured_movement_tick_swarm = swarm.clone();
    configured_movement_tick_swarm.config = configured_movement_tick_swarm
        .config
        .with_flow_field_collision_movement(
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
    let configured_movement_tick_started = Instant::now();
    let configured_movement_tick =
        configured_movement_tick_swarm.tick_with_focus(final_tick, focus);
    let configured_movement_tick_elapsed_us = elapsed_us(configured_movement_tick_started);
    let configured_movement_apply = configured_movement_tick
        .movement
        .as_ref()
        .expect("configured movement tick runs movement");
    let mut configured_batch_movement_tick_swarm = swarm.clone();
    configured_batch_movement_tick_swarm.config = configured_batch_movement_tick_swarm
        .config
        .with_batch_flow_field_collision_movement(
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
    let configured_batch_movement_tick_started = Instant::now();
    let configured_batch_movement_tick =
        configured_batch_movement_tick_swarm.tick_with_focus(final_tick, focus);
    let configured_batch_movement_tick_elapsed_us =
        elapsed_us(configured_batch_movement_tick_started);
    let configured_batch_movement_apply = configured_batch_movement_tick
        .movement
        .as_ref()
        .expect("configured batch movement tick runs movement");
    let mut configured_clamped_movement_tick_swarm = swarm.clone();
    configured_clamped_movement_tick_swarm.config = configured_clamped_movement_tick_swarm
        .config
        .with_flow_field_collision_movement(
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        )
        .with_flow_field_collision_movement_correction_limit(
            SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM,
        );
    let configured_clamped_movement_tick =
        configured_clamped_movement_tick_swarm.tick_with_focus(final_tick, focus);
    let configured_clamped_movement_apply = configured_clamped_movement_tick
        .movement
        .as_ref()
        .expect("configured clamped movement tick runs movement");
    let mut configured_movement_loop_swarm = swarm.clone();
    configured_movement_loop_swarm.config = configured_movement_loop_swarm
        .config
        .with_flow_field_collision_movement(
            SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
    let configured_movement_loop_start_positions = configured_movement_loop_swarm
        .active_entities
        .iter()
        .map(|(entity_id, entity)| (*entity_id, entity.position))
        .collect::<BTreeMap<_, _>>();
    let configured_movement_loop_started = Instant::now();
    let mut configured_movement_loop_spawned_count = 0usize;
    let mut configured_movement_loop_sample_count = 0usize;
    let mut configured_movement_loop_applied_delta_count = 0usize;
    let mut configured_movement_loop_physics_iterations_run = 0usize;
    let mut configured_movement_loop_flow_field_cache_request_count = 0u64;
    let mut configured_movement_loop_flow_field_cache_hit_count = 0u64;
    let mut configured_movement_loop_flow_field_cache_eviction_count = 0u64;
    let mut configured_movement_loop_flow_field_cache_entry_count = 0usize;
    for tick_offset in 0..SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT {
        let report = configured_movement_loop_swarm
            .tick_with_focus(Tick(final_tick.0 + 1 + tick_offset), focus);
        configured_movement_loop_spawned_count += report.spawned_entity_ids.len();
        let movement = report
            .movement
            .as_ref()
            .expect("configured movement loop tick runs movement");
        configured_movement_loop_sample_count += movement.sample_count;
        configured_movement_loop_applied_delta_count += movement.applied_delta_count;
        configured_movement_loop_physics_iterations_run += movement.physics_iterations_run;
        configured_movement_loop_flow_field_cache_request_count +=
            movement.flow_field_cache_request_count;
        configured_movement_loop_flow_field_cache_hit_count += movement.flow_field_cache_hit_count;
        configured_movement_loop_flow_field_cache_eviction_count +=
            movement.flow_field_cache_eviction_count;
        configured_movement_loop_flow_field_cache_entry_count =
            configured_movement_loop_flow_field_cache_entry_count
                .max(movement.flow_field_cache_entry_count);
    }
    let configured_movement_loop_elapsed_us = elapsed_us(configured_movement_loop_started);
    let configured_movement_loop_active_count = configured_movement_loop_swarm.active_count();
    let configured_movement_loop_moved_entity_count = configured_movement_loop_swarm
        .active_entities
        .iter()
        .filter(|(entity_id, entity)| {
            configured_movement_loop_start_positions
                .get(entity_id)
                .copied()
                .is_some_and(|position| position != entity.position)
        })
        .count();
    let static_obstacle_movement_started = Instant::now();
    let mut static_obstacle_swarm = swarm.clone();
    let static_obstacle_map = load_smoke_obstacle_map_data();
    static_obstacle_swarm
        .set_static_obstacles_from_map_data(&static_obstacle_map)
        .expect("load smoke obstacle map data is valid");
    let static_obstacle_behavior = static_obstacle_swarm.evaluate_behavior(final_tick, focus);
    let static_obstacle_behaviors = static_obstacle_behavior
        .samples
        .iter()
        .take(SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT)
        .copied()
        .collect::<Vec<_>>();
    let static_obstacle_flow_bounds = swarm_preview_flow_bounds(
        &static_obstacle_behaviors,
        &static_obstacle_swarm.active_entities,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let static_obstacle_blocker_cell_count = static_obstacle_swarm
        .static_obstacle_blocker_cells(
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            static_obstacle_flow_bounds,
            static_obstacle_swarm.config.collision_radius_mm,
        )
        .len();
    let static_obstacle_movement = static_obstacle_swarm.apply_flow_field_movement_step(
        final_tick,
        focus,
        SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        1,
    );
    let static_obstacle_movement_elapsed_us = elapsed_us(static_obstacle_movement_started);
    let snapshot_started = Instant::now();
    let movement_snapshot = movement_tick_swarm.build_full_snapshot(10_000, final_tick);
    let movement_tick_snapshot_bytes = estimate_snapshot_bytes(&movement_snapshot);
    let movement_delta_snapshot = movement_tick_swarm.build_interest_delta_snapshot(
        PlayerSessionId(77),
        AoiRegion::new(SpatialCell { x: 0, y: 0 }, 128),
        10_001,
        10_000,
        final_tick,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let movement_delta_snapshot_bytes = estimate_snapshot_bytes(&movement_delta_snapshot.snapshot);
    let movement_aggregate_delta_snapshot = movement_tick_swarm.build_interest_delta_snapshot(
        PlayerSessionId(78),
        AoiRegion::new(SpatialCell { x: -6, y: -6 }, 1),
        10_002,
        10_000,
        final_tick,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let movement_aggregate_delta_snapshot_bytes =
        estimate_snapshot_bytes(&movement_aggregate_delta_snapshot.snapshot);
    let batch_movement_snapshot = batch_movement_tick_swarm.build_full_snapshot(10_003, final_tick);
    let batch_movement_tick_snapshot_bytes = estimate_snapshot_bytes(&batch_movement_snapshot);
    let batch_movement_delta_snapshot = batch_movement_tick_swarm.build_interest_delta_snapshot(
        PlayerSessionId(79),
        AoiRegion::new(SpatialCell { x: 0, y: 0 }, 128),
        10_004,
        10_003,
        final_tick,
        SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
    );
    let batch_movement_delta_snapshot_bytes =
        estimate_snapshot_bytes(&batch_movement_delta_snapshot.snapshot);
    let snapshot_elapsed_us = elapsed_us(snapshot_started);
    let collision_diagnostics_started = Instant::now();
    let tick_report = swarm.tick(final_tick);
    let collision_bodies = swarm.collision_bodies();
    let collision_world = crate::CollisionWorld::with_bodies(2_000, collision_bodies.clone())
        .expect("swarm collision bodies are valid");
    let collision_contacts = collision_world.detect_overlaps();
    let resolution_plan = collision_world.plan_overlap_resolution();
    let mut physics_world = collision_world.clone();
    let physics_step = physics_world.step_overlap_resolution(2);
    let collision_movement_probe_corrected_count = movement_preview.corrected_delta_count;
    let collision_movement_probe_blocked_count = movement_preview.blocked_delta_count;
    let collision_batch_movement_probe = collision_world.probe_batch_movements_after_resolution(
        movement_preview
            .samples
            .iter()
            .map(|sample| CollisionBatchMovementCandidate {
                entity_id: sample.entity_id,
                target_position: WorldPosition {
                    x_mm: sample
                        .from_position
                        .x_mm
                        .saturating_add(sample.requested_delta.dx_mm),
                    y_mm: sample
                        .from_position
                        .y_mm
                        .saturating_add(sample.requested_delta.dy_mm),
                },
            }),
        1,
    );
    let collision_resolved_admission_accepted_after_resolution_count = movement_preview
        .samples
        .iter()
        .filter(|sample| sample.collision_decision == CollisionMovementDecision::Corrected)
        .count();
    let collision_resolved_admission_rejected_count = movement_preview
        .samples
        .iter()
        .filter(|sample| sample.collision_decision == CollisionMovementDecision::Blocked)
        .count();
    let collision_resolved_admission_iterations_run_count = movement_preview
        .samples
        .iter()
        .map(|sample| sample.collision_iterations_run)
        .sum();
    let collision_resolved_admission_correction_count = movement_preview
        .samples
        .iter()
        .map(|sample| sample.collision_applied_correction_count)
        .sum();
    let collision_admission_check_count =
        SWARM_COLLISION_ADMISSION_SAMPLE_COUNT.min(collision_bodies.len());
    let resolved_admission_initial_rejected_count = movement_preview
        .samples
        .iter()
        .filter(|sample| {
            matches!(
                sample.collision_decision,
                CollisionMovementDecision::Corrected | CollisionMovementDecision::Blocked
            )
        })
        .count();
    let extra_collision_admission_rejected_count = collision_bodies
        .iter()
        .skip(movement_preview.sample_count)
        .take(collision_admission_check_count.saturating_sub(movement_preview.sample_count))
        .map(|body| collision_world.admit_body_position(body.entity_id, body.position))
        .filter(|admission| admission.result == CollisionAdmissionResult::RejectedOverlap)
        .count();
    let collision_admission_rejected_count =
        resolved_admission_initial_rejected_count + extra_collision_admission_rejected_count;
    let collision_diagnostics_elapsed_us = elapsed_us(collision_diagnostics_started);
    let estimated_snapshot_bytes =
        SNAPSHOT_HEADER_BYTES + collision_bodies.len() as u64 * SNAPSHOT_ENTITY_BYTES;
    let estimated_bandwidth_kb_s_per_client = bandwidth_kb_s(estimated_snapshot_bytes, 1.0 / 20.0);
    let budget_result = evaluate_server_perf_budget(
        ServerPerfScenario::Sim1kSingleClient,
        ServerPerfReport {
            server_tick_p99_ms: Some(0.0),
            sim_p95_ms: None,
            bandwidth_kb_s_per_client_p95: None,
            reconnect_full_snapshot_p95_s: None,
        },
        ServerPerfBudgets::default(),
    );
    let local_smoke_total_elapsed_us = elapsed_us(total_started);

    SwarmLoadSmokeRun {
        tick_count: 250,
        active_count: swarm.active_count(),
        spawned_total: swarm.spawned_total(),
        local_smoke_total_elapsed_us,
        spawn_ticks_elapsed_us,
        behavior_elapsed_us,
        movement_preview_elapsed_us,
        movement_tick_elapsed_us,
        batch_movement_tick_elapsed_us,
        configured_movement_tick_elapsed_us,
        configured_batch_movement_tick_elapsed_us,
        configured_movement_loop_elapsed_us,
        static_obstacle_movement_elapsed_us,
        snapshot_elapsed_us,
        collision_diagnostics_elapsed_us,
        behavior_sample_count: behavior.samples.len(),
        route_pressure_bucket_count: tick_report.route_pressure.len(),
        aggro_trail_count: tick_report.aggro_trails.len(),
        direct_aggro_count: tick_report
            .aggro_trails
            .iter()
            .filter(|sample| sample.lane == AggroTrailLane::Direct)
            .count(),
        memory_aggro_count: tick_report
            .aggro_trails
            .iter()
            .filter(|sample| sample.lane == AggroTrailLane::Memory)
            .count(),
        ai_lod_counts: behavior.lod_counts,
        collision_body_count: collision_bodies.len(),
        collision_contact_count: collision_contacts.len(),
        collision_admission_check_count,
        collision_admission_rejected_count,
        collision_resolved_admission_check_count: movement_preview.sample_count,
        collision_resolved_admission_accepted_after_resolution_count,
        collision_resolved_admission_rejected_count,
        collision_resolved_admission_iterations_run_count,
        collision_resolved_admission_correction_count,
        collision_movement_probe_count: movement_preview.sample_count,
        collision_movement_probe_corrected_count,
        collision_movement_probe_blocked_count,
        collision_batch_movement_probe_count: collision_batch_movement_probe.candidate_count,
        collision_batch_movement_probe_accepted_count: collision_batch_movement_probe
            .accepted_count,
        collision_batch_movement_probe_corrected_count: collision_batch_movement_probe
            .corrected_count,
        collision_batch_movement_probe_blocked_count: collision_batch_movement_probe.blocked_count,
        collision_batch_movement_probe_unknown_body_count: collision_batch_movement_probe
            .unknown_body_count,
        collision_batch_movement_probe_iterations_run_count: collision_batch_movement_probe
            .iterations_run,
        collision_batch_movement_probe_correction_count: collision_batch_movement_probe
            .applied_correction_count,
        collision_batch_movement_probe_correction_abs_mm_total: collision_batch_movement_probe
            .applied_correction_abs_mm_total,
        collision_batch_movement_probe_max_correction_abs_mm: collision_batch_movement_probe
            .max_applied_correction_abs_mm,
        collision_batch_movement_probe_final_contact_count: collision_batch_movement_probe
            .final_contact_count,
        movement_preview_sample_count: movement_preview.sample_count,
        movement_preview_flow_field_build_count: movement_preview.flow_field_build_count,
        movement_preview_flow_field_query_count: movement_preview.flow_field_query_count,
        movement_preview_flow_field_unreachable_count: movement_preview
            .flow_field_unreachable_count,
        movement_preview_physics_candidate_count: movement_preview.physics_candidate_count,
        movement_preview_physics_initial_contact_count: movement_preview
            .physics_initial_contact_count,
        movement_preview_physics_iterations_run: movement_preview.physics_iterations_run,
        movement_preview_physics_applied_correction_count: movement_preview
            .physics_applied_correction_count,
        movement_preview_physics_applied_correction_abs_mm_total: movement_preview
            .physics_applied_correction_abs_mm_total,
        movement_preview_physics_max_applied_correction_abs_mm: movement_preview
            .physics_max_applied_correction_abs_mm,
        movement_preview_physics_final_contact_count: movement_preview.physics_final_contact_count,
        movement_preview_applied_delta_count: movement_preview.applied_delta_count,
        movement_preview_corrected_delta_count: movement_preview.corrected_delta_count,
        movement_preview_blocked_delta_count: movement_preview.blocked_delta_count,
        movement_apply_sample_count: movement_apply.sample_count,
        movement_apply_flow_field_build_count: movement_apply.flow_field_build_count,
        movement_apply_flow_field_cache_request_count: movement_apply
            .flow_field_cache_request_count,
        movement_apply_flow_field_cache_hit_count: movement_apply.flow_field_cache_hit_count,
        movement_apply_flow_field_cache_eviction_count: movement_apply
            .flow_field_cache_eviction_count,
        movement_apply_flow_field_cache_entry_count: movement_apply.flow_field_cache_entry_count,
        movement_apply_flow_field_query_count: movement_apply.flow_field_query_count,
        movement_apply_flow_field_unreachable_count: movement_apply.flow_field_unreachable_count,
        movement_apply_applied_delta_count: movement_apply.applied_delta_count,
        movement_apply_corrected_delta_count: movement_apply.corrected_delta_count,
        movement_apply_blocked_delta_count: movement_apply.blocked_delta_count,
        movement_apply_movement_probe_correction_limit_abs_mm: movement_apply
            .movement_probe_correction_limit_abs_mm,
        movement_apply_movement_probe_clamped_correction_count: movement_apply
            .movement_probe_clamped_correction_count,
        movement_apply_physics_candidate_count: movement_apply.physics_candidate_count,
        movement_apply_physics_initial_contact_count: movement_apply.physics_initial_contact_count,
        movement_apply_physics_iterations_run: movement_apply.physics_iterations_run,
        movement_apply_physics_applied_correction_count: movement_apply
            .physics_applied_correction_count,
        movement_apply_physics_applied_correction_abs_mm_total: movement_apply
            .physics_applied_correction_abs_mm_total,
        movement_apply_physics_max_applied_correction_abs_mm: movement_apply
            .physics_max_applied_correction_abs_mm,
        movement_apply_physics_correction_limit_abs_mm: movement_apply
            .physics_correction_limit_abs_mm,
        movement_apply_physics_clamped_correction_count: movement_apply
            .physics_clamped_correction_count,
        movement_apply_physics_final_contact_count: movement_apply.physics_final_contact_count,
        movement_apply_physics_synced_position_count: movement_apply.physics_synced_position_count,
        movement_apply_physics_sample_synced_position_count: movement_apply
            .physics_sample_synced_position_count,
        clamped_movement_apply_sample_count: clamped_movement_apply.sample_count,
        clamped_movement_apply_movement_probe_correction_limit_abs_mm: clamped_movement_apply
            .movement_probe_correction_limit_abs_mm,
        clamped_movement_apply_movement_probe_clamped_correction_count: clamped_movement_apply
            .movement_probe_clamped_correction_count,
        clamped_movement_apply_physics_correction_limit_abs_mm: clamped_movement_apply
            .physics_correction_limit_abs_mm,
        clamped_movement_apply_physics_clamped_correction_count: clamped_movement_apply
            .physics_clamped_correction_count,
        clamped_movement_apply_physics_max_applied_correction_abs_mm: clamped_movement_apply
            .physics_max_applied_correction_abs_mm,
        batch_movement_apply_sample_count: batch_movement_apply.sample_count,
        batch_movement_apply_flow_field_cache_request_count: batch_movement_apply
            .flow_field_cache_request_count,
        batch_movement_apply_flow_field_cache_hit_count: batch_movement_apply
            .flow_field_cache_hit_count,
        batch_movement_apply_flow_field_cache_eviction_count: batch_movement_apply
            .flow_field_cache_eviction_count,
        batch_movement_apply_flow_field_cache_entry_count: batch_movement_apply
            .flow_field_cache_entry_count,
        batch_movement_apply_applied_delta_count: batch_movement_apply.applied_delta_count,
        batch_movement_apply_corrected_delta_count: batch_movement_apply.corrected_delta_count,
        batch_movement_apply_blocked_delta_count: batch_movement_apply.blocked_delta_count,
        batch_movement_apply_movement_probe_correction_limit_abs_mm: batch_movement_apply
            .movement_probe_correction_limit_abs_mm,
        batch_movement_apply_movement_probe_clamped_correction_count: batch_movement_apply
            .movement_probe_clamped_correction_count,
        batch_movement_apply_physics_candidate_count: batch_movement_apply.physics_candidate_count,
        batch_movement_apply_physics_iterations_run: batch_movement_apply.physics_iterations_run,
        batch_movement_apply_physics_synced_position_count: batch_movement_apply
            .physics_synced_position_count,
        batch_movement_apply_physics_sample_synced_position_count: batch_movement_apply
            .physics_sample_synced_position_count,
        batch_movement_tick_sample_count: batch_movement_tick.movement_report.sample_count,
        batch_movement_tick_active_count: batch_movement_tick.tick_report.active_count,
        batch_movement_tick_spawned_count: batch_movement_tick.tick_report.spawned_entity_ids.len(),
        batch_movement_tick_applied_delta_count: batch_movement_tick
            .movement_report
            .applied_delta_count,
        batch_movement_tick_physics_iterations_run: batch_movement_tick
            .movement_report
            .physics_iterations_run,
        batch_movement_tick_snapshot_entity_count: batch_movement_snapshot.entities.len(),
        batch_movement_tick_snapshot_bytes,
        batch_movement_delta_snapshot_entity_count: batch_movement_delta_snapshot
            .snapshot
            .entities
            .len(),
        batch_movement_delta_snapshot_bytes,
        movement_tick_sample_count: movement_tick.movement_report.sample_count,
        movement_tick_active_count: movement_tick.tick_report.active_count,
        movement_tick_spawned_count: movement_tick.tick_report.spawned_entity_ids.len(),
        movement_tick_applied_delta_count: movement_tick.movement_report.applied_delta_count,
        movement_tick_physics_iterations_run: movement_tick.movement_report.physics_iterations_run,
        configured_movement_tick_sample_count: configured_movement_apply.sample_count,
        configured_movement_tick_active_count: configured_movement_tick.active_count,
        configured_movement_tick_spawned_count: configured_movement_tick.spawned_entity_ids.len(),
        configured_movement_tick_applied_delta_count: configured_movement_apply.applied_delta_count,
        configured_movement_tick_physics_iterations_run: configured_movement_apply
            .physics_iterations_run,
        configured_batch_movement_tick_sample_count: configured_batch_movement_apply.sample_count,
        configured_batch_movement_tick_active_count: configured_batch_movement_tick.active_count,
        configured_batch_movement_tick_spawned_count: configured_batch_movement_tick
            .spawned_entity_ids
            .len(),
        configured_batch_movement_tick_applied_delta_count: configured_batch_movement_apply
            .applied_delta_count,
        configured_batch_movement_tick_physics_iterations_run: configured_batch_movement_apply
            .physics_iterations_run,
        configured_batch_movement_tick_claim_scope: configured_batch_movement_apply.claim_scope,
        configured_clamped_movement_tick_sample_count: configured_clamped_movement_apply
            .sample_count,
        configured_clamped_movement_tick_movement_probe_correction_limit_abs_mm:
            configured_clamped_movement_apply.movement_probe_correction_limit_abs_mm,
        configured_clamped_movement_tick_movement_probe_clamped_correction_count:
            configured_clamped_movement_apply.movement_probe_clamped_correction_count,
        configured_clamped_movement_tick_physics_correction_limit_abs_mm:
            configured_clamped_movement_apply.physics_correction_limit_abs_mm,
        configured_clamped_movement_tick_physics_clamped_correction_count:
            configured_clamped_movement_apply.physics_clamped_correction_count,
        configured_clamped_movement_tick_physics_max_applied_correction_abs_mm:
            configured_clamped_movement_apply.physics_max_applied_correction_abs_mm,
        configured_movement_loop_tick_count: SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT,
        configured_movement_loop_active_count,
        configured_movement_loop_spawned_count,
        configured_movement_loop_sample_count,
        configured_movement_loop_applied_delta_count,
        configured_movement_loop_physics_iterations_run,
        configured_movement_loop_flow_field_cache_request_count,
        configured_movement_loop_flow_field_cache_hit_count,
        configured_movement_loop_flow_field_cache_eviction_count,
        configured_movement_loop_flow_field_cache_entry_count,
        configured_movement_loop_moved_entity_count,
        static_obstacle_count: static_obstacle_swarm.static_obstacle_count(),
        static_obstacle_source: "map_data_import",
        static_obstacle_map_obstacle_count: static_obstacle_map.obstacles.len(),
        static_obstacle_clearance_mm: static_obstacle_swarm.config.collision_radius_mm,
        static_obstacle_blocker_cell_count,
        static_obstacle_movement_sample_count: static_obstacle_movement.sample_count,
        static_obstacle_movement_flow_field_build_count: static_obstacle_movement
            .flow_field_build_count,
        static_obstacle_movement_applied_delta_count: static_obstacle_movement.applied_delta_count,
        static_obstacle_movement_blocked_delta_count: static_obstacle_movement.blocked_delta_count,
        static_obstacle_movement_physics_iterations_run: static_obstacle_movement
            .physics_iterations_run,
        movement_tick_snapshot_entity_count: movement_snapshot.entities.len(),
        movement_tick_snapshot_bytes,
        movement_tick_snapshot_bandwidth_kb_s_per_client: bandwidth_kb_s(
            movement_tick_snapshot_bytes,
            1.0 / 20.0,
        ),
        movement_delta_snapshot_entity_count: movement_delta_snapshot.snapshot.entities.len(),
        movement_delta_snapshot_removed_count: movement_delta_snapshot
            .snapshot
            .removed_entities
            .len(),
        movement_delta_snapshot_aggregate_far_state_count: movement_delta_snapshot
            .aggregate_far_state_count,
        movement_delta_snapshot_bytes,
        movement_delta_snapshot_bandwidth_kb_s_per_client: bandwidth_kb_s(
            movement_delta_snapshot_bytes,
            1.0 / 20.0,
        ),
        movement_aggregate_delta_snapshot_entity_count: movement_aggregate_delta_snapshot
            .snapshot
            .entities
            .len(),
        movement_aggregate_delta_snapshot_removed_count: movement_aggregate_delta_snapshot
            .snapshot
            .removed_entities
            .len(),
        movement_aggregate_delta_snapshot_aggregate_far_state_count:
            movement_aggregate_delta_snapshot.aggregate_far_state_count,
        movement_aggregate_delta_snapshot_bytes,
        movement_aggregate_delta_snapshot_bandwidth_kb_s_per_client: bandwidth_kb_s(
            movement_aggregate_delta_snapshot_bytes,
            1.0 / 20.0,
        ),
        collision_resolution_contact_count: resolution_plan.contact_count,
        collision_resolution_correction_count: resolution_plan.corrections.len(),
        collision_physics_iterations_run: physics_step.iterations_run,
        collision_physics_applied_correction_count: physics_step.applied_correction_count,
        collision_physics_applied_correction_abs_mm_total: physics_step
            .applied_correction_abs_mm_total,
        collision_physics_max_applied_correction_abs_mm: physics_step.max_applied_correction_abs_mm,
        collision_physics_final_contact_count: physics_step.final_contact_count,
        estimated_snapshot_bytes,
        estimated_bandwidth_kb_s_per_client,
        budget_result,
        claim_scope: "informational_contract_only",
    }
}

fn load_smoke_spawn_points() -> Vec<SwarmSpawnPoint> {
    vec![
        SwarmSpawnPoint {
            spawn_id: 1,
            position: WorldPosition {
                x_mm: -12_000,
                y_mm: -12_000,
            },
            route_target: WorldPosition {
                x_mm: 40_000,
                y_mm: 0,
            },
        },
        SwarmSpawnPoint {
            spawn_id: 2,
            position: WorldPosition {
                x_mm: -40_000,
                y_mm: 40_000,
            },
            route_target: WorldPosition {
                x_mm: 40_000,
                y_mm: 0,
            },
        },
        SwarmSpawnPoint {
            spawn_id: 3,
            position: WorldPosition {
                x_mm: 90_000,
                y_mm: 90_000,
            },
            route_target: WorldPosition {
                x_mm: -40_000,
                y_mm: 0,
            },
        },
        SwarmSpawnPoint {
            spawn_id: 4,
            position: WorldPosition {
                x_mm: 12_000,
                y_mm: 12_000,
            },
            route_target: WorldPosition {
                x_mm: -40_000,
                y_mm: 0,
            },
        },
    ]
}

fn load_smoke_obstacle_map_data() -> MapDataImport {
    MapDataImport {
        schema: crate::MAP_DATA_SCHEMA,
        map_id: "swarm-load-smoke-local-map".to_string(),
        map_version: 1,
        checksum: "sum16:swarm-local-obstacles".to_string(),
        bounds_mm: MapBounds {
            min_x: -100_000,
            min_y: -100_000,
            max_x: 100_000,
            max_y: 100_000,
        },
        spawn_points: Vec::new(),
        capture_points: Vec::new(),
        obstacles: vec![
            map_obstacle_shape("obstacle_spawn_1", -12_000, -12_000, 900),
            map_obstacle_shape("obstacle_spawn_2", -40_000, 40_000, 900),
            map_obstacle_shape("obstacle_spawn_3", 90_000, 90_000, 900),
            map_obstacle_shape("obstacle_spawn_4", 12_000, 12_000, 900),
        ],
        cover_objects: Vec::new(),
        navigation_hints: Vec::new(),
    }
}

fn map_obstacle_shape(id: &str, x_mm: i32, y_mm: i32, half_extent_mm: i32) -> MapShape {
    MapShape {
        id: id.to_string(),
        kind: MapShapeKind::Obstacle,
        center: MapPoint { x_mm, y_mm },
        half_extents_mm: MapPoint {
            x_mm: half_extent_mm,
            y_mm: half_extent_mm,
        },
        class_label: "swarm_blocker".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> SwarmConfig {
        SwarmConfig {
            start_tick: Tick(10),
            spawn_interval_ticks: 3,
            spawn_batch_size: 2,
            max_active: 5,
            route_pressure_radius_mm: 4_000,
            collision_radius_mm: 300,
            direct_aggro_ticks: 4,
            aggro_memory_ticks: 12,
            full_lod_distance_mm: 2_000,
            reduced_lod_distance_mm: 12_000,
            movement_mode: SwarmMovementMode::SpawnOnly,
            movement_sample_limit: 2,
            movement_cell_size_mm: SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            movement_physics_iterations: 1,
            movement_correction_limit_abs_mm: None,
        }
    }

    fn spawn_points() -> Vec<SwarmSpawnPoint> {
        vec![
            SwarmSpawnPoint {
                spawn_id: 1,
                position: WorldPosition {
                    x_mm: -1_000,
                    y_mm: 0,
                },
                route_target: WorldPosition {
                    x_mm: 10_000,
                    y_mm: 0,
                },
            },
            SwarmSpawnPoint {
                spawn_id: 2,
                position: WorldPosition {
                    x_mm: 0,
                    y_mm: 2_000,
                },
                route_target: WorldPosition {
                    x_mm: 10_000,
                    y_mm: 0,
                },
            },
            SwarmSpawnPoint {
                spawn_id: 3,
                position: WorldPosition {
                    x_mm: 1_000,
                    y_mm: -2_000,
                },
                route_target: WorldPosition {
                    x_mm: -10_000,
                    y_mm: 0,
                },
            },
        ]
    }

    fn static_obstacle(entity_id: u64, x_mm: i32, y_mm: i32, radius_mm: i32) -> CollisionBody {
        CollisionBody {
            entity_id: EntityId(entity_id),
            kind: CollisionBodyKind::StaticObstacle,
            position: WorldPosition { x_mm, y_mm },
            radius_mm,
        }
    }

    #[test]
    fn swarm_rejects_invalid_authority_inputs() {
        assert_eq!(
            SwarmState::new(config(), Vec::new(), EntityId(100)).unwrap_err(),
            SwarmError::EmptySpawnPoints
        );

        let mut invalid = config();
        invalid.spawn_interval_ticks = 0;
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidSpawnInterval
        );

        invalid = config();
        invalid.spawn_batch_size = 0;
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidSpawnBatchSize
        );

        invalid = config();
        invalid.max_active = 0;
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidMaxActive
        );

        invalid = config();
        invalid.collision_radius_mm = 0;
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidCollisionRadius
        );

        invalid = config();
        invalid.direct_aggro_ticks = 20;
        invalid.aggro_memory_ticks = 10;
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidAggroWindow
        );

        invalid = config();
        invalid.full_lod_distance_mm = 20_000;
        invalid.reduced_lod_distance_mm = 10_000;
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidLodDistance
        );

        invalid = config().with_flow_field_collision_movement(0, 1_000, 1);
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidMovementSampleLimit
        );

        invalid = config().with_flow_field_collision_movement(1, 0, 1);
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidMovementCellSize
        );

        invalid = config().with_flow_field_collision_movement(1, 1_000, 0);
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidMovementPhysicsIterations
        );

        invalid = config().with_batch_flow_field_collision_movement(0, 1_000, 1);
        assert_eq!(
            SwarmState::new(invalid, spawn_points(), EntityId(100)).unwrap_err(),
            SwarmError::InvalidMovementSampleLimit
        );

        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(100)).expect("valid swarm");
        assert_eq!(
            swarm
                .set_static_obstacles(vec![CollisionBody {
                    entity_id: EntityId(900),
                    kind: CollisionBodyKind::Swarm,
                    position: WorldPosition { x_mm: 0, y_mm: 0 },
                    radius_mm: 300,
                }])
                .unwrap_err(),
            SwarmError::InvalidStaticObstacle
        );

        let mut invalid_map = load_smoke_obstacle_map_data();
        invalid_map.schema = "wrong_schema";
        assert_eq!(
            swarm
                .set_static_obstacles_from_map_data(&invalid_map)
                .unwrap_err(),
            SwarmError::InvalidMapData(MapDataValidationError::UnsupportedSchema)
        );
    }

    #[test]
    fn swarm_timer_spawns_gradually_and_caps_active_entities() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(100)).expect("valid swarm");

        let before = swarm.tick(Tick(9));
        assert!(before.spawned_entity_ids.is_empty());
        assert_eq!(before.active_count, 0);
        assert!(!before.spawn_capped);

        let first = swarm.tick(Tick(10));
        assert_eq!(first.spawned_entity_ids, vec![EntityId(100), EntityId(101)]);
        assert_eq!(first.active_count, 2);
        assert!(!first.spawn_capped);
        assert!(first.movement.is_none());

        let between = swarm.tick(Tick(11));
        assert!(between.spawned_entity_ids.is_empty());
        assert_eq!(between.active_count, 2);

        let second = swarm.tick(Tick(13));
        assert_eq!(
            second.spawned_entity_ids,
            vec![EntityId(102), EntityId(103)]
        );
        assert_eq!(second.active_count, 4);

        let capped = swarm.tick(Tick(16));
        assert_eq!(capped.spawned_entity_ids, vec![EntityId(104)]);
        assert_eq!(capped.active_count, 5);
        assert!(capped.spawn_capped);
        assert!(capped.movement.is_none());

        let after_cap = swarm.tick(Tick(19));
        assert!(after_cap.spawned_entity_ids.is_empty());
        assert_eq!(after_cap.active_count, 5);
        assert!(after_cap.spawn_capped);
        assert_eq!(swarm.spawned_total(), 5);
    }

    #[test]
    fn swarm_uses_round_robin_spawn_points_deterministically() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(200)).expect("valid swarm");

        swarm.tick(Tick(10));
        swarm.tick(Tick(13));

        let entities = swarm.active_entities().copied().collect::<Vec<_>>();
        assert_eq!(entities.len(), 4);
        assert_eq!(entities[0].entity_id, EntityId(200));
        assert_eq!(entities[0].position, spawn_points()[0].position);
        assert_eq!(entities[1].entity_id, EntityId(201));
        assert_eq!(entities[1].position, spawn_points()[1].position);
        assert_eq!(entities[2].entity_id, EntityId(202));
        assert_eq!(entities[2].position, spawn_points()[2].position);
        assert_eq!(entities[3].entity_id, EntityId(203));
        assert_eq!(entities[3].position, spawn_points()[0].position);
    }

    #[test]
    fn route_pressure_groups_active_swarm_by_route_target() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(300)).expect("valid swarm");

        swarm.tick(Tick(10));
        let report = swarm.tick(Tick(13));

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.route_pressure.len(), 2);
        assert_eq!(
            report.route_pressure[0],
            RoutePressureSample {
                target_position: WorldPosition {
                    x_mm: -10_000,
                    y_mm: 0,
                },
                active_entity_count: 1,
                radius_mm: 4_000,
                nearest_distance_sq_mm: 125_000_000,
                farthest_distance_sq_mm: 125_000_000,
            }
        );
        assert_eq!(
            report.route_pressure[1],
            RoutePressureSample {
                target_position: WorldPosition {
                    x_mm: 10_000,
                    y_mm: 0,
                },
                active_entity_count: 3,
                radius_mm: 4_000,
                nearest_distance_sq_mm: 104_000_000,
                farthest_distance_sq_mm: 121_000_000,
            }
        );
    }

    #[test]
    fn aggro_trails_split_direct_memory_and_expired_intents() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(400)).expect("valid swarm");
        swarm.tick(Tick(10));
        swarm.record_aggro_stimulus(AggroStimulus {
            source_entity_id: EntityId(900),
            position: WorldPosition {
                x_mm: 5_000,
                y_mm: 5_000,
            },
            tick: Tick(11),
            strength: 50,
        });

        let direct = swarm.tick(Tick(13));
        assert_eq!(
            direct.aggro_trails,
            vec![AggroTrailSample {
                source_entity_id: EntityId(900),
                position: WorldPosition {
                    x_mm: 5_000,
                    y_mm: 5_000,
                },
                last_seen_tick: Tick(11),
                age_ticks: 2,
                strength: 50,
                lane: AggroTrailLane::Direct,
            }]
        );

        let memory = swarm.tick(Tick(20));
        assert_eq!(memory.aggro_trails[0].lane, AggroTrailLane::Memory);
        assert_eq!(memory.aggro_trails[0].age_ticks, 9);

        let expired = swarm.tick(Tick(30));
        assert!(expired.aggro_trails.is_empty());
    }

    #[test]
    fn behavior_report_selects_aggro_intent_and_ai_lod() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(500)).expect("valid swarm");
        swarm.tick(Tick(10));
        swarm.tick(Tick(13));
        swarm.record_aggro_stimulus(AggroStimulus {
            source_entity_id: EntityId(999),
            position: WorldPosition {
                x_mm: 7_000,
                y_mm: 0,
            },
            tick: Tick(12),
            strength: 80,
        });

        let report = swarm.evaluate_behavior(Tick(13), WorldPosition { x_mm: 0, y_mm: 0 });

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.samples.len(), 4);
        assert_eq!(report.lod_counts.full, 3);
        assert_eq!(report.lod_counts.reduced, 1);
        assert_eq!(report.lod_counts.aggregate, 0);
        assert!(report
            .samples
            .iter()
            .all(|sample| sample.intent_kind == SwarmIntentKind::AggroDirect));
        assert!(report.samples.iter().all(|sample| sample.intent_target
            == (WorldPosition {
                x_mm: 7_000,
                y_mm: 0
            })));

        let memory = swarm.evaluate_behavior(Tick(20), WorldPosition { x_mm: 0, y_mm: 0 });
        assert!(memory
            .samples
            .iter()
            .all(|sample| sample.intent_kind == SwarmIntentKind::AggroMemory));

        let route = swarm.evaluate_behavior(Tick(40), WorldPosition { x_mm: 0, y_mm: 0 });
        assert!(route
            .samples
            .iter()
            .all(|sample| sample.intent_kind == SwarmIntentKind::RoutePressure));
    }

    #[test]
    fn movement_preview_applies_flow_field_candidates_to_local_physics_preview() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(550)).expect("valid swarm");
        swarm.tick(Tick(10));

        let report = swarm.movement_preview(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        );

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.flow_field_query_count, 2);
        assert_eq!(report.flow_field_build_count, 1);
        assert_eq!(report.flow_field_unreachable_count, 0);
        assert!(report.flow_field_build_count > 0);
        assert_eq!(report.applied_delta_count, 2);
        assert_eq!(report.physics_candidate_count, report.applied_delta_count);
        assert_eq!(report.physics_initial_contact_count, 0);
        assert_eq!(report.physics_final_contact_count, 0);
        assert!(report
            .samples
            .iter()
            .all(|sample| sample.flow_field_result == FlowFieldStepResult::NextCell));
        assert!(report
            .samples
            .iter()
            .all(|sample| sample.collision_decision == CollisionMovementDecision::Accepted));
        assert!(report
            .samples
            .iter()
            .all(|sample| sample.requested_delta == sample.resolved_delta));
        assert_eq!(report.claim_scope, "movement_preview_only");
    }

    #[test]
    fn movement_preview_uses_static_obstacles_as_flow_field_blockers() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(560)).expect("valid swarm");
        swarm.tick(Tick(10));
        swarm
            .set_static_obstacles(vec![static_obstacle(900, 1_000, 0, 900)])
            .expect("valid static obstacle");

        let report = swarm.movement_preview(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            1,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        );
        let sample = report.samples.first().expect("one movement sample");

        assert_eq!(swarm.static_obstacle_count(), 1);
        assert_eq!(report.sample_count, 1);
        assert_eq!(sample.flow_field_result, FlowFieldStepResult::NextCell);
        assert_eq!(sample.requested_delta.dx_mm, 0);
        assert_ne!(sample.requested_delta.dy_mm, 0);
        assert_eq!(
            sample.collision_decision,
            CollisionMovementDecision::Accepted
        );
    }

    #[test]
    fn map_data_obstacles_become_swarm_static_obstacles() {
        let map = MapDataImport {
            schema: crate::MAP_DATA_SCHEMA,
            map_id: "swarm-test-map".to_string(),
            map_version: 1,
            checksum: "sum16:swarm-test".to_string(),
            bounds_mm: MapBounds {
                min_x: -10_000,
                min_y: -10_000,
                max_x: 10_000,
                max_y: 10_000,
            },
            spawn_points: Vec::new(),
            capture_points: Vec::new(),
            obstacles: vec![MapShape {
                id: "wall_a".to_string(),
                kind: MapShapeKind::Obstacle,
                center: MapPoint {
                    x_mm: 1_000,
                    y_mm: 0,
                },
                half_extents_mm: MapPoint {
                    x_mm: 400,
                    y_mm: 700,
                },
                class_label: "blocker".to_string(),
            }],
            cover_objects: Vec::new(),
            navigation_hints: Vec::new(),
        };
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(565)).expect("valid swarm");

        assert_eq!(swarm.set_static_obstacles_from_map_data(&map), Ok(1));
        assert_eq!(swarm.static_obstacle_count(), 1);
        assert_eq!(swarm.collision_bodies().len(), 0);

        let bodies = swarm.movement_collision_bodies();
        assert_eq!(bodies.len(), 1);
        assert_eq!(
            bodies[0],
            CollisionBody {
                entity_id: EntityId(SWARM_MAP_OBSTACLE_ENTITY_ID_BASE),
                kind: CollisionBodyKind::StaticObstacle,
                position: WorldPosition {
                    x_mm: 1_000,
                    y_mm: 0,
                },
                radius_mm: 700,
            }
        );
    }

    #[test]
    fn map_data_obstacle_extents_expand_flow_field_blocker_cells() {
        let shape = MapShape {
            id: "wide_wall".to_string(),
            kind: MapShapeKind::Obstacle,
            center: MapPoint { x_mm: 0, y_mm: 0 },
            half_extents_mm: MapPoint {
                x_mm: 2_500,
                y_mm: 500,
            },
            class_label: "blocker".to_string(),
        };

        let cells = swarm_blocker_cells_for_map_shape(
            &shape,
            1_000,
            FlowFieldBounds {
                min_x: -4,
                min_y: -2,
                max_x: 4,
                max_y: 2,
            },
            0,
        );

        assert_eq!(cells.len(), 12);
        assert!(cells.contains(&SpatialCell { x: -3, y: -1 }));
        assert!(cells.contains(&SpatialCell { x: 2, y: 0 }));

        let clipped = swarm_blocker_cells_for_map_shape(
            &shape,
            1_000,
            FlowFieldBounds {
                min_x: 0,
                min_y: 0,
                max_x: 1,
                max_y: 0,
            },
            0,
        );
        assert_eq!(
            clipped,
            [SpatialCell { x: 0, y: 0 }, SpatialCell { x: 1, y: 0 }]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
    }

    #[test]
    fn map_data_obstacle_clearance_expands_flow_field_blocker_cells() {
        let shape = MapShape {
            id: "tight_blocker".to_string(),
            kind: MapShapeKind::Obstacle,
            center: MapPoint {
                x_mm: 500,
                y_mm: 500,
            },
            half_extents_mm: MapPoint {
                x_mm: 100,
                y_mm: 100,
            },
            class_label: "blocker".to_string(),
        };
        let bounds = FlowFieldBounds {
            min_x: -1,
            min_y: -1,
            max_x: 1,
            max_y: 1,
        };

        let without_clearance = swarm_blocker_cells_for_map_shape(&shape, 1_000, bounds, 0);
        let with_clearance = swarm_blocker_cells_for_map_shape(&shape, 1_000, bounds, 600);

        assert_eq!(
            without_clearance,
            [SpatialCell { x: 0, y: 0 }]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
        assert_eq!(with_clearance.len(), 9);
        assert!(with_clearance.contains(&SpatialCell { x: -1, y: -1 }));
        assert!(with_clearance.contains(&SpatialCell { x: 0, y: 0 }));
        assert!(with_clearance.contains(&SpatialCell { x: 1, y: 1 }));
    }

    #[test]
    fn swarm_flow_field_cache_evicts_deterministically_at_entry_limit() {
        let mut cache = SwarmFlowFieldCache::default();
        let bounds = FlowFieldBounds {
            min_x: 0,
            min_y: 0,
            max_x: SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT as i32 + 4,
            max_y: 0,
        };

        for target_x in 0..(SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT + 3) {
            cache.get_or_build(
                bounds,
                SpatialCell {
                    x: target_x as i32,
                    y: 0,
                },
                BTreeSet::new(),
                1_000,
            );
        }

        let stats = cache.stats();
        assert_eq!(cache.entry_count(), SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT);
        assert_eq!(
            stats.request_count,
            (SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT + 3) as u64
        );
        assert_eq!(
            stats.build_count,
            (SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT + 3) as u64
        );
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.eviction_count, 3);
        assert!(!cache.fields.contains_key(&SwarmFlowFieldCacheKey {
            bounds,
            target_cell: SpatialCell { x: 0, y: 0 },
            blocked_cells: BTreeSet::new(),
            cell_size_mm: 1_000,
        }));
    }

    #[test]
    fn movement_apply_step_mutates_swarm_positions_from_flow_field_collision_candidates() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(575)).expect("valid swarm");
        swarm.tick(Tick(10));
        let before = swarm
            .active_entities()
            .map(|entity| (entity.entity_id, entity.position))
            .collect::<BTreeMap<_, _>>();

        let report = swarm.apply_flow_field_movement_step(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
        let after = swarm
            .active_entities()
            .map(|entity| (entity.entity_id, entity.position))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.flow_field_query_count, 2);
        assert_eq!(report.flow_field_cache_request_count, 2);
        assert_eq!(report.flow_field_build_count, 1);
        assert_eq!(report.flow_field_cache_hit_count, 1);
        assert_eq!(report.flow_field_cache_eviction_count, 0);
        assert!(report.flow_field_cache_entry_count <= SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT);
        assert_eq!(report.flow_field_unreachable_count, 0);
        assert_eq!(report.applied_delta_count, 2);
        assert_eq!(report.blocked_delta_count, 0);
        assert_eq!(report.movement_probe_correction_limit_abs_mm, None);
        assert_eq!(report.movement_probe_clamped_correction_count, 0);
        assert_eq!(report.physics_candidate_count, 2);
        assert_eq!(report.physics_final_contact_count, 0);
        assert_eq!(report.physics_correction_limit_abs_mm, None);
        assert_eq!(report.physics_clamped_correction_count, 0);
        assert!(
            report.physics_synced_position_count >= report.physics_sample_synced_position_count
        );
        assert!(report.samples.iter().all(|sample| {
            before.get(&sample.entity_id).copied() == Some(sample.from_position)
                && after.get(&sample.entity_id).copied() == Some(sample.final_position)
                && sample.from_position != sample.final_position
        }));
        assert_eq!(report.claim_scope, "swarm_movement_apply_opt_in");
    }

    #[test]
    fn movement_apply_corrects_target_cell_static_obstacle_collision() {
        let mut one_spawn_config = config();
        one_spawn_config.spawn_batch_size = 1;
        one_spawn_config.max_active = 1;
        let mut swarm = SwarmState::new(
            one_spawn_config,
            vec![SwarmSpawnPoint {
                spawn_id: 1,
                position: WorldPosition { x_mm: 0, y_mm: 0 },
                route_target: WorldPosition { x_mm: 250, y_mm: 0 },
            }],
            EntityId(580),
        )
        .expect("valid swarm");
        swarm.tick(Tick(10));
        swarm
            .set_static_obstacles(vec![static_obstacle(901, 250, 0, 300)])
            .expect("valid static obstacle");

        let report = swarm.apply_flow_field_movement_step(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            1,
            250,
            1,
        );
        let sample = report.samples.first().expect("one movement sample");

        assert_eq!(
            sample.requested_delta,
            MovementDelta {
                dx_mm: 250,
                dy_mm: 0
            }
        );
        assert_eq!(
            sample.collision_decision,
            CollisionMovementDecision::Corrected
        );
        assert_ne!(sample.final_position, sample.intent_target);
        assert_eq!(
            swarm
                .active_entities()
                .find(|entity| entity.entity_id == sample.entity_id)
                .map(|entity| entity.position),
            Some(sample.final_position)
        );
        assert_eq!(report.corrected_delta_count, 1);
        assert_eq!(report.movement_probe_correction_limit_abs_mm, None);
        assert_eq!(report.movement_probe_clamped_correction_count, 0);
        assert_eq!(report.physics_candidate_count, 1);
    }

    #[test]
    fn movement_apply_correction_limit_flows_through_collision_probe() {
        let mut one_spawn_config = config();
        one_spawn_config.spawn_batch_size = 1;
        one_spawn_config.max_active = 1;
        let mut swarm = SwarmState::new(
            one_spawn_config,
            vec![SwarmSpawnPoint {
                spawn_id: 1,
                position: WorldPosition { x_mm: 0, y_mm: 0 },
                route_target: WorldPosition { x_mm: 250, y_mm: 0 },
            }],
            EntityId(590),
        )
        .expect("valid swarm");
        swarm.tick(Tick(10));
        swarm
            .set_static_obstacles(vec![static_obstacle(902, 250, 0, 300)])
            .expect("valid static obstacle");

        let report = swarm.apply_flow_field_movement_step_with_correction_limit(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            1,
            250,
            1,
            50,
        );
        let sample = report.samples.first().expect("one movement sample");

        assert_eq!(report.movement_probe_correction_limit_abs_mm, Some(50));
        assert!(report.movement_probe_clamped_correction_count > 0);
        assert_eq!(report.applied_delta_count, 0);
        assert_eq!(report.blocked_delta_count, 1);
        assert_eq!(
            sample.collision_decision,
            CollisionMovementDecision::Blocked
        );
        assert_eq!(sample.final_position, sample.from_position);
        assert_eq!(report.physics_correction_limit_abs_mm, Some(50));
        assert_eq!(report.physics_candidate_count, 0);
        assert_eq!(report.physics_clamped_correction_count, 0);
    }

    #[test]
    fn movement_batch_apply_step_mutates_swarm_positions_from_batch_flow_field_collision_candidates(
    ) {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(595)).expect("valid swarm");
        swarm.tick(Tick(10));
        let before = swarm
            .active_entities()
            .map(|entity| (entity.entity_id, entity.position))
            .collect::<BTreeMap<_, _>>();

        let report = swarm.apply_flow_field_batch_movement_step(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
        let after = swarm
            .active_entities()
            .map(|entity| (entity.entity_id, entity.position))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.flow_field_query_count, 2);
        assert_eq!(report.flow_field_cache_request_count, 2);
        assert_eq!(report.flow_field_build_count, 1);
        assert_eq!(report.flow_field_cache_hit_count, 1);
        assert_eq!(report.flow_field_cache_eviction_count, 0);
        assert_eq!(report.flow_field_unreachable_count, 0);
        assert_eq!(
            report.applied_delta_count + report.blocked_delta_count,
            report.sample_count
        );
        assert_eq!(report.applied_delta_count, 2);
        assert_eq!(report.blocked_delta_count, 0);
        assert_eq!(report.movement_probe_correction_limit_abs_mm, None);
        assert_eq!(report.movement_probe_clamped_correction_count, 0);
        assert_eq!(report.physics_candidate_count, report.applied_delta_count);
        assert!(
            report.physics_synced_position_count >= report.physics_sample_synced_position_count
        );
        assert!(report.samples.iter().all(|sample| {
            before.get(&sample.entity_id).copied() == Some(sample.from_position)
                && after.get(&sample.entity_id).copied() == Some(sample.final_position)
                && sample.from_position != sample.final_position
        }));
        assert_eq!(report.claim_scope, "swarm_batch_movement_apply_opt_in");
    }

    #[test]
    fn movement_batch_apply_correction_limit_flows_through_batch_collision_probe() {
        let mut one_spawn_config = config();
        one_spawn_config.spawn_batch_size = 1;
        one_spawn_config.max_active = 1;
        let mut swarm = SwarmState::new(
            one_spawn_config,
            vec![SwarmSpawnPoint {
                spawn_id: 1,
                position: WorldPosition { x_mm: 0, y_mm: 0 },
                route_target: WorldPosition { x_mm: 250, y_mm: 0 },
            }],
            EntityId(596),
        )
        .expect("valid swarm");
        swarm.tick(Tick(10));
        swarm
            .set_static_obstacles(vec![static_obstacle(903, 250, 0, 300)])
            .expect("valid static obstacle");

        let report = swarm.apply_flow_field_batch_movement_step_with_correction_limit(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            1,
            250,
            1,
            50,
        );
        let sample = report.samples.first().expect("one movement sample");

        assert_eq!(report.claim_scope, "swarm_batch_movement_apply_opt_in");
        assert_eq!(report.movement_probe_correction_limit_abs_mm, Some(50));
        assert!(report.movement_probe_clamped_correction_count > 0);
        assert_eq!(report.applied_delta_count, 0);
        assert_eq!(report.blocked_delta_count, 1);
        assert_eq!(
            sample.collision_decision,
            CollisionMovementDecision::Blocked
        );
        assert_eq!(sample.final_position, sample.from_position);
        assert_eq!(report.physics_correction_limit_abs_mm, Some(50));
        assert_eq!(report.physics_candidate_count, 0);
        assert_eq!(report.physics_clamped_correction_count, 0);
    }

    #[test]
    fn movement_tick_spawns_and_applies_flow_field_movement_in_one_opt_in_step() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(590)).expect("valid swarm");

        let report = swarm.tick_with_flow_field_movement(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.tick, Tick(10));
        assert_eq!(
            report.tick_report.spawned_entity_ids,
            vec![EntityId(590), EntityId(591)]
        );
        assert_eq!(report.tick_report.active_count, 2);
        assert_eq!(report.movement_report.sample_count, 2);
        assert_eq!(report.movement_report.applied_delta_count, 2);
        assert_eq!(report.movement_report.blocked_delta_count, 0);
        assert_eq!(report.movement_report.physics_candidate_count, 2);
        assert_eq!(
            report.tick_report.movement.as_ref(),
            Some(&report.movement_report)
        );
        assert_eq!(report.claim_scope, "swarm_movement_tick_opt_in");
        for sample in &report.movement_report.samples {
            assert_ne!(sample.from_position, sample.final_position);
            assert_eq!(
                swarm
                    .active_entities()
                    .find(|entity| entity.entity_id == sample.entity_id)
                    .map(|entity| entity.position),
                Some(sample.final_position)
            );
        }
    }

    #[test]
    fn batch_movement_tick_spawns_and_applies_batch_flow_field_movement_in_one_opt_in_step() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(597)).expect("valid swarm");

        let report = swarm.tick_with_batch_flow_field_movement(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );

        assert_eq!(report.schema, SWARM_SCHEMA);
        assert_eq!(report.tick, Tick(10));
        assert_eq!(
            report.tick_report.spawned_entity_ids,
            vec![EntityId(597), EntityId(598)]
        );
        assert_eq!(report.tick_report.active_count, 2);
        assert_eq!(report.movement_report.sample_count, 2);
        assert_eq!(report.movement_report.applied_delta_count, 2);
        assert_eq!(report.movement_report.blocked_delta_count, 0);
        assert_eq!(report.movement_report.physics_candidate_count, 2);
        assert_eq!(
            &report.movement_report.claim_scope,
            &"swarm_batch_movement_apply_opt_in"
        );
        assert_eq!(
            report.tick_report.movement.as_ref(),
            Some(&report.movement_report)
        );
        assert_eq!(report.claim_scope, "swarm_batch_movement_tick_opt_in");
        for sample in &report.movement_report.samples {
            assert_ne!(sample.from_position, sample.final_position);
            assert_eq!(
                swarm
                    .active_entities()
                    .find(|entity| entity.entity_id == sample.entity_id)
                    .map(|entity| entity.position),
                Some(sample.final_position)
            );
        }
    }

    #[test]
    fn configured_tick_can_apply_flow_field_collision_movement() {
        let movement_config =
            config().with_flow_field_collision_movement(2, SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM, 1);
        let mut swarm =
            SwarmState::new(movement_config, spawn_points(), EntityId(595)).expect("valid swarm");

        let report = swarm.tick_with_focus(Tick(10), WorldPosition { x_mm: 0, y_mm: 0 });
        let movement = report
            .movement
            .as_ref()
            .expect("configured tick reports movement");

        assert_eq!(
            report.spawned_entity_ids,
            vec![EntityId(595), EntityId(596)]
        );
        assert_eq!(report.active_count, 2);
        assert_eq!(movement.sample_count, 2);
        assert_eq!(movement.applied_delta_count, 2);
        assert_eq!(movement.blocked_delta_count, 0);
        assert_eq!(movement.physics_candidate_count, 2);
        assert_eq!(movement.physics_final_contact_count, 0);
        assert_eq!(movement.claim_scope, "swarm_movement_apply_opt_in");
        for sample in &movement.samples {
            assert_ne!(sample.from_position, sample.final_position);
            assert_eq!(
                swarm
                    .active_entities()
                    .find(|entity| entity.entity_id == sample.entity_id)
                    .map(|entity| entity.position),
                Some(sample.final_position)
            );
        }
    }

    #[test]
    fn configured_tick_can_apply_batch_flow_field_collision_movement() {
        let movement_config = config().with_batch_flow_field_collision_movement(
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
        let mut swarm =
            SwarmState::new(movement_config, spawn_points(), EntityId(599)).expect("valid swarm");

        let report = swarm.tick_with_focus(Tick(10), WorldPosition { x_mm: 0, y_mm: 0 });
        let movement = report
            .movement
            .as_ref()
            .expect("configured batch tick reports movement");

        assert_eq!(
            report.spawned_entity_ids,
            vec![EntityId(599), EntityId(600)]
        );
        assert_eq!(report.active_count, 2);
        assert_eq!(movement.sample_count, 2);
        assert_eq!(movement.applied_delta_count, 2);
        assert_eq!(movement.blocked_delta_count, 0);
        assert_eq!(movement.physics_candidate_count, 2);
        assert_eq!(movement.physics_final_contact_count, 0);
        assert_eq!(movement.claim_scope, "swarm_batch_movement_apply_opt_in");
        for sample in &movement.samples {
            assert_ne!(sample.from_position, sample.final_position);
            assert_eq!(
                swarm
                    .active_entities()
                    .find(|entity| entity.entity_id == sample.entity_id)
                    .map(|entity| entity.position),
                Some(sample.final_position)
            );
        }
    }

    #[test]
    fn swarm_snapshot_entities_export_authoritative_positions() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(595)).expect("valid swarm");
        swarm.tick(Tick(10));

        let entities = swarm.snapshot_entities();
        let snapshot = swarm.build_full_snapshot(7, Tick(10));

        assert_eq!(entities.len(), 2);
        assert_eq!(snapshot.snapshot_id, 7);
        assert_eq!(snapshot.baseline_snapshot_id, 0);
        assert_eq!(snapshot.tick, Tick(10));
        assert_eq!(snapshot.entities, entities);
        assert!(snapshot.removed_entities.is_empty());
        assert!(snapshot.entities.iter().all(|entity| {
            entity.entity_kind == SWARM_ENTITY_KIND
                && entity.faction_id == 0
                && entity.flags == 1
                && entity.health_q8 == 256
        }));
    }

    #[test]
    fn movement_tick_snapshot_contains_moved_swarm_positions() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(596)).expect("valid swarm");

        let report = swarm.tick_with_flow_field_movement(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
        let snapshot = swarm.build_full_snapshot(8, Tick(10));

        assert_eq!(snapshot.entities.len(), report.movement_report.sample_count);
        for sample in &report.movement_report.samples {
            assert!(snapshot.entities.iter().any(|entity| {
                entity.entity_id == sample.entity_id
                    && entity.position == sample.final_position
                    && entity.entity_kind == SWARM_ENTITY_KIND
            }));
        }
        assert!(estimate_snapshot_bytes(&snapshot) > SNAPSHOT_HEADER_BYTES);
    }

    #[test]
    fn movement_tick_interest_delta_snapshot_contains_moved_visible_swarm_entities() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(597)).expect("valid swarm");

        let movement = swarm.tick_with_flow_field_movement(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
        let delta = swarm.build_interest_delta_snapshot(
            PlayerSessionId(9),
            AoiRegion::new(SpatialCell { x: 0, y: 0 }, 16),
            9,
            8,
            Tick(10),
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        );

        assert_eq!(delta.schema, SWARM_SCHEMA);
        assert_eq!(delta.tick, Tick(10));
        assert_eq!(delta.snapshot.snapshot_id, 9);
        assert_eq!(delta.snapshot.baseline_snapshot_id, 8);
        assert_eq!(delta.visible_entity_count, 2);
        assert_eq!(delta.removed_entity_count, 0);
        assert!(delta.aggregate_far_state_count <= 1);
        assert_eq!(
            delta.visible_entity_ids,
            movement
                .movement_report
                .samples
                .iter()
                .map(|sample| sample.entity_id)
                .collect::<BTreeSet<_>>()
        );
        for sample in &movement.movement_report.samples {
            assert!(delta.snapshot.entities.iter().any(|entity| {
                entity.entity_id == sample.entity_id
                    && entity.position == sample.final_position
                    && entity.entity_kind == SWARM_ENTITY_KIND
            }));
        }
        assert_eq!(delta.claim_scope, "swarm_movement_delta_snapshot_local");
    }

    #[test]
    fn batch_movement_tick_snapshots_contain_moved_swarm_positions() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(598)).expect("valid swarm");

        let movement = swarm.tick_with_batch_flow_field_movement(
            Tick(10),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
            1,
        );
        let snapshot = swarm.build_full_snapshot(10, Tick(10));
        let delta = swarm.build_interest_delta_snapshot(
            PlayerSessionId(10),
            AoiRegion::new(SpatialCell { x: 0, y: 0 }, 16),
            11,
            10,
            Tick(10),
            SWARM_MOVEMENT_PREVIEW_CELL_SIZE_MM,
        );

        assert_eq!(
            movement.movement_report.claim_scope,
            "swarm_batch_movement_apply_opt_in"
        );
        assert_eq!(
            snapshot.entities.len(),
            movement.movement_report.sample_count
        );
        assert_eq!(
            delta.visible_entity_count,
            movement.movement_report.sample_count
        );
        for sample in &movement.movement_report.samples {
            assert!(snapshot.entities.iter().any(|entity| {
                entity.entity_id == sample.entity_id
                    && entity.position == sample.final_position
                    && entity.entity_kind == SWARM_ENTITY_KIND
            }));
            assert!(delta.snapshot.entities.iter().any(|entity| {
                entity.entity_id == sample.entity_id
                    && entity.position == sample.final_position
                    && entity.entity_kind == SWARM_ENTITY_KIND
            }));
        }
        assert!(estimate_snapshot_bytes(&snapshot) > SNAPSHOT_HEADER_BYTES);
        assert_eq!(delta.claim_scope, "swarm_movement_delta_snapshot_local");
    }

    #[test]
    fn swarm_batch_movement_replication_smoke_records_changed_delta_positions() {
        let run = run_swarm_batch_movement_replication_smoke();

        assert_eq!(run.tick, Tick(1_500));
        assert_eq!(run.active_count, 1_000);
        assert_eq!(run.baseline_snapshot_entity_count, run.active_count);
        assert_eq!(run.movement_snapshot_entity_count, run.active_count);
        assert_eq!(run.delta_visible_entity_count, run.active_count);
        assert!(run.movement_sample_count > 0);
        assert!(run.movement_applied_delta_count > 0);
        assert!(run.movement_physics_iterations_run > 0);
        assert!(run.delta_changed_visible_entity_count > 0);
        assert_eq!(run.delta_removed_entity_count, 0);
        assert_eq!(run.delta_aggregate_far_state_count, 0);
        assert!(run.delta_snapshot_bytes > SNAPSHOT_HEADER_BYTES);
        assert!(run.delta_bandwidth_kb_s_per_client > 0.0);
        assert!(run.aggregate_visible_entity_count > 0);
        assert!(run.aggregate_visible_entity_count < run.active_count);
        assert!(run.aggregate_changed_visible_entity_count > 0);
        assert!(run.aggregate_far_state_count > 0);
        assert!(run.aggregate_snapshot_bytes > SNAPSHOT_HEADER_BYTES);
        assert_eq!(run.budget_result, BudgetResult::Blocked);
        assert_eq!(
            run.claim_scope,
            "local_batch_movement_replication_smoke_only"
        );
    }

    #[test]
    fn swarm_exports_collision_bodies_for_future_collision_pipeline() {
        let mut swarm =
            SwarmState::new(config(), spawn_points(), EntityId(600)).expect("valid swarm");

        swarm.tick(Tick(10));
        let bodies = swarm.collision_bodies();

        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[0].entity_id, EntityId(600));
        assert_eq!(bodies[0].kind, CollisionBodyKind::Swarm);
        assert_eq!(bodies[0].radius_mm, config().collision_radius_mm);
        assert_eq!(bodies[1].entity_id, EntityId(601));
    }

    #[test]
    fn swarm_load_smoke_records_one_thousand_zombie_pressure() {
        let run = run_swarm_load_smoke();

        assert_eq!(run.tick_count, 250);
        assert_eq!(run.active_count, 1_000);
        assert_eq!(run.spawned_total, 1_000);
        assert!(run.local_smoke_total_elapsed_us > 0);
        assert!(run.spawn_ticks_elapsed_us > 0);
        assert!(run.behavior_elapsed_us > 0);
        assert!(run.movement_preview_elapsed_us > 0);
        assert!(run.movement_tick_elapsed_us > 0);
        assert!(run.batch_movement_tick_elapsed_us > 0);
        assert!(run.configured_movement_tick_elapsed_us > 0);
        assert!(run.configured_batch_movement_tick_elapsed_us > 0);
        assert!(run.configured_movement_loop_elapsed_us > 0);
        assert!(run.static_obstacle_movement_elapsed_us > 0);
        assert!(run.snapshot_elapsed_us > 0);
        assert!(run.collision_diagnostics_elapsed_us > 0);
        assert!(run.local_smoke_total_elapsed_us >= run.spawn_ticks_elapsed_us);
        assert!(run.local_smoke_total_elapsed_us >= run.movement_tick_elapsed_us);
        assert!(run.local_smoke_total_elapsed_us >= run.batch_movement_tick_elapsed_us);
        assert!(run.local_smoke_total_elapsed_us >= run.collision_diagnostics_elapsed_us);
        assert_eq!(run.behavior_sample_count, 1_000);
        assert_eq!(run.collision_body_count, 1_000);
        assert!(run.collision_contact_count > 0);
        assert_eq!(run.collision_admission_check_count, 16);
        assert!(run.collision_admission_rejected_count > 0);
        assert_eq!(
            run.collision_resolved_admission_check_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert!(run.collision_resolved_admission_rejected_count > 0);
        assert!(run.collision_resolved_admission_iterations_run_count > 0);
        assert!(run.collision_resolved_admission_correction_count > 0);
        assert_eq!(
            run.collision_movement_probe_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert!(run.collision_movement_probe_blocked_count > 0);
        assert_eq!(
            run.collision_movement_probe_blocked_count,
            run.movement_preview_blocked_delta_count
        );
        assert_eq!(
            run.collision_batch_movement_probe_count,
            run.movement_preview_sample_count
        );
        assert_eq!(run.collision_batch_movement_probe_unknown_body_count, 0);
        assert_eq!(
            run.collision_batch_movement_probe_accepted_count
                + run.collision_batch_movement_probe_corrected_count
                + run.collision_batch_movement_probe_blocked_count,
            run.collision_batch_movement_probe_count
        );
        assert!(run.collision_batch_movement_probe_iterations_run_count > 0);
        assert!(run.collision_batch_movement_probe_correction_count > 0);
        assert!(run.collision_batch_movement_probe_correction_abs_mm_total > 0);
        assert!(run.collision_batch_movement_probe_max_correction_abs_mm > 0);
        assert_eq!(
            run.movement_preview_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert!(run.movement_preview_flow_field_build_count > 0);
        assert!(
            run.movement_preview_flow_field_build_count <= SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(
            run.movement_preview_flow_field_query_count,
            run.movement_preview_sample_count
        );
        assert_eq!(run.movement_preview_flow_field_unreachable_count, 0);
        assert_eq!(
            run.movement_preview_physics_candidate_count,
            run.movement_preview_applied_delta_count
        );
        assert!(run.movement_preview_physics_candidate_count <= run.movement_preview_sample_count);
        assert!(run.movement_preview_physics_initial_contact_count > 0);
        assert!(run.movement_preview_physics_iterations_run > 0);
        assert!(run.movement_preview_physics_applied_correction_count > 0);
        assert!(run.movement_preview_physics_applied_correction_abs_mm_total > 0);
        assert!(run.movement_preview_physics_max_applied_correction_abs_mm > 0);
        assert!(
            run.movement_preview_physics_final_contact_count
                <= run.movement_preview_physics_initial_contact_count
        );
        assert_eq!(
            run.movement_apply_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert!(run.movement_apply_flow_field_build_count > 0);
        assert_eq!(
            run.movement_apply_flow_field_cache_request_count,
            run.movement_apply_sample_count as u64
        );
        assert!(run.movement_apply_flow_field_cache_hit_count > 0);
        assert_eq!(run.movement_apply_flow_field_cache_eviction_count, 0);
        assert!(
            run.movement_apply_flow_field_cache_entry_count <= SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT
        );
        assert_eq!(
            run.movement_apply_flow_field_query_count,
            run.movement_apply_sample_count
        );
        assert_eq!(run.movement_apply_flow_field_unreachable_count, 0);
        assert_eq!(
            run.movement_apply_applied_delta_count + run.movement_apply_blocked_delta_count,
            run.movement_apply_sample_count
        );
        assert_eq!(
            run.movement_apply_physics_candidate_count,
            run.movement_apply_applied_delta_count
        );
        assert!(run.movement_apply_physics_candidate_count <= run.movement_apply_sample_count);
        assert!(run.movement_apply_physics_initial_contact_count > 0);
        assert!(run.movement_apply_physics_iterations_run > 0);
        assert!(run.movement_apply_physics_applied_correction_count > 0);
        assert!(run.movement_apply_physics_applied_correction_abs_mm_total > 0);
        assert!(run.movement_apply_physics_max_applied_correction_abs_mm > 0);
        assert_eq!(
            run.movement_apply_movement_probe_correction_limit_abs_mm,
            None
        );
        assert_eq!(
            run.movement_apply_movement_probe_clamped_correction_count,
            0
        );
        assert_eq!(run.movement_apply_physics_correction_limit_abs_mm, None);
        assert_eq!(run.movement_apply_physics_clamped_correction_count, 0);
        assert!(
            run.movement_apply_physics_final_contact_count
                <= run.movement_apply_physics_initial_contact_count
        );
        assert!(run.movement_apply_physics_synced_position_count > 0);
        assert!(run.movement_apply_physics_sample_synced_position_count > 0);
        assert!(
            run.movement_apply_physics_sample_synced_position_count
                <= run.movement_apply_physics_synced_position_count
        );
        assert_eq!(
            run.clamped_movement_apply_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(
            run.clamped_movement_apply_movement_probe_correction_limit_abs_mm,
            Some(SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM)
        );
        assert!(run.clamped_movement_apply_movement_probe_clamped_correction_count > 0);
        assert_eq!(
            run.clamped_movement_apply_physics_correction_limit_abs_mm,
            Some(SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM)
        );
        assert!(
            run.clamped_movement_apply_physics_max_applied_correction_abs_mm
                <= SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM
        );
        assert_eq!(
            run.batch_movement_apply_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(
            run.batch_movement_apply_flow_field_cache_request_count,
            run.batch_movement_apply_sample_count as u64
        );
        assert!(run.batch_movement_apply_flow_field_cache_hit_count > 0);
        assert_eq!(run.batch_movement_apply_flow_field_cache_eviction_count, 0);
        assert!(
            run.batch_movement_apply_flow_field_cache_entry_count
                <= SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT
        );
        assert_eq!(
            run.batch_movement_apply_applied_delta_count
                + run.batch_movement_apply_blocked_delta_count,
            run.batch_movement_apply_sample_count
        );
        assert_eq!(
            run.batch_movement_apply_physics_candidate_count,
            run.batch_movement_apply_applied_delta_count
        );
        assert!(run.batch_movement_apply_applied_delta_count > 0);
        assert_eq!(
            run.batch_movement_apply_movement_probe_correction_limit_abs_mm,
            None
        );
        assert_eq!(
            run.batch_movement_apply_movement_probe_clamped_correction_count,
            0
        );
        assert!(run.batch_movement_apply_physics_iterations_run > 0);
        assert!(run.batch_movement_apply_physics_synced_position_count > 0);
        assert!(run.batch_movement_apply_physics_sample_synced_position_count > 0);
        assert!(
            run.batch_movement_apply_physics_sample_synced_position_count
                <= run.batch_movement_apply_physics_synced_position_count
        );
        assert_eq!(
            run.batch_movement_tick_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(run.batch_movement_tick_active_count, 1_000);
        assert_eq!(run.batch_movement_tick_spawned_count, 0);
        assert_eq!(
            run.batch_movement_tick_applied_delta_count,
            run.batch_movement_apply_applied_delta_count
        );
        assert_eq!(
            run.batch_movement_tick_physics_iterations_run,
            run.batch_movement_apply_physics_iterations_run
        );
        assert_eq!(
            run.movement_tick_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(run.movement_tick_active_count, 1_000);
        assert_eq!(run.movement_tick_spawned_count, 0);
        assert_eq!(
            run.movement_tick_applied_delta_count,
            run.movement_apply_applied_delta_count
        );
        assert_eq!(
            run.movement_tick_physics_iterations_run,
            run.movement_apply_physics_iterations_run
        );
        assert_eq!(
            run.configured_movement_tick_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(run.configured_movement_tick_active_count, 1_000);
        assert_eq!(run.configured_movement_tick_spawned_count, 0);
        assert_eq!(
            run.configured_movement_tick_applied_delta_count,
            run.movement_apply_applied_delta_count
        );
        assert_eq!(
            run.configured_movement_tick_physics_iterations_run,
            run.movement_apply_physics_iterations_run
        );
        assert_eq!(
            run.configured_batch_movement_tick_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(run.configured_batch_movement_tick_active_count, 1_000);
        assert_eq!(run.configured_batch_movement_tick_spawned_count, 0);
        assert_eq!(
            run.configured_batch_movement_tick_applied_delta_count,
            run.batch_movement_apply_applied_delta_count
        );
        assert_eq!(
            run.configured_batch_movement_tick_physics_iterations_run,
            run.batch_movement_apply_physics_iterations_run
        );
        assert_eq!(
            run.configured_batch_movement_tick_claim_scope,
            "swarm_batch_movement_apply_opt_in"
        );
        assert_eq!(
            run.configured_clamped_movement_tick_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert_eq!(
            run.configured_clamped_movement_tick_movement_probe_correction_limit_abs_mm,
            Some(SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM)
        );
        assert!(run.configured_clamped_movement_tick_movement_probe_clamped_correction_count > 0);
        assert_eq!(
            run.configured_clamped_movement_tick_physics_correction_limit_abs_mm,
            Some(SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM)
        );
        assert!(
            run.configured_clamped_movement_tick_physics_max_applied_correction_abs_mm
                <= SWARM_MOVEMENT_APPLY_CLAMP_LIMIT_ABS_MM
        );
        assert_eq!(
            run.configured_movement_loop_tick_count,
            SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT
        );
        assert_eq!(run.configured_movement_loop_active_count, 1_000);
        assert_eq!(run.configured_movement_loop_spawned_count, 0);
        assert_eq!(
            run.configured_movement_loop_sample_count,
            SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT
                * SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT as usize
        );
        assert!(run.configured_movement_loop_applied_delta_count > 0);
        assert!(run.configured_movement_loop_physics_iterations_run > 0);
        assert_eq!(
            run.configured_movement_loop_flow_field_cache_request_count,
            run.configured_movement_loop_sample_count as u64
        );
        assert!(run.configured_movement_loop_flow_field_cache_hit_count > 0);
        assert_eq!(
            run.configured_movement_loop_flow_field_cache_eviction_count,
            0
        );
        assert!(
            run.configured_movement_loop_flow_field_cache_entry_count
                <= SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT
        );
        assert!(run.configured_movement_loop_moved_entity_count > 0);
        assert_eq!(run.static_obstacle_count, 4);
        assert_eq!(run.static_obstacle_source, "map_data_import");
        assert_eq!(
            run.static_obstacle_map_obstacle_count,
            run.static_obstacle_count
        );
        assert_eq!(
            run.static_obstacle_clearance_mm,
            SwarmConfig::local_scale_smoke().collision_radius_mm
        );
        assert!(run.static_obstacle_blocker_cell_count > run.static_obstacle_count);
        assert_eq!(
            run.static_obstacle_movement_sample_count,
            SWARM_RESOLVED_ADMISSION_SAMPLE_COUNT
        );
        assert!(run.static_obstacle_movement_flow_field_build_count > 0);
        assert!(run.static_obstacle_movement_applied_delta_count > 0);
        assert!(
            run.static_obstacle_movement_applied_delta_count
                + run.static_obstacle_movement_blocked_delta_count
                <= run.static_obstacle_movement_sample_count
        );
        assert!(run.static_obstacle_movement_physics_iterations_run > 0);
        assert_eq!(run.movement_tick_snapshot_entity_count, 1_000);
        assert_eq!(
            run.movement_tick_snapshot_bytes,
            SNAPSHOT_HEADER_BYTES + 1_000 * SNAPSHOT_ENTITY_BYTES
        );
        assert!(run.movement_tick_snapshot_bandwidth_kb_s_per_client > 0.0);
        assert_eq!(run.movement_delta_snapshot_entity_count, 1_000);
        assert_eq!(run.movement_delta_snapshot_removed_count, 0);
        assert_eq!(run.movement_delta_snapshot_aggregate_far_state_count, 0);
        assert_eq!(
            run.movement_delta_snapshot_bytes,
            SNAPSHOT_HEADER_BYTES + 1_000 * SNAPSHOT_ENTITY_BYTES
        );
        assert!(run.movement_delta_snapshot_bandwidth_kb_s_per_client > 0.0);
        assert!(run.movement_aggregate_delta_snapshot_entity_count > 0);
        assert!(run.movement_aggregate_delta_snapshot_entity_count < run.active_count);
        assert_eq!(run.movement_aggregate_delta_snapshot_removed_count, 0);
        assert!(run.movement_aggregate_delta_snapshot_aggregate_far_state_count > 0);
        assert!(run.movement_aggregate_delta_snapshot_bytes > SNAPSHOT_HEADER_BYTES);
        assert!(run.movement_aggregate_delta_snapshot_bytes < run.movement_delta_snapshot_bytes);
        assert!(run.movement_aggregate_delta_snapshot_bandwidth_kb_s_per_client > 0.0);
        assert_eq!(run.batch_movement_tick_snapshot_entity_count, 1_000);
        assert_eq!(
            run.batch_movement_tick_snapshot_bytes,
            SNAPSHOT_HEADER_BYTES + 1_000 * SNAPSHOT_ENTITY_BYTES
        );
        assert_eq!(run.batch_movement_delta_snapshot_entity_count, 1_000);
        assert_eq!(
            run.batch_movement_delta_snapshot_bytes,
            SNAPSHOT_HEADER_BYTES + 1_000 * SNAPSHOT_ENTITY_BYTES
        );
        assert_eq!(
            run.collision_resolution_contact_count,
            run.collision_contact_count
        );
        assert!(run.collision_resolution_correction_count > 0);
        assert!(run.collision_physics_iterations_run > 0);
        assert!(run.collision_physics_applied_correction_count > 0);
        assert!(run.collision_physics_applied_correction_abs_mm_total > 0);
        assert!(run.collision_physics_max_applied_correction_abs_mm > 0);
        assert!(run.collision_physics_final_contact_count <= run.collision_contact_count);
        assert_eq!(run.route_pressure_bucket_count, 2);
        assert_eq!(run.aggro_trail_count, 2);
        assert_eq!(run.direct_aggro_count, 1);
        assert_eq!(run.memory_aggro_count, 1);
        assert!(run.ai_lod_counts.full > 0);
        assert!(run.ai_lod_counts.reduced > 0);
        assert!(run.ai_lod_counts.aggregate > 0);
        assert_eq!(
            run.ai_lod_counts.full + run.ai_lod_counts.reduced + run.ai_lod_counts.aggregate,
            1_000
        );
        assert!(run.estimated_snapshot_bytes > 0);
        assert!(run.estimated_bandwidth_kb_s_per_client > 0.0);
        assert_eq!(run.budget_result, BudgetResult::Blocked);
        assert_eq!(run.claim_scope, "informational_contract_only");
    }

    #[test]
    fn swarm_configured_movement_loop_measurement_records_percentiles() {
        let run = run_swarm_configured_movement_loop_measurement(
            SWARM_CONFIGURED_MOVEMENT_MEASUREMENT_SAMPLE_COUNT,
        );

        assert_eq!(
            run.sample_count,
            SWARM_CONFIGURED_MOVEMENT_MEASUREMENT_SAMPLE_COUNT
        );
        assert_eq!(
            run.tick_count_per_sample,
            SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT
        );
        assert_eq!(
            run.movement_sample_limit,
            SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT
        );
        assert_eq!(run.active_count, 1_000);
        assert_eq!(run.spawned_count_total, 0);
        assert!(run.elapsed_us_min > 0);
        assert!(run.elapsed_us_min <= run.elapsed_us_p50);
        assert!(run.elapsed_us_p50 <= run.elapsed_us_p95);
        assert!(run.elapsed_us_p95 <= run.elapsed_us_p99);
        assert!(run.elapsed_us_p99 <= run.elapsed_us_max);
        assert_eq!(
            run.movement_sample_count_total,
            run.sample_count
                * SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT
                * SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT as usize
        );
        assert!(run.applied_delta_count_total > 0);
        assert!(run.physics_iterations_run_total > 0);
        assert_eq!(
            run.flow_field_cache_request_count_total,
            run.movement_sample_count_total as u64
        );
        assert!(run.flow_field_cache_hit_count_total > 0);
        assert_eq!(run.flow_field_cache_eviction_count_total, 0);
        assert!(run.flow_field_cache_entry_count_max <= SWARM_FLOW_FIELD_CACHE_ENTRY_LIMIT);
        assert!(run.moved_entity_count_min > 0);
        assert_eq!(run.budget_result, BudgetResult::Blocked);
        assert_eq!(run.claim_scope, "local_measured_harness_only");
        println!(
            "swarm_configured_movement_loop_measurement samples={} active={} p50_us={} p95_us={} p99_us={} budget_result=blocked",
            run.sample_count,
            run.active_count,
            run.elapsed_us_p50,
            run.elapsed_us_p95,
            run.elapsed_us_p99
        );
    }

    #[test]
    fn swarm_batch_vs_single_movement_loop_measurement_records_comparison() {
        let run = run_swarm_batch_vs_single_movement_loop_measurement(
            SWARM_CONFIGURED_MOVEMENT_MEASUREMENT_SAMPLE_COUNT,
        );

        assert_eq!(
            run.sample_count,
            SWARM_CONFIGURED_MOVEMENT_MEASUREMENT_SAMPLE_COUNT
        );
        assert_eq!(
            run.tick_count_per_sample,
            SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT
        );
        assert_eq!(
            run.movement_sample_limit,
            SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT
        );
        assert_eq!(run.active_count, 1_000);
        assert!(run.single_elapsed_us_p50 > 0);
        assert!(run.single_elapsed_us_p50 <= run.single_elapsed_us_p95);
        assert!(run.single_elapsed_us_p95 <= run.single_elapsed_us_p99);
        assert!(run.batch_elapsed_us_p50 > 0);
        assert!(run.batch_elapsed_us_p50 <= run.batch_elapsed_us_p95);
        assert!(run.batch_elapsed_us_p95 <= run.batch_elapsed_us_p99);
        assert!(run.batch_to_single_elapsed_p95_bps > 0);
        assert_eq!(
            run.single_movement_sample_count_total,
            run.sample_count
                * SWARM_CONFIGURED_MOVEMENT_LOOP_SAMPLE_COUNT
                * SWARM_CONFIGURED_MOVEMENT_LOOP_TICK_COUNT as usize
        );
        assert_eq!(
            run.batch_movement_sample_count_total,
            run.single_movement_sample_count_total
        );
        assert!(run.single_applied_delta_count_total > 0);
        assert!(run.batch_applied_delta_count_total > 0);
        assert!(run.single_physics_iterations_run_total > 0);
        assert!(run.batch_physics_iterations_run_total > 0);
        assert!(run.single_flow_field_cache_hit_count_total > 0);
        assert!(run.batch_flow_field_cache_hit_count_total > 0);
        assert_eq!(run.single_flow_field_cache_eviction_count_total, 0);
        assert_eq!(run.batch_flow_field_cache_eviction_count_total, 0);
        assert!(run.single_moved_entity_count_min > 0);
        assert!(run.batch_moved_entity_count_min > 0);
        assert_eq!(run.budget_result, BudgetResult::Blocked);
        assert_eq!(
            run.claim_scope,
            "local_batch_vs_single_measured_harness_only"
        );
        println!(
            "swarm_batch_vs_single_movement_loop_measurement samples={} active={} single_p95_us={} batch_p95_us={} batch_to_single_p95_bps={} budget_result=blocked",
            run.sample_count,
            run.active_count,
            run.single_elapsed_us_p95,
            run.batch_elapsed_us_p95,
            run.batch_to_single_elapsed_p95_bps
        );
    }

    #[test]
    fn local_scale_smoke_config_is_bounded_for_one_thousand_swarm() {
        let config = SwarmConfig::local_scale_smoke();
        let mut swarm = SwarmState::new(
            config,
            vec![SwarmSpawnPoint {
                spawn_id: 1,
                position: WorldPosition { x_mm: 0, y_mm: 0 },
                route_target: WorldPosition {
                    x_mm: 100_000,
                    y_mm: 0,
                },
            }],
            EntityId(1),
        )
        .expect("valid smoke swarm");

        for tick in 0..250 {
            swarm.tick(Tick(tick));
        }

        assert_eq!(swarm.active_count(), 1_000);
        assert_eq!(swarm.spawned_total(), 1_000);
    }
}
