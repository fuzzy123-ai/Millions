use std::collections::BTreeMap;

pub const SERVER_TICK_HZ: u32 = 20;
pub const SERVER_TICK_MILLIS: u64 = 1000 / SERVER_TICK_HZ as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerSessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPosition {
    pub x_mm: i32,
    pub y_mm: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementDelta {
    pub dx_mm: i32,
    pub dy_mm: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityState {
    pub entity_id: EntityId,
    pub entity_kind: u16,
    pub faction_id: u16,
    pub flags: u32,
    pub position: WorldPosition,
    pub facing_millirad: i32,
    pub health_q8: u16,
    pub state_id: u16,
    pub state_param_q8: i16,
}

impl EntityState {
    pub fn apply_movement_stub(&mut self, delta: MovementDelta) {
        self.position.x_mm = self.position.x_mm.saturating_add(delta.dx_mm);
        self.position.y_mm = self.position.y_mm.saturating_add(delta.dy_mm);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityColumns {
    entity_ids: Vec<EntityId>,
    entity_kinds: Vec<u16>,
    faction_ids: Vec<u16>,
    flags: Vec<u32>,
    x_mm: Vec<i32>,
    y_mm: Vec<i32>,
    facing_millirad: Vec<i32>,
    health_q8: Vec<u16>,
    state_ids: Vec<u16>,
    state_param_q8: Vec<i16>,
}

impl EntityColumns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entity_ids: Vec::with_capacity(capacity),
            entity_kinds: Vec::with_capacity(capacity),
            faction_ids: Vec::with_capacity(capacity),
            flags: Vec::with_capacity(capacity),
            x_mm: Vec::with_capacity(capacity),
            y_mm: Vec::with_capacity(capacity),
            facing_millirad: Vec::with_capacity(capacity),
            health_q8: Vec::with_capacity(capacity),
            state_ids: Vec::with_capacity(capacity),
            state_param_q8: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, entity: EntityState) -> usize {
        let index = self.entity_ids.len();
        self.entity_ids.push(entity.entity_id);
        self.entity_kinds.push(entity.entity_kind);
        self.faction_ids.push(entity.faction_id);
        self.flags.push(entity.flags);
        self.x_mm.push(entity.position.x_mm);
        self.y_mm.push(entity.position.y_mm);
        self.facing_millirad.push(entity.facing_millirad);
        self.health_q8.push(entity.health_q8);
        self.state_ids.push(entity.state_id);
        self.state_param_q8.push(entity.state_param_q8);
        index
    }

    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entity_ids.is_empty()
    }

    pub fn entity_id(&self, index: usize) -> Option<EntityId> {
        self.entity_ids.get(index).copied()
    }

    pub fn position(&self, index: usize) -> Option<WorldPosition> {
        Some(WorldPosition {
            x_mm: *self.x_mm.get(index)?,
            y_mm: *self.y_mm.get(index)?,
        })
    }

    pub fn apply_movement_stub(&mut self, index: usize, delta: MovementDelta) -> bool {
        let Some(x_mm) = self.x_mm.get_mut(index) else {
            return false;
        };
        let Some(y_mm) = self.y_mm.get_mut(index) else {
            return false;
        };

        *x_mm = x_mm.saturating_add(delta.dx_mm);
        *y_mm = y_mm.saturating_add(delta.dy_mm);
        true
    }

    pub fn entity(&self, index: usize) -> Option<EntityState> {
        Some(EntityState {
            entity_id: *self.entity_ids.get(index)?,
            entity_kind: *self.entity_kinds.get(index)?,
            faction_id: *self.faction_ids.get(index)?,
            flags: *self.flags.get(index)?,
            position: self.position(index)?,
            facing_millirad: *self.facing_millirad.get(index)?,
            health_q8: *self.health_q8.get(index)?,
            state_id: *self.state_ids.get(index)?,
            state_param_q8: *self.state_param_q8.get(index)?,
        })
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = EntityState> + '_ {
        (0..self.len()).filter_map(|index| self.entity(index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpatialCell {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialGrid {
    cell_size_mm: i32,
    cells: BTreeMap<SpatialCell, Vec<EntityId>>,
}

impl SpatialGrid {
    pub fn new(cell_size_mm: i32) -> Self {
        assert!(cell_size_mm > 0, "spatial cell size must be positive");
        Self {
            cell_size_mm,
            cells: BTreeMap::new(),
        }
    }

    pub fn cell_size_mm(&self) -> i32 {
        self.cell_size_mm
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn cell_for_position(&self, position: WorldPosition) -> SpatialCell {
        SpatialCell {
            x: position.x_mm.div_euclid(self.cell_size_mm),
            y: position.y_mm.div_euclid(self.cell_size_mm),
        }
    }

    pub fn rebuild(&mut self, entities: &EntityColumns) {
        self.clear();
        for index in 0..entities.len() {
            let Some(entity_id) = entities.entity_id(index) else {
                continue;
            };
            let Some(position) = entities.position(index) else {
                continue;
            };
            let cell = self.cell_for_position(position);
            self.cells.entry(cell).or_default().push(entity_id);
        }
    }

    pub fn entities_in_cell(&self, cell: SpatialCell) -> &[EntityId] {
        self.cells.get(&cell).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn occupied_cells(&self) -> impl Iterator<Item = (&SpatialCell, &[EntityId])> + '_ {
        self.cells
            .iter()
            .map(|(cell, entity_ids)| (cell, entity_ids.as_slice()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub snapshot_id: u64,
    pub baseline_snapshot_id: u64,
    pub tick: Tick,
    pub entities: Vec<EntityState>,
    pub removed_entities: Vec<EntityId>,
}

pub struct SnapshotBuilder {
    snapshot_id: u64,
    baseline_snapshot_id: u64,
    tick: Tick,
    entities: Vec<EntityState>,
    removed_entities: Vec<EntityId>,
}

impl SnapshotBuilder {
    pub fn full(snapshot_id: u64, tick: Tick) -> Self {
        Self {
            snapshot_id,
            baseline_snapshot_id: 0,
            tick,
            entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    pub fn delta(snapshot_id: u64, baseline_snapshot_id: u64, tick: Tick) -> Self {
        Self {
            snapshot_id,
            baseline_snapshot_id,
            tick,
            entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    pub fn push_entity(&mut self, entity: EntityState) {
        self.entities.push(entity);
    }

    pub fn push_removed(&mut self, entity_id: EntityId) {
        self.removed_entities.push(entity_id);
    }

    pub fn build(self) -> Snapshot {
        Snapshot {
            snapshot_id: self.snapshot_id,
            baseline_snapshot_id: self.baseline_snapshot_id,
            tick: self.tick,
            entities: self.entities,
            removed_entities: self.removed_entities,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickConfig {
    pub tick_hz: u32,
    pub tick_millis: u64,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            tick_hz: SERVER_TICK_HZ,
            tick_millis: SERVER_TICK_MILLIS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickLoop {
    config: TickConfig,
    current_tick: Tick,
}

impl TickLoop {
    pub fn new(config: TickConfig) -> Self {
        Self {
            config,
            current_tick: Tick(0),
        }
    }

    pub fn foundation_default() -> Self {
        Self::new(TickConfig::default())
    }

    pub fn config(&self) -> TickConfig {
        self.config
    }

    pub fn current_tick(&self) -> Tick {
        self.current_tick
    }

    pub fn step(&mut self) -> Tick {
        self.current_tick.0 += 1;
        self.current_tick
    }

    pub fn step_n(&mut self, count: u64) -> Tick {
        self.current_tick.0 += count;
        self.current_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(entity_id: u64, x_mm: i32, y_mm: i32) -> EntityState {
        EntityState {
            entity_id: EntityId(entity_id),
            entity_kind: 2,
            faction_id: 3,
            flags: 4,
            position: WorldPosition { x_mm, y_mm },
            facing_millirad: 900,
            health_q8: 255,
            state_id: 6,
            state_param_q8: -7,
        }
    }

    #[test]
    fn entity_columns_roundtrip_abstract_entity_state() {
        let mut entities = EntityColumns::with_capacity(2);
        let first = entity(10, 1200, -3400);
        let second = entity(11, -1, 0);

        assert_eq!(entities.push(first), 0);
        assert_eq!(entities.push(second), 1);

        assert_eq!(entities.len(), 2);
        assert!(!entities.is_empty());
        assert_eq!(entities.entity(0), Some(first));
        assert_eq!(entities.entity(1), Some(second));
        assert_eq!(entities.entity(2), None);
        assert_eq!(
            entities.iter_entities().collect::<Vec<_>>(),
            vec![first, second]
        );
    }

    #[test]
    fn entity_columns_apply_movement_stub_updates_position_columns() {
        let mut entities = EntityColumns::new();
        entities.push(entity(12, i32::MAX - 10, -10));

        assert!(entities.apply_movement_stub(
            0,
            MovementDelta {
                dx_mm: 20,
                dy_mm: -15,
            }
        ));
        assert!(!entities.apply_movement_stub(1, MovementDelta { dx_mm: 1, dy_mm: 1 }));

        assert_eq!(
            entities.position(0),
            Some(WorldPosition {
                x_mm: i32::MAX,
                y_mm: -25,
            })
        );
    }

    #[test]
    fn spatial_grid_assigns_cells_with_euclidean_negative_coordinates() {
        let grid = SpatialGrid::new(1000);

        assert_eq!(
            grid.cell_for_position(WorldPosition { x_mm: 0, y_mm: 999 }),
            SpatialCell { x: 0, y: 0 }
        );
        assert_eq!(
            grid.cell_for_position(WorldPosition {
                x_mm: -1,
                y_mm: -1000
            }),
            SpatialCell { x: -1, y: -1 }
        );
        assert_eq!(
            grid.cell_for_position(WorldPosition {
                x_mm: -1001,
                y_mm: 1000,
            }),
            SpatialCell { x: -2, y: 1 }
        );
    }

    #[test]
    fn spatial_grid_rebuild_is_deterministic_and_clears_stale_membership() {
        let mut entities = EntityColumns::new();
        entities.push(entity(1, 100, 100));
        entities.push(entity(2, 900, 900));
        entities.push(entity(3, 1000, 1000));

        let mut grid = SpatialGrid::new(1000);
        grid.rebuild(&entities);

        assert_eq!(
            grid.entities_in_cell(SpatialCell { x: 0, y: 0 }),
            &[EntityId(1), EntityId(2)]
        );
        assert_eq!(
            grid.entities_in_cell(SpatialCell { x: 1, y: 1 }),
            &[EntityId(3)]
        );

        let occupied = grid
            .occupied_cells()
            .map(|(cell, ids)| (*cell, ids.to_vec()))
            .collect::<Vec<_>>();
        assert_eq!(
            occupied,
            vec![
                (SpatialCell { x: 0, y: 0 }, vec![EntityId(1), EntityId(2)]),
                (SpatialCell { x: 1, y: 1 }, vec![EntityId(3)]),
            ]
        );

        let mut fewer_entities = EntityColumns::new();
        fewer_entities.push(entity(4, -1, -1));
        grid.rebuild(&fewer_entities);

        assert!(grid.entities_in_cell(SpatialCell { x: 0, y: 0 }).is_empty());
        assert_eq!(
            grid.entities_in_cell(SpatialCell { x: -1, y: -1 }),
            &[EntityId(4)]
        );
    }
}
