use std::collections::{BTreeMap, BTreeSet};

use crate::simulation::{
    EntityId, EntityState, PlayerSessionId, Snapshot, SnapshotBuilder, SpatialCell, SpatialGrid,
    Tick,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AoiRegion {
    pub center: SpatialCell,
    pub radius_cells: u16,
}

impl AoiRegion {
    pub fn new(center: SpatialCell, radius_cells: u16) -> Self {
        Self {
            center,
            radius_cells,
        }
    }

    pub fn contains_cell(self, cell: SpatialCell) -> bool {
        let radius = i32::from(self.radius_cells);
        (cell.x - self.center.x).abs() <= radius && (cell.y - self.center.y).abs() <= radius
    }

    pub fn cells(self) -> Vec<SpatialCell> {
        let radius = i32::from(self.radius_cells);
        let mut cells = Vec::with_capacity(((radius * 2 + 1) * (radius * 2 + 1)) as usize);
        for y in (self.center.y - radius)..=(self.center.y + radius) {
            for x in (self.center.x - radius)..=(self.center.x + radius) {
                cells.push(SpatialCell { x, y });
            }
        }
        cells
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInterestState {
    pub session_id: PlayerSessionId,
    pub region: AoiRegion,
    visible_entities: BTreeSet<EntityId>,
}

impl ClientInterestState {
    pub fn new(session_id: PlayerSessionId, region: AoiRegion) -> Self {
        Self {
            session_id,
            region,
            visible_entities: BTreeSet::new(),
        }
    }

    pub fn visible_entities(&self) -> &BTreeSet<EntityId> {
        &self.visible_entities
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterestUpdate {
    pub session_id: PlayerSessionId,
    pub region: AoiRegion,
    pub visible_entities: BTreeSet<EntityId>,
    pub entered_entities: BTreeSet<EntityId>,
    pub left_entities: BTreeSet<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggregateFarState {
    pub cell: SpatialCell,
    pub entity_count: u32,
    pub representative_entity_id: EntityId,
    pub faction_mask: u64,
    pub flags_or: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterestSnapshotDelta {
    pub snapshot: Snapshot,
    pub aggregate_far_state: Vec<AggregateFarState>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InterestManager {
    subscriptions: BTreeMap<PlayerSessionId, ClientInterestState>,
}

impl InterestManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_subscription(&mut self, session_id: PlayerSessionId, region: AoiRegion) {
        self.subscriptions
            .entry(session_id)
            .and_modify(|state| state.region = region)
            .or_insert_with(|| ClientInterestState::new(session_id, region));
    }

    pub fn remove_subscription(
        &mut self,
        session_id: PlayerSessionId,
    ) -> Option<ClientInterestState> {
        self.subscriptions.remove(&session_id)
    }

    pub fn subscription(&self, session_id: PlayerSessionId) -> Option<&ClientInterestState> {
        self.subscriptions.get(&session_id)
    }

    pub fn refresh_subscription(
        &mut self,
        session_id: PlayerSessionId,
        grid: &SpatialGrid,
    ) -> Option<InterestUpdate> {
        let state = self.subscriptions.get_mut(&session_id)?;
        let next_visible = visible_entities_for_region(state.region, grid);
        let entered_entities = next_visible
            .difference(&state.visible_entities)
            .copied()
            .collect::<BTreeSet<_>>();
        let left_entities = state
            .visible_entities
            .difference(&next_visible)
            .copied()
            .collect::<BTreeSet<_>>();

        state.visible_entities = next_visible.clone();

        Some(InterestUpdate {
            session_id,
            region: state.region,
            visible_entities: next_visible,
            entered_entities,
            left_entities,
        })
    }
}

pub fn visible_entities_for_region(region: AoiRegion, grid: &SpatialGrid) -> BTreeSet<EntityId> {
    let mut visible = BTreeSet::new();
    for cell in region.cells() {
        visible.extend(grid.entities_in_cell(cell).iter().copied());
    }
    visible
}

pub fn build_visible_delta_snapshot(
    update: &InterestUpdate,
    entities: impl IntoIterator<Item = EntityState>,
    snapshot_id: u64,
    baseline_snapshot_id: u64,
    tick: Tick,
) -> Snapshot {
    let entity_by_id = entities
        .into_iter()
        .map(|entity| (entity.entity_id, entity))
        .collect::<BTreeMap<_, _>>();
    let mut builder = SnapshotBuilder::delta(snapshot_id, baseline_snapshot_id, tick);

    for entity_id in &update.visible_entities {
        if let Some(entity) = entity_by_id.get(entity_id) {
            builder.push_entity(*entity);
        }
    }

    for entity_id in &update.left_entities {
        builder.push_removed(*entity_id);
    }

    builder.build()
}

pub fn build_interest_snapshot_delta(
    update: &InterestUpdate,
    entities: impl IntoIterator<Item = EntityState> + Clone,
    grid: &SpatialGrid,
    snapshot_id: u64,
    baseline_snapshot_id: u64,
    tick: Tick,
) -> InterestSnapshotDelta {
    let snapshot = build_visible_delta_snapshot(
        update,
        entities.clone(),
        snapshot_id,
        baseline_snapshot_id,
        tick,
    );
    let aggregate_far_state = aggregate_far_state(update.region, grid, entities);

    InterestSnapshotDelta {
        snapshot,
        aggregate_far_state,
    }
}

pub fn aggregate_far_state(
    visible_region: AoiRegion,
    grid: &SpatialGrid,
    entities: impl IntoIterator<Item = EntityState>,
) -> Vec<AggregateFarState> {
    let entity_by_id = entities
        .into_iter()
        .map(|entity| (entity.entity_id, entity))
        .collect::<BTreeMap<_, _>>();
    let mut aggregate = Vec::new();

    for (cell, entity_ids) in grid.occupied_cells() {
        if visible_region.contains_cell(*cell) || entity_ids.is_empty() {
            continue;
        }

        let mut entity_count = 0_u32;
        let mut representative_entity_id = None;
        let mut faction_mask = 0_u64;
        let mut flags_or = 0_u32;

        for entity_id in entity_ids {
            let Some(entity) = entity_by_id.get(entity_id) else {
                continue;
            };
            entity_count += 1;
            representative_entity_id = Some(
                representative_entity_id
                    .map_or(*entity_id, |current: EntityId| current.min(*entity_id)),
            );
            if entity.faction_id < 64 {
                faction_mask |= 1_u64 << entity.faction_id;
            }
            flags_or |= entity.flags;
        }

        if let Some(representative_entity_id) = representative_entity_id {
            aggregate.push(AggregateFarState {
                cell: *cell,
                entity_count,
                representative_entity_id,
                faction_mask,
                flags_or,
            });
        }
    }

    aggregate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{EntityColumns, EntityState, WorldPosition};

    fn entity(entity_id: u64, x_mm: i32, y_mm: i32) -> EntityState {
        EntityState {
            entity_id: EntityId(entity_id),
            entity_kind: 1,
            faction_id: 1,
            flags: 0,
            position: WorldPosition { x_mm, y_mm },
            facing_millirad: 0,
            health_q8: 256,
            state_id: 0,
            state_param_q8: 0,
        }
    }

    fn grid_for_entities(entities: &[EntityState]) -> SpatialGrid {
        let mut columns = EntityColumns::with_capacity(entities.len());
        for entity in entities {
            columns.push(*entity);
        }
        let mut grid = SpatialGrid::new(1_000);
        grid.rebuild(&columns);
        grid
    }

    #[test]
    fn aoi_region_cells_are_square_and_deterministic() {
        let region = AoiRegion::new(SpatialCell { x: 5, y: -2 }, 1);

        assert!(region.contains_cell(SpatialCell { x: 4, y: -3 }));
        assert!(region.contains_cell(SpatialCell { x: 6, y: -1 }));
        assert!(!region.contains_cell(SpatialCell { x: 7, y: -2 }));
        assert_eq!(
            region.cells(),
            vec![
                SpatialCell { x: 4, y: -3 },
                SpatialCell { x: 5, y: -3 },
                SpatialCell { x: 6, y: -3 },
                SpatialCell { x: 4, y: -2 },
                SpatialCell { x: 5, y: -2 },
                SpatialCell { x: 6, y: -2 },
                SpatialCell { x: 4, y: -1 },
                SpatialCell { x: 5, y: -1 },
                SpatialCell { x: 6, y: -1 },
            ]
        );
    }

    #[test]
    fn visible_entities_are_collected_from_region_cells() {
        let grid = grid_for_entities(&[
            entity(1, 0, 0),
            entity(2, 999, 999),
            entity(3, 1_000, 0),
            entity(4, 3_000, 0),
        ]);
        let region = AoiRegion::new(SpatialCell { x: 0, y: 0 }, 1);

        assert_eq!(
            visible_entities_for_region(region, &grid),
            BTreeSet::from([EntityId(1), EntityId(2), EntityId(3)])
        );
    }

    #[test]
    fn subscription_refresh_reports_entered_and_left_entities() {
        let session = PlayerSessionId(77);
        let mut manager = InterestManager::new();
        manager.upsert_subscription(session, AoiRegion::new(SpatialCell { x: 0, y: 0 }, 0));

        let first_grid = grid_for_entities(&[entity(1, 0, 0), entity(2, 1_000, 0)]);
        let first = manager
            .refresh_subscription(session, &first_grid)
            .expect("subscription exists");

        assert_eq!(first.visible_entities, BTreeSet::from([EntityId(1)]));
        assert_eq!(first.entered_entities, BTreeSet::from([EntityId(1)]));
        assert!(first.left_entities.is_empty());

        manager.upsert_subscription(session, AoiRegion::new(SpatialCell { x: 1, y: 0 }, 0));
        let second_grid = grid_for_entities(&[entity(2, 1_000, 0), entity(3, 1_000, 500)]);
        let second = manager
            .refresh_subscription(session, &second_grid)
            .expect("subscription exists");

        assert_eq!(
            second.visible_entities,
            BTreeSet::from([EntityId(2), EntityId(3)])
        );
        assert_eq!(
            second.entered_entities,
            BTreeSet::from([EntityId(2), EntityId(3)])
        );
        assert_eq!(second.left_entities, BTreeSet::from([EntityId(1)]));
        assert_eq!(
            manager.subscription(session).unwrap().visible_entities(),
            &BTreeSet::from([EntityId(2), EntityId(3)])
        );
    }

    #[test]
    fn subscriptions_are_independent_per_session() {
        let grid = grid_for_entities(&[entity(1, 0, 0), entity(2, 2_000, 0)]);
        let mut manager = InterestManager::new();
        let first = PlayerSessionId(1);
        let second = PlayerSessionId(2);

        manager.upsert_subscription(first, AoiRegion::new(SpatialCell { x: 0, y: 0 }, 0));
        manager.upsert_subscription(second, AoiRegion::new(SpatialCell { x: 2, y: 0 }, 0));

        assert_eq!(
            manager
                .refresh_subscription(first, &grid)
                .unwrap()
                .visible_entities,
            BTreeSet::from([EntityId(1)])
        );
        assert_eq!(
            manager
                .refresh_subscription(second, &grid)
                .unwrap()
                .visible_entities,
            BTreeSet::from([EntityId(2)])
        );

        assert!(manager.remove_subscription(first).is_some());
        assert!(manager.refresh_subscription(first, &grid).is_none());
        assert!(manager.refresh_subscription(second, &grid).is_some());
    }

    #[test]
    fn visible_delta_snapshot_contains_sorted_visible_entities_and_left_ids() {
        let update = InterestUpdate {
            session_id: PlayerSessionId(9),
            region: AoiRegion::new(SpatialCell { x: 0, y: 0 }, 1),
            visible_entities: BTreeSet::from([EntityId(3), EntityId(1)]),
            entered_entities: BTreeSet::from([EntityId(3)]),
            left_entities: BTreeSet::from([EntityId(2)]),
        };

        let snapshot = build_visible_delta_snapshot(
            &update,
            [
                entity(3, 3_000, 0),
                entity(1, 1_000, 0),
                entity(2, 2_000, 0),
            ],
            20,
            19,
            Tick(100),
        );

        assert_eq!(snapshot.snapshot_id, 20);
        assert_eq!(snapshot.baseline_snapshot_id, 19);
        assert_eq!(
            snapshot
                .entities
                .iter()
                .map(|entity| entity.entity_id)
                .collect::<Vec<_>>(),
            vec![EntityId(1), EntityId(3)]
        );
        assert_eq!(snapshot.removed_entities, vec![EntityId(2)]);
    }

    #[test]
    fn aggregate_far_state_skips_visible_cells_and_summarizes_far_cells() {
        let far_same_cell_a = EntityState {
            faction_id: 1,
            flags: 0b0001,
            ..entity(3, 3_000, 0)
        };
        let far_same_cell_b = EntityState {
            faction_id: 2,
            flags: 0b0100,
            ..entity(4, 3_500, 500)
        };
        let entities = [
            entity(1, 0, 0),
            entity(2, 1_000, 0),
            far_same_cell_a,
            far_same_cell_b,
        ];
        let grid = grid_for_entities(&entities);
        let region = AoiRegion::new(SpatialCell { x: 0, y: 0 }, 1);

        let aggregate = aggregate_far_state(region, &grid, entities);

        assert_eq!(
            aggregate,
            vec![AggregateFarState {
                cell: SpatialCell { x: 3, y: 0 },
                entity_count: 2,
                representative_entity_id: EntityId(3),
                faction_mask: (1 << 1) | (1 << 2),
                flags_or: 0b0101,
            }]
        );
    }

    #[test]
    fn interest_snapshot_delta_pairs_visible_delta_with_far_aggregate() {
        let session = PlayerSessionId(12);
        let mut manager = InterestManager::new();
        manager.upsert_subscription(session, AoiRegion::new(SpatialCell { x: 0, y: 0 }, 0));
        let entities = [entity(1, 0, 0), entity(2, 2_000, 0)];
        let grid = grid_for_entities(&entities);
        let update = manager.refresh_subscription(session, &grid).unwrap();

        let delta = build_interest_snapshot_delta(&update, entities, &grid, 30, 29, Tick(120));

        assert_eq!(delta.snapshot.entities.len(), 1);
        assert_eq!(delta.snapshot.entities[0].entity_id, EntityId(1));
        assert_eq!(delta.aggregate_far_state.len(), 1);
        assert_eq!(
            delta.aggregate_far_state[0].representative_entity_id,
            EntityId(2)
        );
    }
}
