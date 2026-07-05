use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::{EntityId, MovementDelta, SpatialCell, WorldPosition};

pub const COLLISION_SCHEMA: &str = "millions_collision_prep_v0";
const COLLISION_STABILITY_CLAMP_LIMIT_ABS_MM: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollisionBodyKind {
    Swarm,
    Unit,
    StaticObstacle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionBody {
    pub entity_id: EntityId,
    pub kind: CollisionBodyKind,
    pub position: WorldPosition,
    pub radius_mm: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionContact {
    pub a: EntityId,
    pub b: EntityId,
    pub distance_sq_mm: u64,
    pub min_distance_mm: i32,
    pub overlap_mm: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionProbe {
    pub position: WorldPosition,
    pub radius_mm: i32,
    pub overlapping_entity_ids: Vec<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionAdmissionResult {
    Accepted,
    RejectedOverlap,
    UnknownBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionResolvedAdmissionResult {
    Accepted,
    AcceptedAfterResolution,
    RejectedStillOverlapping,
    UnknownBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionAdmission {
    pub entity_id: EntityId,
    pub from: Option<WorldPosition>,
    pub to: WorldPosition,
    pub radius_mm: i32,
    pub result: CollisionAdmissionResult,
    pub blocking_entity_ids: Vec<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionResolvedAdmission {
    pub entity_id: EntityId,
    pub requested_position: WorldPosition,
    pub resolved_position: Option<WorldPosition>,
    pub initial_result: CollisionAdmissionResult,
    pub initial_blocking_entity_ids: Vec<EntityId>,
    pub result: CollisionResolvedAdmissionResult,
    pub blocking_entity_ids: Vec<EntityId>,
    pub iterations_requested: usize,
    pub iterations_run: usize,
    pub applied_correction_count: usize,
    pub applied_correction_abs_mm_total: u64,
    pub max_applied_correction_abs_mm: u32,
    pub correction_limit_abs_mm: Option<u32>,
    pub clamped_correction_count: usize,
    pub final_contact_count: usize,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionMovementDecision {
    Accepted,
    Corrected,
    Blocked,
    UnknownBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionSlideAxis {
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionSlideAttempt {
    pub axis: CollisionSlideAxis,
    pub probe: CollisionMovementProbe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionMovementSlideProbe {
    pub entity_id: EntityId,
    pub requested_probe: CollisionMovementProbe,
    pub decision: CollisionMovementDecision,
    pub selected_axis: Option<CollisionSlideAxis>,
    pub selected_position: Option<WorldPosition>,
    pub selected_delta: Option<MovementDelta>,
    pub attempt_count: usize,
    pub attempts: Vec<CollisionSlideAttempt>,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionMovementProbe {
    pub entity_id: EntityId,
    pub from: Option<WorldPosition>,
    pub requested_position: WorldPosition,
    pub resolved_position: Option<WorldPosition>,
    pub requested_delta: Option<MovementDelta>,
    pub resolved_delta: Option<MovementDelta>,
    pub initial_result: CollisionAdmissionResult,
    pub resolved_result: CollisionResolvedAdmissionResult,
    pub decision: CollisionMovementDecision,
    pub initial_blocking_entity_ids: Vec<EntityId>,
    pub blocking_entity_ids: Vec<EntityId>,
    pub iterations_requested: usize,
    pub iterations_run: usize,
    pub applied_correction_count: usize,
    pub applied_correction_abs_mm_total: u64,
    pub max_applied_correction_abs_mm: u32,
    pub correction_limit_abs_mm: Option<u32>,
    pub clamped_correction_count: usize,
    pub final_contact_count: usize,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionBatchMovementCandidate {
    pub entity_id: EntityId,
    pub target_position: WorldPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionBatchMovementSample {
    pub entity_id: EntityId,
    pub from: Option<WorldPosition>,
    pub requested_position: WorldPosition,
    pub resolved_position: Option<WorldPosition>,
    pub requested_delta: Option<MovementDelta>,
    pub resolved_delta: Option<MovementDelta>,
    pub initial_result: CollisionAdmissionResult,
    pub resolved_result: CollisionResolvedAdmissionResult,
    pub decision: CollisionMovementDecision,
    pub initial_blocking_entity_ids: Vec<EntityId>,
    pub blocking_entity_ids: Vec<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionBatchMovementProbeReport {
    pub candidate_count: usize,
    pub accepted_count: usize,
    pub corrected_count: usize,
    pub blocked_count: usize,
    pub unknown_body_count: usize,
    pub initial_rejected_count: usize,
    pub iterations_requested: usize,
    pub iterations_run: usize,
    pub applied_correction_count: usize,
    pub applied_correction_abs_mm_total: u64,
    pub max_applied_correction_abs_mm: u32,
    pub correction_limit_abs_mm: Option<u32>,
    pub clamped_correction_count: usize,
    pub final_contact_count: usize,
    pub samples: Vec<CollisionBatchMovementSample>,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionCorrection {
    pub entity_id: EntityId,
    pub dx_mm: i32,
    pub dy_mm: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionResolutionPlan {
    pub contact_count: usize,
    pub corrections: Vec<CollisionCorrection>,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionPhysicsStep {
    pub initial_contact_count: usize,
    pub iterations_requested: usize,
    pub iterations_run: usize,
    pub applied_correction_count: usize,
    pub applied_correction_abs_mm_total: u64,
    pub max_applied_correction_abs_mm: u32,
    pub correction_limit_abs_mm: Option<u32>,
    pub clamped_correction_count: usize,
    pub final_contact_count: usize,
    pub resolved: bool,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionPerfScenarioId {
    ClusteredSwarm1k,
    ClusteredSwarm5k,
}

impl CollisionPerfScenarioId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClusteredSwarm1k => "collision_clustered_swarm_1k",
            Self::ClusteredSwarm5k => "collision_clustered_swarm_5k",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionPerfScenario {
    pub id: CollisionPerfScenarioId,
    pub body_count: usize,
    pub cluster_count: usize,
    pub cell_size_mm: i32,
    pub radius_mm: i32,
    pub max_iterations: usize,
}

pub const COLLISION_PERF_SCENARIOS: [CollisionPerfScenario; 2] = [
    CollisionPerfScenario {
        id: CollisionPerfScenarioId::ClusteredSwarm1k,
        body_count: 1_000,
        cluster_count: 50,
        cell_size_mm: 2_000,
        radius_mm: 350,
        max_iterations: 2,
    },
    CollisionPerfScenario {
        id: CollisionPerfScenarioId::ClusteredSwarm5k,
        body_count: 5_000,
        cluster_count: 250,
        cell_size_mm: 2_000,
        radius_mm: 350,
        max_iterations: 2,
    },
];

pub fn collision_perf_scenarios() -> &'static [CollisionPerfScenario] {
    &COLLISION_PERF_SCENARIOS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionPerfRun {
    pub scenario_id: CollisionPerfScenarioId,
    pub body_count: usize,
    pub cluster_count: usize,
    pub resolved_admission_check_count: usize,
    pub resolved_admission_accepted_after_resolution_count: usize,
    pub resolved_admission_rejected_count: usize,
    pub static_obstacle_resolution_check_count: usize,
    pub static_obstacle_correction_count: usize,
    pub clamped_resolution_check_count: usize,
    pub clamped_correction_limit_abs_mm: u32,
    pub clamped_correction_count: usize,
    pub clamped_max_applied_correction_abs_mm: u32,
    pub initial_contact_count: usize,
    pub iterations_requested: usize,
    pub iterations_run: usize,
    pub applied_correction_count: usize,
    pub applied_correction_abs_mm_total: u64,
    pub max_applied_correction_abs_mm: u32,
    pub final_contact_count: usize,
    pub elapsed_us: u64,
    pub budget_result: &'static str,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AppliedCollisionCorrections {
    applied_correction_count: usize,
    applied_correction_abs_mm_total: u64,
    max_applied_correction_abs_mm: u32,
    clamped_correction_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionError {
    InvalidCellSize,
    InvalidRadius,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionWorld {
    cell_size_mm: i32,
    bodies: BTreeMap<EntityId, CollisionBody>,
    cells: BTreeMap<SpatialCell, Vec<EntityId>>,
}

impl CollisionWorld {
    pub fn new(cell_size_mm: i32) -> Result<Self, CollisionError> {
        if cell_size_mm <= 0 {
            return Err(CollisionError::InvalidCellSize);
        }

        Ok(Self {
            cell_size_mm,
            bodies: BTreeMap::new(),
            cells: BTreeMap::new(),
        })
    }

    pub fn with_bodies(
        cell_size_mm: i32,
        bodies: impl IntoIterator<Item = CollisionBody>,
    ) -> Result<Self, CollisionError> {
        let mut world = Self::new(cell_size_mm)?;
        for body in bodies {
            if body.radius_mm <= 0 {
                return Err(CollisionError::InvalidRadius);
            }

            world.bodies.insert(body.entity_id, body);
        }
        world.rebuild_cells();
        Ok(world)
    }

    pub fn cell_size_mm(&self) -> i32 {
        self.cell_size_mm
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn insert_or_update(&mut self, body: CollisionBody) -> Result<(), CollisionError> {
        if body.radius_mm <= 0 {
            return Err(CollisionError::InvalidRadius);
        }

        self.bodies.insert(body.entity_id, body);
        self.rebuild_cells();
        Ok(())
    }

    pub fn body(&self, entity_id: EntityId) -> Option<CollisionBody> {
        self.bodies.get(&entity_id).copied()
    }

    pub fn detect_overlaps(&self) -> Vec<CollisionContact> {
        let mut seen_pairs = BTreeSet::new();
        let mut contacts = Vec::new();

        for (cell, entity_ids) in &self.cells {
            for neighbor in neighbor_cells(*cell) {
                let Some(neighbor_ids) = self.cells.get(&neighbor) else {
                    continue;
                };

                for a in entity_ids {
                    for b in neighbor_ids {
                        if a == b {
                            continue;
                        }
                        let pair = ordered_pair(*a, *b);
                        if !seen_pairs.insert(pair) {
                            continue;
                        }

                        let Some(contact) = self.contact_for_pair(pair.0, pair.1) else {
                            continue;
                        };
                        contacts.push(contact);
                    }
                }
            }
        }

        contacts.sort_by_key(|contact| (contact.a, contact.b));
        contacts
    }

    pub fn probe_circle(
        &self,
        position: WorldPosition,
        radius_mm: i32,
    ) -> Result<CollisionProbe, CollisionError> {
        if radius_mm <= 0 {
            return Err(CollisionError::InvalidRadius);
        }

        let mut overlapping_entity_ids = Vec::new();
        for body in self.bodies.values() {
            let min_distance_mm = body.radius_mm.saturating_add(radius_mm);
            if distance_sq_mm(position, body.position)
                <= i64::from(min_distance_mm) * i64::from(min_distance_mm)
            {
                overlapping_entity_ids.push(body.entity_id);
            }
        }

        Ok(CollisionProbe {
            position,
            radius_mm,
            overlapping_entity_ids,
        })
    }

    pub fn admit_body_position(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
    ) -> CollisionAdmission {
        let Some(body) = self.bodies.get(&entity_id).copied() else {
            return CollisionAdmission {
                entity_id,
                from: None,
                to: target_position,
                radius_mm: 0,
                result: CollisionAdmissionResult::UnknownBody,
                blocking_entity_ids: Vec::new(),
            };
        };

        let blocking_entity_ids =
            self.overlapping_entity_ids(target_position, body.radius_mm, Some(entity_id));
        let result = if blocking_entity_ids.is_empty() {
            CollisionAdmissionResult::Accepted
        } else {
            CollisionAdmissionResult::RejectedOverlap
        };

        CollisionAdmission {
            entity_id,
            from: Some(body.position),
            to: target_position,
            radius_mm: body.radius_mm,
            result,
            blocking_entity_ids,
        }
    }

    pub fn admit_body_position_after_resolution(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
    ) -> CollisionResolvedAdmission {
        self.admit_body_position_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            None,
        )
    }

    pub fn admit_body_position_after_resolution_with_correction_limit(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> CollisionResolvedAdmission {
        self.admit_body_position_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            Some(correction_limit_abs_mm),
        )
    }

    fn admit_body_position_after_resolution_with_optional_limit(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> CollisionResolvedAdmission {
        let initial = self.admit_body_position(entity_id, target_position);
        if initial.result == CollisionAdmissionResult::UnknownBody {
            return CollisionResolvedAdmission {
                entity_id,
                requested_position: target_position,
                resolved_position: None,
                initial_result: initial.result,
                initial_blocking_entity_ids: initial.blocking_entity_ids,
                result: CollisionResolvedAdmissionResult::UnknownBody,
                blocking_entity_ids: Vec::new(),
                iterations_requested: max_iterations,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm,
                clamped_correction_count: 0,
                final_contact_count: self.detect_overlaps().len(),
                claim_scope: "resolved_admission_only",
            };
        }

        if initial.result == CollisionAdmissionResult::Accepted {
            return CollisionResolvedAdmission {
                entity_id,
                requested_position: target_position,
                resolved_position: Some(target_position),
                initial_result: initial.result,
                initial_blocking_entity_ids: initial.blocking_entity_ids,
                result: CollisionResolvedAdmissionResult::Accepted,
                blocking_entity_ids: Vec::new(),
                iterations_requested: max_iterations,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm,
                clamped_correction_count: 0,
                final_contact_count: self.detect_overlaps().len(),
                claim_scope: "resolved_admission_only",
            };
        }

        let Some(mut body) = self.body(entity_id) else {
            return CollisionResolvedAdmission {
                entity_id,
                requested_position: target_position,
                resolved_position: None,
                initial_result: initial.result,
                initial_blocking_entity_ids: initial.blocking_entity_ids,
                result: CollisionResolvedAdmissionResult::UnknownBody,
                blocking_entity_ids: Vec::new(),
                iterations_requested: max_iterations,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm,
                clamped_correction_count: 0,
                final_contact_count: self.detect_overlaps().len(),
                claim_scope: "resolved_admission_only",
            };
        };

        let mut candidate_world = self.clone();
        body.position = target_position;
        candidate_world
            .insert_or_update(body)
            .expect("existing collision body keeps a valid radius");
        let step = candidate_world
            .step_overlap_resolution_with_limit(max_iterations, correction_limit_abs_mm);
        let Some(resolved_body) = candidate_world.body(entity_id) else {
            return CollisionResolvedAdmission {
                entity_id,
                requested_position: target_position,
                resolved_position: None,
                initial_result: initial.result,
                initial_blocking_entity_ids: initial.blocking_entity_ids,
                result: CollisionResolvedAdmissionResult::UnknownBody,
                blocking_entity_ids: Vec::new(),
                iterations_requested: max_iterations,
                iterations_run: step.iterations_run,
                applied_correction_count: step.applied_correction_count,
                applied_correction_abs_mm_total: step.applied_correction_abs_mm_total,
                max_applied_correction_abs_mm: step.max_applied_correction_abs_mm,
                correction_limit_abs_mm: step.correction_limit_abs_mm,
                clamped_correction_count: step.clamped_correction_count,
                final_contact_count: step.final_contact_count,
                claim_scope: "resolved_admission_only",
            };
        };

        let blocking_entity_ids = candidate_world.overlapping_entity_ids(
            resolved_body.position,
            resolved_body.radius_mm,
            Some(entity_id),
        );
        let result = if blocking_entity_ids.is_empty() {
            CollisionResolvedAdmissionResult::AcceptedAfterResolution
        } else {
            CollisionResolvedAdmissionResult::RejectedStillOverlapping
        };

        CollisionResolvedAdmission {
            entity_id,
            requested_position: target_position,
            resolved_position: Some(resolved_body.position),
            initial_result: initial.result,
            initial_blocking_entity_ids: initial.blocking_entity_ids,
            result,
            blocking_entity_ids,
            iterations_requested: max_iterations,
            iterations_run: step.iterations_run,
            applied_correction_count: step.applied_correction_count,
            applied_correction_abs_mm_total: step.applied_correction_abs_mm_total,
            max_applied_correction_abs_mm: step.max_applied_correction_abs_mm,
            correction_limit_abs_mm: step.correction_limit_abs_mm,
            clamped_correction_count: step.clamped_correction_count,
            final_contact_count: step.final_contact_count,
            claim_scope: "resolved_admission_only",
        }
    }

    pub fn probe_movement_after_resolution(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
    ) -> CollisionMovementProbe {
        self.probe_movement_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            None,
        )
    }

    pub fn probe_movement_after_resolution_with_correction_limit(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> CollisionMovementProbe {
        self.probe_movement_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            Some(correction_limit_abs_mm),
        )
    }

    fn probe_movement_after_resolution_with_optional_limit(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> CollisionMovementProbe {
        let from = self.body(entity_id).map(|body| body.position);
        let admission = self.admit_body_position_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            correction_limit_abs_mm,
        );
        let requested_delta =
            from.map(|position| movement_delta_between(position, target_position));
        let resolved_delta = from.and_then(|position| {
            admission
                .resolved_position
                .map(|resolved_position| movement_delta_between(position, resolved_position))
        });
        let decision = match admission.result {
            CollisionResolvedAdmissionResult::Accepted => CollisionMovementDecision::Accepted,
            CollisionResolvedAdmissionResult::AcceptedAfterResolution => {
                CollisionMovementDecision::Corrected
            }
            CollisionResolvedAdmissionResult::RejectedStillOverlapping => {
                CollisionMovementDecision::Blocked
            }
            CollisionResolvedAdmissionResult::UnknownBody => CollisionMovementDecision::UnknownBody,
        };

        CollisionMovementProbe {
            entity_id,
            from,
            requested_position: target_position,
            resolved_position: admission.resolved_position,
            requested_delta,
            resolved_delta,
            initial_result: admission.initial_result,
            resolved_result: admission.result,
            decision,
            initial_blocking_entity_ids: admission.initial_blocking_entity_ids,
            blocking_entity_ids: admission.blocking_entity_ids,
            iterations_requested: admission.iterations_requested,
            iterations_run: admission.iterations_run,
            applied_correction_count: admission.applied_correction_count,
            applied_correction_abs_mm_total: admission.applied_correction_abs_mm_total,
            max_applied_correction_abs_mm: admission.max_applied_correction_abs_mm,
            correction_limit_abs_mm: admission.correction_limit_abs_mm,
            clamped_correction_count: admission.clamped_correction_count,
            final_contact_count: admission.final_contact_count,
            claim_scope: "movement_probe_only",
        }
    }

    pub fn probe_movement_with_axis_slide_after_resolution(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
    ) -> CollisionMovementSlideProbe {
        self.probe_movement_with_axis_slide_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            None,
        )
    }

    pub fn probe_movement_with_axis_slide_after_resolution_with_correction_limit(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> CollisionMovementSlideProbe {
        self.probe_movement_with_axis_slide_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            Some(correction_limit_abs_mm),
        )
    }

    fn probe_movement_with_axis_slide_after_resolution_with_optional_limit(
        &self,
        entity_id: EntityId,
        target_position: WorldPosition,
        max_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> CollisionMovementSlideProbe {
        let requested_probe = self.probe_movement_after_resolution_with_optional_limit(
            entity_id,
            target_position,
            max_iterations,
            correction_limit_abs_mm,
        );
        let mut decision = requested_probe.decision;
        let mut selected_axis = None;
        let mut selected_position = requested_probe.resolved_position;
        let mut selected_delta = requested_probe.resolved_delta;
        let mut attempts = Vec::new();

        if requested_probe.decision == CollisionMovementDecision::Blocked {
            if let (Some(from), Some(requested_delta)) =
                (requested_probe.from, requested_probe.requested_delta)
            {
                for (axis, slide_target) in axis_slide_targets(from, requested_delta) {
                    let probe = self.probe_movement_after_resolution_with_optional_limit(
                        entity_id,
                        slide_target,
                        max_iterations,
                        correction_limit_abs_mm,
                    );
                    let accepted = matches!(
                        probe.decision,
                        CollisionMovementDecision::Accepted | CollisionMovementDecision::Corrected
                    ) && probe
                        .resolved_delta
                        .is_some_and(|delta| delta.dx_mm != 0 || delta.dy_mm != 0);
                    let attempt = CollisionSlideAttempt { axis, probe };

                    if accepted {
                        decision = attempt.probe.decision;
                        selected_axis = Some(axis);
                        selected_position = attempt.probe.resolved_position;
                        selected_delta = attempt.probe.resolved_delta;
                        attempts.push(attempt);
                        break;
                    }

                    attempts.push(attempt);
                }
            }
        }

        CollisionMovementSlideProbe {
            entity_id,
            requested_probe,
            decision,
            selected_axis,
            selected_position,
            selected_delta,
            attempt_count: attempts.len(),
            attempts,
            claim_scope: "axis_slide_probe_only",
        }
    }

    pub fn probe_batch_movements_after_resolution(
        &self,
        candidates: impl IntoIterator<Item = CollisionBatchMovementCandidate>,
        max_iterations: usize,
    ) -> CollisionBatchMovementProbeReport {
        self.probe_batch_movements_after_resolution_with_optional_limit(
            candidates,
            max_iterations,
            None,
        )
    }

    pub fn probe_batch_movements_after_resolution_with_correction_limit(
        &self,
        candidates: impl IntoIterator<Item = CollisionBatchMovementCandidate>,
        max_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> CollisionBatchMovementProbeReport {
        self.probe_batch_movements_after_resolution_with_optional_limit(
            candidates,
            max_iterations,
            Some(correction_limit_abs_mm),
        )
    }

    fn probe_batch_movements_after_resolution_with_optional_limit(
        &self,
        candidates: impl IntoIterator<Item = CollisionBatchMovementCandidate>,
        max_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> CollisionBatchMovementProbeReport {
        let candidates = candidates.into_iter().collect::<Vec<_>>();
        let mut candidate_world = self.clone();
        let mut initial_samples = Vec::with_capacity(candidates.len());
        let mut known_candidate_count = 0usize;
        let mut initial_rejected_count = 0usize;

        for candidate in candidates {
            let from = self.body(candidate.entity_id).map(|body| body.position);
            let requested_delta =
                from.map(|position| movement_delta_between(position, candidate.target_position));
            let initial = self.admit_body_position(candidate.entity_id, candidate.target_position);
            if initial.result == CollisionAdmissionResult::RejectedOverlap {
                initial_rejected_count += 1;
            }
            if let Some(mut body) = self.body(candidate.entity_id) {
                body.position = candidate.target_position;
                candidate_world
                    .insert_or_update(body)
                    .expect("existing collision body keeps a valid radius");
                known_candidate_count += 1;
            }
            initial_samples.push((
                candidate,
                from,
                requested_delta,
                initial.result,
                initial.blocking_entity_ids,
            ));
        }

        let step = if known_candidate_count > 0 {
            candidate_world
                .step_overlap_resolution_with_limit(max_iterations, correction_limit_abs_mm)
        } else {
            CollisionPhysicsStep {
                initial_contact_count: self.detect_overlaps().len(),
                iterations_requested: max_iterations,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm,
                clamped_correction_count: 0,
                final_contact_count: self.detect_overlaps().len(),
                resolved: true,
                claim_scope: "collision_world_only",
            }
        };

        let mut accepted_count = 0usize;
        let mut corrected_count = 0usize;
        let mut blocked_count = 0usize;
        let mut unknown_body_count = 0usize;
        let mut samples = Vec::with_capacity(initial_samples.len());

        for (candidate, from, requested_delta, initial_result, initial_blocking_entity_ids) in
            initial_samples
        {
            let Some(resolved_body) = candidate_world.body(candidate.entity_id) else {
                unknown_body_count += 1;
                samples.push(CollisionBatchMovementSample {
                    entity_id: candidate.entity_id,
                    from,
                    requested_position: candidate.target_position,
                    resolved_position: None,
                    requested_delta,
                    resolved_delta: None,
                    initial_result,
                    resolved_result: CollisionResolvedAdmissionResult::UnknownBody,
                    decision: CollisionMovementDecision::UnknownBody,
                    initial_blocking_entity_ids,
                    blocking_entity_ids: Vec::new(),
                });
                continue;
            };

            let blocking_entity_ids = candidate_world.overlapping_entity_ids(
                resolved_body.position,
                resolved_body.radius_mm,
                Some(candidate.entity_id),
            );
            let resolved_result = if blocking_entity_ids.is_empty() {
                if initial_result == CollisionAdmissionResult::Accepted
                    && resolved_body.position == candidate.target_position
                {
                    CollisionResolvedAdmissionResult::Accepted
                } else {
                    CollisionResolvedAdmissionResult::AcceptedAfterResolution
                }
            } else {
                CollisionResolvedAdmissionResult::RejectedStillOverlapping
            };
            let decision = match resolved_result {
                CollisionResolvedAdmissionResult::Accepted => {
                    accepted_count += 1;
                    CollisionMovementDecision::Accepted
                }
                CollisionResolvedAdmissionResult::AcceptedAfterResolution => {
                    corrected_count += 1;
                    CollisionMovementDecision::Corrected
                }
                CollisionResolvedAdmissionResult::RejectedStillOverlapping => {
                    blocked_count += 1;
                    CollisionMovementDecision::Blocked
                }
                CollisionResolvedAdmissionResult::UnknownBody => {
                    unknown_body_count += 1;
                    CollisionMovementDecision::UnknownBody
                }
            };
            let resolved_delta =
                from.map(|position| movement_delta_between(position, resolved_body.position));

            samples.push(CollisionBatchMovementSample {
                entity_id: candidate.entity_id,
                from,
                requested_position: candidate.target_position,
                resolved_position: Some(resolved_body.position),
                requested_delta,
                resolved_delta,
                initial_result,
                resolved_result,
                decision,
                initial_blocking_entity_ids,
                blocking_entity_ids,
            });
        }

        CollisionBatchMovementProbeReport {
            candidate_count: samples.len(),
            accepted_count,
            corrected_count,
            blocked_count,
            unknown_body_count,
            initial_rejected_count,
            iterations_requested: max_iterations,
            iterations_run: step.iterations_run,
            applied_correction_count: step.applied_correction_count,
            applied_correction_abs_mm_total: step.applied_correction_abs_mm_total,
            max_applied_correction_abs_mm: step.max_applied_correction_abs_mm,
            correction_limit_abs_mm: step.correction_limit_abs_mm,
            clamped_correction_count: step.clamped_correction_count,
            final_contact_count: step.final_contact_count,
            samples,
            claim_scope: "batch_movement_probe_only",
        }
    }

    pub fn plan_overlap_resolution(&self) -> CollisionResolutionPlan {
        let contacts = self.detect_overlaps();
        let mut corrections: BTreeMap<EntityId, (i64, i64)> = BTreeMap::new();

        for contact in &contacts {
            let Some(first) = self.bodies.get(&contact.a) else {
                continue;
            };
            let Some(second) = self.bodies.get(&contact.b) else {
                continue;
            };

            let (first_delta, second_delta) =
                correction_for_pair(*first, *second, contact.overlap_mm);
            let first_entry = corrections.entry(first.entity_id).or_insert((0, 0));
            first_entry.0 += i64::from(first_delta.dx_mm);
            first_entry.1 += i64::from(first_delta.dy_mm);
            let second_entry = corrections.entry(second.entity_id).or_insert((0, 0));
            second_entry.0 += i64::from(second_delta.dx_mm);
            second_entry.1 += i64::from(second_delta.dy_mm);
        }

        CollisionResolutionPlan {
            contact_count: contacts.len(),
            corrections: corrections
                .into_iter()
                .map(|(entity_id, (dx_mm, dy_mm))| CollisionCorrection {
                    entity_id,
                    dx_mm: saturating_i64_to_i32(dx_mm),
                    dy_mm: saturating_i64_to_i32(dy_mm),
                })
                .filter(|correction| correction.dx_mm != 0 || correction.dy_mm != 0)
                .collect(),
            claim_scope: "resolution_plan_only",
        }
    }

    pub fn apply_resolution_plan(&mut self, plan: &CollisionResolutionPlan) -> usize {
        self.apply_resolution_plan_with_limit(plan, None)
            .applied_correction_count
    }

    fn apply_resolution_plan_with_limit(
        &mut self,
        plan: &CollisionResolutionPlan,
        correction_limit_abs_mm: Option<u32>,
    ) -> AppliedCollisionCorrections {
        let mut applied = AppliedCollisionCorrections::default();
        for correction in &plan.corrections {
            let Some(body) = self.bodies.get_mut(&correction.entity_id) else {
                continue;
            };

            let applied_correction = correction_limit_abs_mm
                .map(|limit| clamp_correction_abs_mm(*correction, limit))
                .unwrap_or(*correction);
            if applied_correction.dx_mm == 0 && applied_correction.dy_mm == 0 {
                continue;
            }
            if correction_abs_mm(&applied_correction) < correction_abs_mm(correction) {
                applied.clamped_correction_count += 1;
            }

            body.position.x_mm = body.position.x_mm.saturating_add(applied_correction.dx_mm);
            body.position.y_mm = body.position.y_mm.saturating_add(applied_correction.dy_mm);
            applied.applied_correction_count += 1;
            let correction_abs_mm = correction_abs_mm(&applied_correction);
            applied.applied_correction_abs_mm_total = applied
                .applied_correction_abs_mm_total
                .saturating_add(u64::from(correction_abs_mm));
            applied.max_applied_correction_abs_mm =
                applied.max_applied_correction_abs_mm.max(correction_abs_mm);
        }

        if applied.applied_correction_count > 0 {
            self.rebuild_cells();
        }
        applied
    }

    pub fn step_overlap_resolution(&mut self, max_iterations: usize) -> CollisionPhysicsStep {
        self.step_overlap_resolution_with_limit(max_iterations, None)
    }

    pub fn step_overlap_resolution_with_correction_limit(
        &mut self,
        max_iterations: usize,
        correction_limit_abs_mm: u32,
    ) -> CollisionPhysicsStep {
        self.step_overlap_resolution_with_limit(max_iterations, Some(correction_limit_abs_mm))
    }

    fn step_overlap_resolution_with_limit(
        &mut self,
        max_iterations: usize,
        correction_limit_abs_mm: Option<u32>,
    ) -> CollisionPhysicsStep {
        let initial_contact_count = self.detect_overlaps().len();
        let mut iterations_run = 0;
        let mut applied_correction_count = 0;
        let mut applied_correction_abs_mm_total = 0u64;
        let mut max_applied_correction_abs_mm = 0u32;
        let mut clamped_correction_count = 0usize;

        for _ in 0..max_iterations {
            let plan = self.plan_overlap_resolution();
            if plan.contact_count == 0 || plan.corrections.is_empty() {
                break;
            }

            let applied = self.apply_resolution_plan_with_limit(&plan, correction_limit_abs_mm);
            applied_correction_count += applied.applied_correction_count;
            applied_correction_abs_mm_total = applied_correction_abs_mm_total
                .saturating_add(applied.applied_correction_abs_mm_total);
            max_applied_correction_abs_mm =
                max_applied_correction_abs_mm.max(applied.max_applied_correction_abs_mm);
            clamped_correction_count += applied.clamped_correction_count;
            iterations_run += 1;
        }

        let final_contact_count = self.detect_overlaps().len();
        CollisionPhysicsStep {
            initial_contact_count,
            iterations_requested: max_iterations,
            iterations_run,
            applied_correction_count,
            applied_correction_abs_mm_total,
            max_applied_correction_abs_mm,
            correction_limit_abs_mm,
            clamped_correction_count,
            final_contact_count,
            resolved: final_contact_count == 0,
            claim_scope: "collision_world_only",
        }
    }

    fn rebuild_cells(&mut self) {
        self.cells.clear();
        for body in self.bodies.values() {
            self.cells
                .entry(cell_for_position(body.position, self.cell_size_mm))
                .or_default()
                .push(body.entity_id);
        }
    }

    fn contact_for_pair(&self, a: EntityId, b: EntityId) -> Option<CollisionContact> {
        let first = self.bodies.get(&a)?;
        let second = self.bodies.get(&b)?;
        let min_distance_mm = first.radius_mm.saturating_add(second.radius_mm);
        let distance_sq_mm = distance_sq_mm(first.position, second.position);
        let min_distance_sq_mm = i64::from(min_distance_mm) * i64::from(min_distance_mm);
        if distance_sq_mm > min_distance_sq_mm {
            return None;
        }

        Some(CollisionContact {
            a,
            b,
            distance_sq_mm: distance_sq_mm as u64,
            min_distance_mm,
            overlap_mm: min_distance_mm.saturating_sub(integer_sqrt(distance_sq_mm as u64)),
        })
    }

    fn overlapping_entity_ids(
        &self,
        position: WorldPosition,
        radius_mm: i32,
        excluded_entity_id: Option<EntityId>,
    ) -> Vec<EntityId> {
        let mut overlapping_entity_ids = Vec::new();
        for body in self.bodies.values() {
            if Some(body.entity_id) == excluded_entity_id {
                continue;
            }

            let min_distance_mm = body.radius_mm.saturating_add(radius_mm);
            if distance_sq_mm(position, body.position)
                <= i64::from(min_distance_mm) * i64::from(min_distance_mm)
            {
                overlapping_entity_ids.push(body.entity_id);
            }
        }
        overlapping_entity_ids
    }
}

pub fn run_collision_perf_smoke(scenario: CollisionPerfScenario) -> CollisionPerfRun {
    let started = Instant::now();
    let bodies = build_collision_perf_bodies(scenario);
    let mut world = CollisionWorld::with_bodies(scenario.cell_size_mm, bodies.clone())
        .expect("collision perf scenario uses valid body and cell sizes");
    let initial_contact_count = world.detect_overlaps().len();
    let resolved_admissions = bodies
        .iter()
        .enumerate()
        .take(4)
        .map(|(index, body)| {
            let admission_iterations = if index.is_multiple_of(2) {
                scenario.max_iterations
            } else {
                0
            };
            world.admit_body_position_after_resolution(
                body.entity_id,
                body.position,
                admission_iterations,
            )
        })
        .collect::<Vec<_>>();
    let resolved_admission_accepted_after_resolution_count = resolved_admissions
        .iter()
        .filter(|admission| {
            admission.result == CollisionResolvedAdmissionResult::AcceptedAfterResolution
        })
        .count();
    let resolved_admission_rejected_count = resolved_admissions
        .iter()
        .filter(|admission| {
            admission.result == CollisionResolvedAdmissionResult::RejectedStillOverlapping
        })
        .count();
    let static_obstacle_step = run_static_obstacle_resolution_check(scenario.max_iterations);
    let clamped_step = run_clamped_resolution_check(
        scenario.max_iterations,
        COLLISION_STABILITY_CLAMP_LIMIT_ABS_MM,
    );
    let step = world.step_overlap_resolution(scenario.max_iterations);
    let elapsed_us = started.elapsed().as_micros().max(1) as u64;

    CollisionPerfRun {
        scenario_id: scenario.id,
        body_count: scenario.body_count,
        cluster_count: scenario.cluster_count,
        resolved_admission_check_count: resolved_admissions.len(),
        resolved_admission_accepted_after_resolution_count,
        resolved_admission_rejected_count,
        static_obstacle_resolution_check_count: usize::from(
            static_obstacle_step.initial_contact_count > 0,
        ),
        static_obstacle_correction_count: static_obstacle_step.applied_correction_count,
        clamped_resolution_check_count: usize::from(clamped_step.correction_limit_abs_mm.is_some()),
        clamped_correction_limit_abs_mm: COLLISION_STABILITY_CLAMP_LIMIT_ABS_MM,
        clamped_correction_count: clamped_step.clamped_correction_count,
        clamped_max_applied_correction_abs_mm: clamped_step.max_applied_correction_abs_mm,
        initial_contact_count,
        iterations_requested: scenario.max_iterations,
        iterations_run: step.iterations_run,
        applied_correction_count: step.applied_correction_count,
        applied_correction_abs_mm_total: step.applied_correction_abs_mm_total,
        max_applied_correction_abs_mm: step.max_applied_correction_abs_mm,
        final_contact_count: step.final_contact_count,
        elapsed_us,
        budget_result: "blocked",
        claim_scope: "local_smoke_only",
    }
}

fn build_collision_perf_bodies(scenario: CollisionPerfScenario) -> Vec<CollisionBody> {
    let mut bodies = Vec::with_capacity(scenario.body_count);
    let cluster_count = scenario.cluster_count.max(1);

    for index in 0..scenario.body_count {
        let cluster = index % cluster_count;
        let lane = index / cluster_count;
        let base_x = (cluster % 50) as i32 * 10_000;
        let base_y = (cluster / 50) as i32 * 10_000;
        let local_x = ((lane % 5) as i32 - 2) * 80;
        let local_y = ((lane / 5) as i32 - 2) * 80;

        bodies.push(CollisionBody {
            entity_id: EntityId(index as u64 + 1),
            kind: CollisionBodyKind::Swarm,
            position: WorldPosition {
                x_mm: base_x + local_x,
                y_mm: base_y + local_y,
            },
            radius_mm: scenario.radius_mm,
        });
    }

    bodies
}

fn run_static_obstacle_resolution_check(max_iterations: usize) -> CollisionPhysicsStep {
    let bodies = vec![
        CollisionBody {
            entity_id: EntityId(900_000),
            kind: CollisionBodyKind::Swarm,
            position: WorldPosition { x_mm: 0, y_mm: 0 },
            radius_mm: 500,
        },
        CollisionBody {
            entity_id: EntityId(900_001),
            kind: CollisionBodyKind::StaticObstacle,
            position: WorldPosition { x_mm: 800, y_mm: 0 },
            radius_mm: 500,
        },
    ];
    let mut world =
        CollisionWorld::with_bodies(1_000, bodies).expect("static obstacle check is valid");
    world.step_overlap_resolution(max_iterations.max(1))
}

fn run_clamped_resolution_check(
    max_iterations: usize,
    correction_limit_abs_mm: u32,
) -> CollisionPhysicsStep {
    let bodies = vec![
        CollisionBody {
            entity_id: EntityId(901_000),
            kind: CollisionBodyKind::Swarm,
            position: WorldPosition { x_mm: 0, y_mm: 0 },
            radius_mm: 500,
        },
        CollisionBody {
            entity_id: EntityId(901_001),
            kind: CollisionBodyKind::Swarm,
            position: WorldPosition { x_mm: 800, y_mm: 0 },
            radius_mm: 500,
        },
    ];
    let mut world =
        CollisionWorld::with_bodies(1_000, bodies).expect("clamped resolution check is valid");
    world.step_overlap_resolution_with_correction_limit(
        max_iterations.max(1),
        correction_limit_abs_mm,
    )
}

fn correction_for_pair(
    first: CollisionBody,
    second: CollisionBody,
    overlap_mm: i32,
) -> (CollisionCorrection, CollisionCorrection) {
    let separation = overlap_mm.saturating_add(1);
    let first_is_static = is_static_body(first);
    let second_is_static = is_static_body(second);
    let dx = second.position.x_mm.saturating_sub(first.position.x_mm);
    let dy = second.position.y_mm.saturating_sub(first.position.y_mm);
    let axis = if dx == 0 && dy == 0 {
        if first.entity_id <= second.entity_id {
            (1, 0)
        } else {
            (-1, 0)
        }
    } else if dx.abs() >= dy.abs() {
        (dx.signum(), 0)
    } else {
        (0, dy.signum())
    };
    let (first_push, second_push) = match (first_is_static, second_is_static) {
        (true, true) => (0, 0),
        (true, false) => (0, separation),
        (false, true) => (separation, 0),
        (false, false) => {
            let first_push = separation / 2;
            (first_push, separation.saturating_sub(first_push))
        }
    };

    (
        CollisionCorrection {
            entity_id: first.entity_id,
            dx_mm: -axis.0 * first_push,
            dy_mm: -axis.1 * first_push,
        },
        CollisionCorrection {
            entity_id: second.entity_id,
            dx_mm: axis.0 * second_push,
            dy_mm: axis.1 * second_push,
        },
    )
}

fn is_static_body(body: CollisionBody) -> bool {
    body.kind == CollisionBodyKind::StaticObstacle
}

fn correction_abs_mm(correction: &CollisionCorrection) -> u32 {
    correction
        .dx_mm
        .unsigned_abs()
        .saturating_add(correction.dy_mm.unsigned_abs())
}

fn clamp_correction_abs_mm(
    correction: CollisionCorrection,
    limit_abs_mm: u32,
) -> CollisionCorrection {
    let correction_abs = correction_abs_mm(&correction);
    if correction_abs <= limit_abs_mm {
        return correction;
    }
    if limit_abs_mm == 0 {
        return CollisionCorrection {
            entity_id: correction.entity_id,
            dx_mm: 0,
            dy_mm: 0,
        };
    }

    let dx_abs = scaled_abs_component(
        correction.dx_mm.unsigned_abs(),
        correction_abs,
        limit_abs_mm,
    );
    let dy_abs = scaled_abs_component(
        correction.dy_mm.unsigned_abs(),
        correction_abs,
        limit_abs_mm,
    );
    let mut clamped = CollisionCorrection {
        entity_id: correction.entity_id,
        dx_mm: dx_abs as i32 * correction.dx_mm.signum(),
        dy_mm: dy_abs as i32 * correction.dy_mm.signum(),
    };
    if clamped.dx_mm == 0 && clamped.dy_mm == 0 {
        if correction.dx_mm.unsigned_abs() >= correction.dy_mm.unsigned_abs() {
            clamped.dx_mm = correction.dx_mm.signum();
        } else {
            clamped.dy_mm = correction.dy_mm.signum();
        }
    }
    clamped
}

fn scaled_abs_component(component_abs: u32, total_abs: u32, limit_abs_mm: u32) -> u32 {
    if component_abs == 0 || total_abs == 0 {
        0
    } else {
        ((u64::from(component_abs) * u64::from(limit_abs_mm)) / u64::from(total_abs)) as u32
    }
}

fn movement_delta_between(from: WorldPosition, to: WorldPosition) -> MovementDelta {
    MovementDelta {
        dx_mm: to.x_mm.saturating_sub(from.x_mm),
        dy_mm: to.y_mm.saturating_sub(from.y_mm),
    }
}

fn axis_slide_targets(
    from: WorldPosition,
    requested_delta: MovementDelta,
) -> Vec<(CollisionSlideAxis, WorldPosition)> {
    let primary_first =
        if requested_delta.dx_mm.unsigned_abs() >= requested_delta.dy_mm.unsigned_abs() {
            [CollisionSlideAxis::X, CollisionSlideAxis::Y]
        } else {
            [CollisionSlideAxis::Y, CollisionSlideAxis::X]
        };
    let mut targets = Vec::new();

    for axis in primary_first {
        let target = match axis {
            CollisionSlideAxis::X if requested_delta.dx_mm != 0 => WorldPosition {
                x_mm: from.x_mm.saturating_add(requested_delta.dx_mm),
                y_mm: from.y_mm,
            },
            CollisionSlideAxis::Y if requested_delta.dy_mm != 0 => WorldPosition {
                x_mm: from.x_mm,
                y_mm: from.y_mm.saturating_add(requested_delta.dy_mm),
            },
            _ => continue,
        };

        if target != from && !targets.iter().any(|(_, existing)| *existing == target) {
            targets.push((axis, target));
        }
    }

    targets
}

fn saturating_i64_to_i32(value: i64) -> i32 {
    if value > i64::from(i32::MAX) {
        i32::MAX
    } else if value < i64::from(i32::MIN) {
        i32::MIN
    } else {
        value as i32
    }
}

fn ordered_pair(a: EntityId, b: EntityId) -> (EntityId, EntityId) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn neighbor_cells(cell: SpatialCell) -> impl Iterator<Item = SpatialCell> {
    (-1..=1).flat_map(move |dx| {
        (-1..=1).map(move |dy| SpatialCell {
            x: cell.x + dx,
            y: cell.y + dy,
        })
    })
}

fn cell_for_position(position: WorldPosition, cell_size_mm: i32) -> SpatialCell {
    SpatialCell {
        x: position.x_mm.div_euclid(cell_size_mm),
        y: position.y_mm.div_euclid(cell_size_mm),
    }
}

fn distance_sq_mm(from: WorldPosition, to: WorldPosition) -> i64 {
    let dx = i64::from(to.x_mm) - i64::from(from.x_mm);
    let dy = i64::from(to.y_mm) - i64::from(from.y_mm);
    dx * dx + dy * dy
}

fn integer_sqrt(value: u64) -> i32 {
    (value as f64).sqrt().floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(entity_id: u64, x_mm: i32, y_mm: i32, radius_mm: i32) -> CollisionBody {
        CollisionBody {
            entity_id: EntityId(entity_id),
            kind: CollisionBodyKind::Swarm,
            position: WorldPosition { x_mm, y_mm },
            radius_mm,
        }
    }

    fn static_body(entity_id: u64, x_mm: i32, y_mm: i32, radius_mm: i32) -> CollisionBody {
        CollisionBody {
            entity_id: EntityId(entity_id),
            kind: CollisionBodyKind::StaticObstacle,
            position: WorldPosition { x_mm, y_mm },
            radius_mm,
        }
    }

    #[test]
    fn collision_world_rejects_invalid_grid_or_body_radius() {
        assert_eq!(
            CollisionWorld::new(0).unwrap_err(),
            CollisionError::InvalidCellSize
        );

        let mut world = CollisionWorld::new(1_000).expect("valid world");
        assert_eq!(
            world.insert_or_update(body(1, 0, 0, 0)).unwrap_err(),
            CollisionError::InvalidRadius
        );
    }

    #[test]
    fn collision_world_detects_deterministic_broad_phase_overlaps() {
        let world = CollisionWorld::with_bodies(
            1_000,
            vec![
                body(10, 0, 0, 600),
                body(11, 900, 0, 600),
                body(12, 3_000, 0, 600),
                body(13, -900, 0, 600),
            ],
        )
        .expect("valid world");

        assert_eq!(world.body_count(), 4);
        assert_eq!(world.cell_size_mm(), 1_000);
        assert_eq!(
            world.detect_overlaps(),
            vec![
                CollisionContact {
                    a: EntityId(10),
                    b: EntityId(11),
                    distance_sq_mm: 810_000,
                    min_distance_mm: 1_200,
                    overlap_mm: 300,
                },
                CollisionContact {
                    a: EntityId(10),
                    b: EntityId(13),
                    distance_sq_mm: 810_000,
                    min_distance_mm: 1_200,
                    overlap_mm: 300,
                },
            ]
        );
    }

    #[test]
    fn collision_probe_returns_sorted_overlapping_entity_ids() {
        let world = CollisionWorld::with_bodies(
            1_000,
            vec![
                body(2, 1_500, 0, 400),
                body(1, 0, 0, 400),
                body(3, 4_000, 0, 400),
            ],
        )
        .expect("valid world");

        let probe = world
            .probe_circle(WorldPosition { x_mm: 700, y_mm: 0 }, 500)
            .expect("valid probe");

        assert_eq!(probe.overlapping_entity_ids, vec![EntityId(1), EntityId(2)]);
    }

    #[test]
    fn collision_admission_accepts_clear_position_and_ignores_self() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 400), body(2, 2_000, 0, 400)])
                .expect("valid world");

        let stay = world.admit_body_position(EntityId(1), WorldPosition { x_mm: 0, y_mm: 0 });
        assert_eq!(
            stay,
            CollisionAdmission {
                entity_id: EntityId(1),
                from: Some(WorldPosition { x_mm: 0, y_mm: 0 }),
                to: WorldPosition { x_mm: 0, y_mm: 0 },
                radius_mm: 400,
                result: CollisionAdmissionResult::Accepted,
                blocking_entity_ids: Vec::new(),
            }
        );

        let clear = world.admit_body_position(
            EntityId(1),
            WorldPosition {
                x_mm: -2_000,
                y_mm: 0,
            },
        );
        assert_eq!(clear.result, CollisionAdmissionResult::Accepted);
        assert!(clear.blocking_entity_ids.is_empty());
    }

    #[test]
    fn collision_admission_rejects_overlap_at_target_position() {
        let world = CollisionWorld::with_bodies(
            1_000,
            vec![
                body(1, 0, 0, 500),
                body(2, 1_200, 0, 500),
                body(3, 4_000, 0, 500),
            ],
        )
        .expect("valid world");

        let admission =
            world.admit_body_position(EntityId(1), WorldPosition { x_mm: 800, y_mm: 0 });

        assert_eq!(admission.result, CollisionAdmissionResult::RejectedOverlap);
        assert_eq!(admission.blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(admission.from, Some(WorldPosition { x_mm: 0, y_mm: 0 }));
    }

    #[test]
    fn collision_admission_reports_unknown_body_without_panic() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500)]).expect("valid world");

        let admission = world.admit_body_position(EntityId(99), WorldPosition { x_mm: 0, y_mm: 0 });

        assert_eq!(
            admission,
            CollisionAdmission {
                entity_id: EntityId(99),
                from: None,
                to: WorldPosition { x_mm: 0, y_mm: 0 },
                radius_mm: 0,
                result: CollisionAdmissionResult::UnknownBody,
                blocking_entity_ids: Vec::new(),
            }
        );
    }

    #[test]
    fn collision_resolved_admission_accepts_clear_position_without_iterations() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 400), body(2, 2_000, 0, 400)])
                .expect("valid world");

        let admission = world.admit_body_position_after_resolution(
            EntityId(1),
            WorldPosition {
                x_mm: -2_000,
                y_mm: 0,
            },
            2,
        );

        assert_eq!(
            admission,
            CollisionResolvedAdmission {
                entity_id: EntityId(1),
                requested_position: WorldPosition {
                    x_mm: -2_000,
                    y_mm: 0,
                },
                resolved_position: Some(WorldPosition {
                    x_mm: -2_000,
                    y_mm: 0,
                }),
                initial_result: CollisionAdmissionResult::Accepted,
                initial_blocking_entity_ids: Vec::new(),
                result: CollisionResolvedAdmissionResult::Accepted,
                blocking_entity_ids: Vec::new(),
                iterations_requested: 2,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm: None,
                clamped_correction_count: 0,
                final_contact_count: 0,
                claim_scope: "resolved_admission_only",
            }
        );
    }

    #[test]
    fn collision_resolved_admission_accepts_after_bounded_local_resolution() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let admission = world.admit_body_position_after_resolution(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            1,
        );

        assert_eq!(admission.entity_id, EntityId(1));
        assert_eq!(
            admission.initial_result,
            CollisionAdmissionResult::RejectedOverlap
        );
        assert_eq!(admission.initial_blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(
            admission.result,
            CollisionResolvedAdmissionResult::AcceptedAfterResolution
        );
        assert_eq!(
            admission.resolved_position,
            Some(WorldPosition { x_mm: 500, y_mm: 0 })
        );
        assert!(admission.blocking_entity_ids.is_empty());
        assert_eq!(admission.iterations_requested, 1);
        assert_eq!(admission.iterations_run, 1);
        assert_eq!(admission.applied_correction_count, 2);
        assert_eq!(admission.applied_correction_abs_mm_total, 601);
        assert_eq!(admission.max_applied_correction_abs_mm, 301);
        assert_eq!(admission.correction_limit_abs_mm, None);
        assert_eq!(admission.clamped_correction_count, 0);
        assert_eq!(admission.final_contact_count, 0);
        assert_eq!(admission.claim_scope, "resolved_admission_only");
    }

    #[test]
    fn collision_resolved_admission_can_clamp_local_resolution() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let admission = world.admit_body_position_after_resolution_with_correction_limit(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            1,
            50,
        );

        assert_eq!(
            admission.initial_result,
            CollisionAdmissionResult::RejectedOverlap
        );
        assert_eq!(admission.initial_blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(
            admission.result,
            CollisionResolvedAdmissionResult::RejectedStillOverlapping
        );
        assert_eq!(
            admission.resolved_position,
            Some(WorldPosition { x_mm: 750, y_mm: 0 })
        );
        assert_eq!(admission.blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(admission.iterations_requested, 1);
        assert_eq!(admission.iterations_run, 1);
        assert_eq!(admission.applied_correction_count, 2);
        assert_eq!(admission.applied_correction_abs_mm_total, 100);
        assert_eq!(admission.max_applied_correction_abs_mm, 50);
        assert_eq!(admission.correction_limit_abs_mm, Some(50));
        assert_eq!(admission.clamped_correction_count, 2);
        assert_eq!(admission.final_contact_count, 1);
    }

    #[test]
    fn collision_resolved_admission_rejects_when_iteration_budget_cannot_resolve() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let admission = world.admit_body_position_after_resolution(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            0,
        );

        assert_eq!(
            admission.initial_result,
            CollisionAdmissionResult::RejectedOverlap
        );
        assert_eq!(admission.initial_blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(
            admission.result,
            CollisionResolvedAdmissionResult::RejectedStillOverlapping
        );
        assert_eq!(
            admission.resolved_position,
            Some(WorldPosition { x_mm: 800, y_mm: 0 })
        );
        assert_eq!(admission.blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(admission.iterations_requested, 0);
        assert_eq!(admission.iterations_run, 0);
        assert_eq!(admission.applied_correction_count, 0);
        assert_eq!(admission.applied_correction_abs_mm_total, 0);
        assert_eq!(admission.max_applied_correction_abs_mm, 0);
        assert_eq!(admission.correction_limit_abs_mm, None);
        assert_eq!(admission.clamped_correction_count, 0);
        assert_eq!(admission.final_contact_count, 1);
    }

    #[test]
    fn collision_resolved_admission_reports_unknown_body_without_iterations() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500)]).expect("valid world");

        let admission = world.admit_body_position_after_resolution(
            EntityId(99),
            WorldPosition { x_mm: 0, y_mm: 0 },
            2,
        );

        assert_eq!(admission.entity_id, EntityId(99));
        assert_eq!(admission.resolved_position, None);
        assert_eq!(
            admission.initial_result,
            CollisionAdmissionResult::UnknownBody
        );
        assert!(admission.initial_blocking_entity_ids.is_empty());
        assert_eq!(
            admission.result,
            CollisionResolvedAdmissionResult::UnknownBody
        );
        assert_eq!(admission.iterations_run, 0);
        assert_eq!(admission.applied_correction_count, 0);
        assert_eq!(admission.applied_correction_abs_mm_total, 0);
        assert_eq!(admission.max_applied_correction_abs_mm, 0);
        assert_eq!(admission.correction_limit_abs_mm, None);
        assert_eq!(admission.clamped_correction_count, 0);
        assert_eq!(admission.claim_scope, "resolved_admission_only");
    }

    #[test]
    fn collision_movement_probe_reports_corrected_candidate_delta() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let probe = world.probe_movement_after_resolution(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            1,
        );

        assert_eq!(probe.entity_id, EntityId(1));
        assert_eq!(probe.from, Some(WorldPosition { x_mm: 0, y_mm: 0 }));
        assert_eq!(
            probe.requested_position,
            WorldPosition { x_mm: 800, y_mm: 0 }
        );
        assert_eq!(
            probe.resolved_position,
            Some(WorldPosition { x_mm: 500, y_mm: 0 })
        );
        assert_eq!(
            probe.requested_delta,
            Some(MovementDelta {
                dx_mm: 800,
                dy_mm: 0
            })
        );
        assert_eq!(
            probe.resolved_delta,
            Some(MovementDelta {
                dx_mm: 500,
                dy_mm: 0
            })
        );
        assert_eq!(
            probe.initial_result,
            CollisionAdmissionResult::RejectedOverlap
        );
        assert_eq!(
            probe.resolved_result,
            CollisionResolvedAdmissionResult::AcceptedAfterResolution
        );
        assert_eq!(probe.decision, CollisionMovementDecision::Corrected);
        assert_eq!(probe.initial_blocking_entity_ids, vec![EntityId(2)]);
        assert!(probe.blocking_entity_ids.is_empty());
        assert_eq!(probe.iterations_run, 1);
        assert_eq!(probe.applied_correction_count, 2);
        assert_eq!(probe.applied_correction_abs_mm_total, 601);
        assert_eq!(probe.max_applied_correction_abs_mm, 301);
        assert_eq!(probe.correction_limit_abs_mm, None);
        assert_eq!(probe.clamped_correction_count, 0);
        assert_eq!(probe.final_contact_count, 0);
        assert_eq!(probe.claim_scope, "movement_probe_only");
    }

    #[test]
    fn collision_movement_probe_can_clamp_local_resolution() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let probe = world.probe_movement_after_resolution_with_correction_limit(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            1,
            50,
        );

        assert_eq!(probe.decision, CollisionMovementDecision::Blocked);
        assert_eq!(
            probe.resolved_result,
            CollisionResolvedAdmissionResult::RejectedStillOverlapping
        );
        assert_eq!(
            probe.resolved_position,
            Some(WorldPosition { x_mm: 750, y_mm: 0 })
        );
        assert_eq!(
            probe.resolved_delta,
            Some(MovementDelta {
                dx_mm: 750,
                dy_mm: 0
            })
        );
        assert_eq!(probe.blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(probe.applied_correction_abs_mm_total, 100);
        assert_eq!(probe.max_applied_correction_abs_mm, 50);
        assert_eq!(probe.correction_limit_abs_mm, Some(50));
        assert_eq!(probe.clamped_correction_count, 2);
    }

    #[test]
    fn collision_axis_slide_probe_accepts_axis_candidate_when_direct_move_blocks() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 300), body(2, 800, 800, 300)])
                .expect("valid world");

        let probe = world.probe_movement_with_axis_slide_after_resolution(
            EntityId(1),
            WorldPosition {
                x_mm: 800,
                y_mm: 800,
            },
            0,
        );

        assert_eq!(probe.entity_id, EntityId(1));
        assert_eq!(
            probe.requested_probe.decision,
            CollisionMovementDecision::Blocked
        );
        assert_eq!(probe.decision, CollisionMovementDecision::Accepted);
        assert_eq!(probe.selected_axis, Some(CollisionSlideAxis::X));
        assert_eq!(
            probe.selected_position,
            Some(WorldPosition { x_mm: 800, y_mm: 0 })
        );
        assert_eq!(
            probe.selected_delta,
            Some(MovementDelta {
                dx_mm: 800,
                dy_mm: 0
            })
        );
        assert_eq!(probe.attempt_count, 1);
        assert_eq!(probe.attempts[0].axis, CollisionSlideAxis::X);
        assert_eq!(
            probe.attempts[0].probe.requested_position,
            WorldPosition { x_mm: 800, y_mm: 0 }
        );
        assert_eq!(probe.claim_scope, "axis_slide_probe_only");
    }

    #[test]
    fn collision_axis_slide_probe_skips_attempts_when_direct_move_is_accepted() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 300), body(2, 2_000, 0, 300)])
                .expect("valid world");

        let probe = world.probe_movement_with_axis_slide_after_resolution(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            0,
        );

        assert_eq!(
            probe.requested_probe.decision,
            CollisionMovementDecision::Accepted
        );
        assert_eq!(probe.decision, CollisionMovementDecision::Accepted);
        assert_eq!(probe.selected_axis, None);
        assert_eq!(
            probe.selected_position,
            Some(WorldPosition { x_mm: 800, y_mm: 0 })
        );
        assert_eq!(probe.attempt_count, 0);
        assert!(probe.attempts.is_empty());
    }

    #[test]
    fn collision_axis_slide_probe_carries_correction_limit_into_slide_attempts() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 800, 800, 500)])
                .expect("valid world");

        let probe = world.probe_movement_with_axis_slide_after_resolution_with_correction_limit(
            EntityId(1),
            WorldPosition {
                x_mm: 800,
                y_mm: 800,
            },
            1,
            50,
        );

        assert_eq!(probe.requested_probe.correction_limit_abs_mm, Some(50));
        assert_eq!(probe.decision, CollisionMovementDecision::Blocked);
        assert_eq!(probe.attempt_count, 2);
        assert!(probe
            .attempts
            .iter()
            .all(|attempt| attempt.probe.correction_limit_abs_mm == Some(50)));
        assert!(probe
            .attempts
            .iter()
            .any(|attempt| attempt.probe.clamped_correction_count > 0));
    }

    #[test]
    fn collision_batch_movement_probe_resolves_multiple_candidates_deterministically() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let report = world.probe_batch_movements_after_resolution(
            vec![CollisionBatchMovementCandidate {
                entity_id: EntityId(1),
                target_position: WorldPosition { x_mm: 800, y_mm: 0 },
            }],
            1,
        );

        assert_eq!(report.candidate_count, 1);
        assert_eq!(report.accepted_count, 0);
        assert_eq!(report.corrected_count, 1);
        assert_eq!(report.blocked_count, 0);
        assert_eq!(report.unknown_body_count, 0);
        assert_eq!(report.initial_rejected_count, 1);
        assert_eq!(report.iterations_requested, 1);
        assert_eq!(report.iterations_run, 1);
        assert_eq!(report.applied_correction_count, 2);
        assert_eq!(report.applied_correction_abs_mm_total, 601);
        assert_eq!(report.max_applied_correction_abs_mm, 301);
        assert_eq!(report.correction_limit_abs_mm, None);
        assert_eq!(report.clamped_correction_count, 0);
        assert_eq!(report.final_contact_count, 0);
        assert_eq!(report.claim_scope, "batch_movement_probe_only");
        assert_eq!(report.samples.len(), 1);
        assert_eq!(
            report.samples[0].resolved_position,
            Some(WorldPosition { x_mm: 500, y_mm: 0 })
        );
        assert_eq!(
            report.samples[0].decision,
            CollisionMovementDecision::Corrected
        );
    }

    #[test]
    fn collision_batch_movement_probe_can_clamp_local_resolution() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let report = world.probe_batch_movements_after_resolution_with_correction_limit(
            vec![
                CollisionBatchMovementCandidate {
                    entity_id: EntityId(1),
                    target_position: WorldPosition { x_mm: 800, y_mm: 0 },
                },
                CollisionBatchMovementCandidate {
                    entity_id: EntityId(99),
                    target_position: WorldPosition { x_mm: 0, y_mm: 0 },
                },
            ],
            1,
            50,
        );

        assert_eq!(report.candidate_count, 2);
        assert_eq!(report.accepted_count, 0);
        assert_eq!(report.corrected_count, 0);
        assert_eq!(report.blocked_count, 1);
        assert_eq!(report.unknown_body_count, 1);
        assert_eq!(report.initial_rejected_count, 1);
        assert_eq!(report.iterations_requested, 1);
        assert_eq!(report.iterations_run, 1);
        assert_eq!(report.applied_correction_count, 2);
        assert_eq!(report.applied_correction_abs_mm_total, 100);
        assert_eq!(report.max_applied_correction_abs_mm, 50);
        assert_eq!(report.correction_limit_abs_mm, Some(50));
        assert_eq!(report.clamped_correction_count, 2);
        assert_eq!(report.final_contact_count, 1);
        assert_eq!(report.samples.len(), 2);
        assert_eq!(
            report.samples[0].resolved_position,
            Some(WorldPosition { x_mm: 750, y_mm: 0 })
        );
        assert_eq!(
            report.samples[0].decision,
            CollisionMovementDecision::Blocked
        );
        assert_eq!(report.samples[0].blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(
            report.samples[1].decision,
            CollisionMovementDecision::UnknownBody
        );
        assert_eq!(report.samples[1].resolved_position, None);
    }

    #[test]
    fn collision_movement_probe_reports_blocked_and_unknown_candidates() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 1_200, 0, 500)])
                .expect("valid world");

        let blocked = world.probe_movement_after_resolution(
            EntityId(1),
            WorldPosition { x_mm: 800, y_mm: 0 },
            0,
        );
        assert_eq!(blocked.decision, CollisionMovementDecision::Blocked);
        assert_eq!(
            blocked.resolved_result,
            CollisionResolvedAdmissionResult::RejectedStillOverlapping
        );
        assert_eq!(
            blocked.resolved_delta,
            Some(MovementDelta {
                dx_mm: 800,
                dy_mm: 0
            })
        );
        assert_eq!(blocked.blocking_entity_ids, vec![EntityId(2)]);
        assert_eq!(blocked.applied_correction_abs_mm_total, 0);
        assert_eq!(blocked.max_applied_correction_abs_mm, 0);
        assert_eq!(blocked.correction_limit_abs_mm, None);
        assert_eq!(blocked.clamped_correction_count, 0);

        let unknown = world.probe_movement_after_resolution(
            EntityId(99),
            WorldPosition { x_mm: 0, y_mm: 0 },
            1,
        );
        assert_eq!(unknown.decision, CollisionMovementDecision::UnknownBody);
        assert_eq!(unknown.from, None);
        assert_eq!(unknown.requested_delta, None);
        assert_eq!(unknown.resolved_delta, None);
        assert_eq!(unknown.applied_correction_abs_mm_total, 0);
        assert_eq!(unknown.max_applied_correction_abs_mm, 0);
        assert_eq!(unknown.correction_limit_abs_mm, None);
        assert_eq!(unknown.clamped_correction_count, 0);
    }

    #[test]
    fn collision_resolution_plan_is_empty_without_contacts() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 400), body(2, 2_000, 0, 400)])
                .expect("valid world");

        let plan = world.plan_overlap_resolution();

        assert_eq!(
            plan,
            CollisionResolutionPlan {
                contact_count: 0,
                corrections: Vec::new(),
                claim_scope: "resolution_plan_only",
            }
        );
    }

    #[test]
    fn collision_resolution_plan_splits_overlap_deterministically() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 800, 0, 500)])
                .expect("valid world");

        let plan = world.plan_overlap_resolution();

        assert_eq!(plan.contact_count, 1);
        assert_eq!(plan.claim_scope, "resolution_plan_only");
        assert_eq!(
            plan.corrections,
            vec![
                CollisionCorrection {
                    entity_id: EntityId(1),
                    dx_mm: -100,
                    dy_mm: 0,
                },
                CollisionCorrection {
                    entity_id: EntityId(2),
                    dx_mm: 101,
                    dy_mm: 0,
                },
            ]
        );
    }

    #[test]
    fn collision_resolution_plan_handles_identical_positions_by_entity_order() {
        let world =
            CollisionWorld::with_bodies(1_000, vec![body(10, 0, 0, 300), body(11, 0, 0, 300)])
                .expect("valid world");

        let plan = world.plan_overlap_resolution();

        assert_eq!(plan.contact_count, 1);
        assert_eq!(
            plan.corrections,
            vec![
                CollisionCorrection {
                    entity_id: EntityId(10),
                    dx_mm: -300,
                    dy_mm: 0,
                },
                CollisionCorrection {
                    entity_id: EntityId(11),
                    dx_mm: 301,
                    dy_mm: 0,
                },
            ]
        );
    }

    #[test]
    fn collision_resolution_plan_keeps_static_obstacles_immovable() {
        let world = CollisionWorld::with_bodies(
            1_000,
            vec![body(1, 0, 0, 500), static_body(2, 800, 0, 500)],
        )
        .expect("valid world");

        let plan = world.plan_overlap_resolution();

        assert_eq!(plan.contact_count, 1);
        assert_eq!(
            plan.corrections,
            vec![CollisionCorrection {
                entity_id: EntityId(1),
                dx_mm: -201,
                dy_mm: 0,
            }]
        );
    }

    #[test]
    fn collision_resolution_plan_does_not_correct_static_static_contacts() {
        let world = CollisionWorld::with_bodies(
            1_000,
            vec![static_body(1, 0, 0, 500), static_body(2, 800, 0, 500)],
        )
        .expect("valid world");

        let plan = world.plan_overlap_resolution();

        assert_eq!(plan.contact_count, 1);
        assert!(plan.corrections.is_empty());
    }

    #[test]
    fn collision_world_applies_resolution_plan_to_local_body_positions() {
        let mut world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 800, 0, 500)])
                .expect("valid world");
        let plan = world.plan_overlap_resolution();

        assert_eq!(world.apply_resolution_plan(&plan), 2);

        assert_eq!(
            world.body(EntityId(1)).map(|body| body.position),
            Some(WorldPosition {
                x_mm: -100,
                y_mm: 0,
            })
        );
        assert_eq!(
            world.body(EntityId(2)).map(|body| body.position),
            Some(WorldPosition { x_mm: 901, y_mm: 0 })
        );
        assert!(world.detect_overlaps().is_empty());
    }

    #[test]
    fn collision_physics_step_resolves_against_static_obstacle_without_moving_it() {
        let mut world = CollisionWorld::with_bodies(
            1_000,
            vec![body(1, 0, 0, 500), static_body(2, 800, 0, 500)],
        )
        .expect("valid world");

        let step = world.step_overlap_resolution(1);

        assert_eq!(step.initial_contact_count, 1);
        assert_eq!(step.iterations_run, 1);
        assert_eq!(step.applied_correction_count, 1);
        assert_eq!(step.applied_correction_abs_mm_total, 201);
        assert_eq!(step.max_applied_correction_abs_mm, 201);
        assert_eq!(step.correction_limit_abs_mm, None);
        assert_eq!(step.clamped_correction_count, 0);
        assert_eq!(step.final_contact_count, 0);
        assert_eq!(
            world.body(EntityId(1)).map(|body| body.position),
            Some(WorldPosition {
                x_mm: -201,
                y_mm: 0
            })
        );
        assert_eq!(
            world.body(EntityId(2)).map(|body| body.position),
            Some(WorldPosition { x_mm: 800, y_mm: 0 })
        );
    }

    #[test]
    fn collision_physics_step_leaves_static_static_contacts_unresolved() {
        let mut world = CollisionWorld::with_bodies(
            1_000,
            vec![static_body(1, 0, 0, 500), static_body(2, 800, 0, 500)],
        )
        .expect("valid world");

        let step = world.step_overlap_resolution(2);

        assert_eq!(step.initial_contact_count, 1);
        assert_eq!(step.iterations_run, 0);
        assert_eq!(step.applied_correction_count, 0);
        assert_eq!(step.applied_correction_abs_mm_total, 0);
        assert_eq!(step.max_applied_correction_abs_mm, 0);
        assert_eq!(step.correction_limit_abs_mm, None);
        assert_eq!(step.clamped_correction_count, 0);
        assert_eq!(step.final_contact_count, 1);
        assert!(!step.resolved);
        assert_eq!(
            world.body(EntityId(1)).map(|body| body.position),
            Some(WorldPosition { x_mm: 0, y_mm: 0 })
        );
        assert_eq!(
            world.body(EntityId(2)).map(|body| body.position),
            Some(WorldPosition { x_mm: 800, y_mm: 0 })
        );
    }

    #[test]
    fn collision_physics_step_resolves_simple_overlap_with_bounded_iterations() {
        let mut world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 800, 0, 500)])
                .expect("valid world");

        let step = world.step_overlap_resolution(1);

        assert_eq!(
            step,
            CollisionPhysicsStep {
                initial_contact_count: 1,
                iterations_requested: 1,
                iterations_run: 1,
                applied_correction_count: 2,
                applied_correction_abs_mm_total: 201,
                max_applied_correction_abs_mm: 101,
                correction_limit_abs_mm: None,
                clamped_correction_count: 0,
                final_contact_count: 0,
                resolved: true,
                claim_scope: "collision_world_only",
            }
        );
    }

    #[test]
    fn collision_physics_step_can_clamp_large_local_corrections() {
        let mut world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 800, 0, 500)])
                .expect("valid world");

        let step = world.step_overlap_resolution_with_correction_limit(1, 50);

        assert_eq!(step.initial_contact_count, 1);
        assert_eq!(step.iterations_requested, 1);
        assert_eq!(step.iterations_run, 1);
        assert_eq!(step.applied_correction_count, 2);
        assert_eq!(step.applied_correction_abs_mm_total, 100);
        assert_eq!(step.max_applied_correction_abs_mm, 50);
        assert_eq!(step.correction_limit_abs_mm, Some(50));
        assert_eq!(step.clamped_correction_count, 2);
        assert_eq!(step.final_contact_count, 1);
        assert!(!step.resolved);
        assert_eq!(
            world.body(EntityId(1)).map(|body| body.position),
            Some(WorldPosition { x_mm: -50, y_mm: 0 })
        );
        assert_eq!(
            world.body(EntityId(2)).map(|body| body.position),
            Some(WorldPosition { x_mm: 850, y_mm: 0 })
        );
    }

    #[test]
    fn collision_physics_step_respects_zero_iteration_budget() {
        let mut world =
            CollisionWorld::with_bodies(1_000, vec![body(1, 0, 0, 500), body(2, 800, 0, 500)])
                .expect("valid world");

        let step = world.step_overlap_resolution(0);

        assert_eq!(
            step,
            CollisionPhysicsStep {
                initial_contact_count: 1,
                iterations_requested: 0,
                iterations_run: 0,
                applied_correction_count: 0,
                applied_correction_abs_mm_total: 0,
                max_applied_correction_abs_mm: 0,
                correction_limit_abs_mm: None,
                clamped_correction_count: 0,
                final_contact_count: 1,
                resolved: false,
                claim_scope: "collision_world_only",
            }
        );
        assert_eq!(
            world.body(EntityId(1)).map(|body| body.position),
            Some(WorldPosition { x_mm: 0, y_mm: 0 })
        );
    }

    #[test]
    fn collision_perf_smoke_scenarios_cover_1k_and_5k() {
        let scenarios = collision_perf_scenarios()
            .iter()
            .map(|scenario| (scenario.id.as_str(), scenario.body_count))
            .collect::<Vec<_>>();

        assert_eq!(
            scenarios,
            vec![
                ("collision_clustered_swarm_1k", 1_000),
                ("collision_clustered_swarm_5k", 5_000),
            ]
        );
    }

    #[test]
    fn collision_perf_smoke_records_1k_contacts_iterations_and_time() {
        let run = run_collision_perf_smoke(COLLISION_PERF_SCENARIOS[0]);

        assert_eq!(run.scenario_id, CollisionPerfScenarioId::ClusteredSwarm1k);
        assert_eq!(run.body_count, 1_000);
        assert_eq!(run.cluster_count, 50);
        assert_eq!(run.resolved_admission_check_count, 4);
        assert!(run.resolved_admission_accepted_after_resolution_count > 0);
        assert!(run.resolved_admission_rejected_count > 0);
        assert_eq!(run.static_obstacle_resolution_check_count, 1);
        assert!(run.static_obstacle_correction_count > 0);
        assert_eq!(run.clamped_resolution_check_count, 1);
        assert_eq!(
            run.clamped_correction_limit_abs_mm,
            COLLISION_STABILITY_CLAMP_LIMIT_ABS_MM
        );
        assert!(run.clamped_correction_count > 0);
        assert!(run.clamped_max_applied_correction_abs_mm <= run.clamped_correction_limit_abs_mm);
        assert!(run.initial_contact_count > 1_000);
        assert_eq!(run.iterations_requested, 2);
        assert!(run.iterations_run > 0);
        assert!(run.iterations_run <= run.iterations_requested);
        assert!(run.applied_correction_count > 0);
        assert!(run.elapsed_us > 0);
        assert_eq!(run.budget_result, "blocked");
        assert_eq!(run.claim_scope, "local_smoke_only");
    }

    #[test]
    fn collision_perf_smoke_records_bounded_5k_physics_pressure() {
        let run = run_collision_perf_smoke(COLLISION_PERF_SCENARIOS[1]);

        assert_eq!(run.scenario_id, CollisionPerfScenarioId::ClusteredSwarm5k);
        assert_eq!(run.body_count, 5_000);
        assert_eq!(run.cluster_count, 250);
        assert_eq!(run.resolved_admission_check_count, 4);
        assert!(run.resolved_admission_accepted_after_resolution_count > 0);
        assert!(run.resolved_admission_rejected_count > 0);
        assert_eq!(run.static_obstacle_resolution_check_count, 1);
        assert!(run.static_obstacle_correction_count > 0);
        assert_eq!(run.clamped_resolution_check_count, 1);
        assert_eq!(
            run.clamped_correction_limit_abs_mm,
            COLLISION_STABILITY_CLAMP_LIMIT_ABS_MM
        );
        assert!(run.clamped_correction_count > 0);
        assert!(run.clamped_max_applied_correction_abs_mm <= run.clamped_correction_limit_abs_mm);
        assert!(run.initial_contact_count > 5_000);
        assert_eq!(run.iterations_requested, 2);
        assert!(run.iterations_run > 0);
        assert!(run.iterations_run <= run.iterations_requested);
        assert!(run.applied_correction_count > 0);
        assert!(run.elapsed_us > 0);
        assert_eq!(run.budget_result, "blocked");
        assert_eq!(run.claim_scope, "local_smoke_only");
    }
}
