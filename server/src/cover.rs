use std::collections::{BTreeMap, BTreeSet};

use crate::{
    validate_map_data_import, EntityId, MapDataImport, MapDataValidationError, MapPoint, MapShape,
    WorldPosition,
};

pub const COVER_SCHEMA: &str = "millions_cover_v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverCombatMap {
    map_id: String,
    map_version: u32,
    checksum: String,
    obstacles: BTreeMap<String, AxisAlignedVolume>,
    cover_objects: BTreeMap<String, CoverObject>,
    occupancy: BTreeMap<String, CoverOccupancy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisAlignedVolume {
    pub id: String,
    pub class_label: String,
    pub min: MapPoint,
    pub max: MapPoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverObject {
    pub id: String,
    pub class_label: String,
    pub volume: AxisAlignedVolume,
    pub slot_capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverOccupancy {
    pub cover_id: String,
    pub slot_capacity: usize,
    pub occupants: BTreeSet<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineQuery {
    pub from: WorldPosition,
    pub to: WorldPosition,
    pub clear: bool,
    pub blocking_obstacle_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetingQuery {
    pub attacker_id: EntityId,
    pub target_id: EntityId,
    pub distance_sq_mm: i64,
    pub max_range_mm: u32,
    pub line_of_fire: LineQuery,
    pub target_cover_id: Option<String>,
    pub result: TargetingResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetingResult {
    InRangeClear,
    InRangeTargetInCover,
    OutOfRange,
    BlockedByObstacle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverCombatPerfRun {
    pub query_count: usize,
    pub in_range_clear: usize,
    pub in_range_target_in_cover: usize,
    pub blocked_by_obstacle: usize,
    pub out_of_range: usize,
    pub claim_scope: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverError {
    InvalidMapData(MapDataValidationError),
    UnknownCoverId,
    CoverFull,
}

impl CoverCombatMap {
    pub fn from_map_data(map: &MapDataImport) -> Result<Self, CoverError> {
        validate_map_data_import(map).map_err(CoverError::InvalidMapData)?;

        let mut obstacles = BTreeMap::new();
        for obstacle in &map.obstacles {
            let volume = AxisAlignedVolume::from_shape(obstacle);
            obstacles.insert(volume.id.clone(), volume);
        }

        let mut cover_objects = BTreeMap::new();
        let mut occupancy = BTreeMap::new();
        for cover_shape in &map.cover_objects {
            let cover = CoverObject::from_shape(cover_shape);
            occupancy.insert(
                cover.id.clone(),
                CoverOccupancy {
                    cover_id: cover.id.clone(),
                    slot_capacity: cover.slot_capacity,
                    occupants: BTreeSet::new(),
                },
            );
            cover_objects.insert(cover.id.clone(), cover);
        }

        Ok(Self {
            map_id: map.map_id.clone(),
            map_version: map.map_version,
            checksum: map.checksum.clone(),
            obstacles,
            cover_objects,
            occupancy,
        })
    }

    pub fn map_id(&self) -> &str {
        &self.map_id
    }

    pub fn map_version(&self) -> u32 {
        self.map_version
    }

    pub fn checksum(&self) -> &str {
        &self.checksum
    }

    pub fn obstacle_count(&self) -> usize {
        self.obstacles.len()
    }

    pub fn cover_count(&self) -> usize {
        self.cover_objects.len()
    }

    pub fn obstacle_at_point(&self, point: MapPoint) -> Option<&AxisAlignedVolume> {
        self.obstacles
            .values()
            .find(|obstacle| obstacle.contains_point(point))
    }

    pub fn cover_at_point(&self, point: MapPoint) -> Option<&CoverObject> {
        self.cover_objects
            .values()
            .find(|cover| cover.volume.contains_point(point))
    }

    pub fn line_of_sight(&self, from: WorldPosition, to: WorldPosition) -> LineQuery {
        self.line_query(from, to)
    }

    pub fn line_of_fire(&self, from: WorldPosition, to: WorldPosition) -> LineQuery {
        self.line_query(from, to)
    }

    pub fn claim_cover_slot(
        &mut self,
        cover_id: &str,
        entity_id: EntityId,
    ) -> Result<(), CoverError> {
        let occupancy = self
            .occupancy
            .get_mut(cover_id)
            .ok_or(CoverError::UnknownCoverId)?;

        if occupancy.occupants.contains(&entity_id) {
            return Ok(());
        }
        if occupancy.occupants.len() >= occupancy.slot_capacity {
            return Err(CoverError::CoverFull);
        }
        occupancy.occupants.insert(entity_id);
        Ok(())
    }

    pub fn release_cover_slot(
        &mut self,
        cover_id: &str,
        entity_id: EntityId,
    ) -> Result<bool, CoverError> {
        let occupancy = self
            .occupancy
            .get_mut(cover_id)
            .ok_or(CoverError::UnknownCoverId)?;
        Ok(occupancy.occupants.remove(&entity_id))
    }

    pub fn occupancy(&self, cover_id: &str) -> Option<&CoverOccupancy> {
        self.occupancy.get(cover_id)
    }

    pub fn evaluate_range_first_targeting(
        &self,
        attacker_id: EntityId,
        attacker_position: WorldPosition,
        target_id: EntityId,
        target_position: WorldPosition,
        max_range_mm: u32,
    ) -> TargetingQuery {
        let distance_sq_mm = distance_sq(attacker_position, target_position);
        let max_range_sq_mm = i64::from(max_range_mm) * i64::from(max_range_mm);
        if distance_sq_mm > max_range_sq_mm {
            return TargetingQuery {
                attacker_id,
                target_id,
                distance_sq_mm,
                max_range_mm,
                line_of_fire: LineQuery {
                    from: attacker_position,
                    to: target_position,
                    clear: true,
                    blocking_obstacle_id: None,
                },
                target_cover_id: self
                    .cover_at_point(MapPoint {
                        x_mm: target_position.x_mm,
                        y_mm: target_position.y_mm,
                    })
                    .map(|cover| cover.id.clone()),
                result: TargetingResult::OutOfRange,
            };
        }

        let line_of_fire = self.line_of_fire(attacker_position, target_position);
        let target_cover_id = self
            .cover_at_point(MapPoint {
                x_mm: target_position.x_mm,
                y_mm: target_position.y_mm,
            })
            .map(|cover| cover.id.clone());
        let result = if !line_of_fire.clear {
            TargetingResult::BlockedByObstacle
        } else if target_cover_id.is_some() {
            TargetingResult::InRangeTargetInCover
        } else {
            TargetingResult::InRangeClear
        };

        TargetingQuery {
            attacker_id,
            target_id,
            distance_sq_mm,
            max_range_mm,
            line_of_fire,
            target_cover_id,
            result,
        }
    }

    pub fn run_dense_targeting_smoke(
        &self,
        attackers: &[WorldPosition],
        targets: &[WorldPosition],
        max_range_mm: u32,
    ) -> CoverCombatPerfRun {
        let mut run = CoverCombatPerfRun {
            query_count: 0,
            in_range_clear: 0,
            in_range_target_in_cover: 0,
            blocked_by_obstacle: 0,
            out_of_range: 0,
            claim_scope: "informational_contract_only",
        };

        for (attacker_index, attacker_position) in attackers.iter().enumerate() {
            for (target_index, target_position) in targets.iter().enumerate() {
                let query = self.evaluate_range_first_targeting(
                    EntityId((attacker_index + 1) as u64),
                    *attacker_position,
                    EntityId((target_index + 10_000) as u64),
                    *target_position,
                    max_range_mm,
                );
                run.query_count += 1;
                match query.result {
                    TargetingResult::InRangeClear => run.in_range_clear += 1,
                    TargetingResult::InRangeTargetInCover => {
                        run.in_range_target_in_cover += 1;
                    }
                    TargetingResult::BlockedByObstacle => run.blocked_by_obstacle += 1,
                    TargetingResult::OutOfRange => run.out_of_range += 1,
                }
            }
        }

        run
    }

    fn line_query(&self, from: WorldPosition, to: WorldPosition) -> LineQuery {
        let from_point = MapPoint {
            x_mm: from.x_mm,
            y_mm: from.y_mm,
        };
        let to_point = MapPoint {
            x_mm: to.x_mm,
            y_mm: to.y_mm,
        };
        let blocking_obstacle_id = self
            .obstacles
            .values()
            .find(|obstacle| obstacle.intersects_segment(from_point, to_point))
            .map(|obstacle| obstacle.id.clone());

        LineQuery {
            from,
            to,
            clear: blocking_obstacle_id.is_none(),
            blocking_obstacle_id,
        }
    }
}

impl AxisAlignedVolume {
    pub fn from_shape(shape: &MapShape) -> Self {
        let min = MapPoint {
            x_mm: shape.center.x_mm.saturating_sub(shape.half_extents_mm.x_mm),
            y_mm: shape.center.y_mm.saturating_sub(shape.half_extents_mm.y_mm),
        };
        let max = MapPoint {
            x_mm: shape.center.x_mm.saturating_add(shape.half_extents_mm.x_mm),
            y_mm: shape.center.y_mm.saturating_add(shape.half_extents_mm.y_mm),
        };

        Self {
            id: shape.id.clone(),
            class_label: shape.class_label.clone(),
            min,
            max,
        }
    }

    pub fn contains_point(&self, point: MapPoint) -> bool {
        point.x_mm >= self.min.x_mm
            && point.x_mm <= self.max.x_mm
            && point.y_mm >= self.min.y_mm
            && point.y_mm <= self.max.y_mm
    }

    pub fn intersects_segment(&self, from: MapPoint, to: MapPoint) -> bool {
        if self.contains_point(from) || self.contains_point(to) {
            return true;
        }

        let corners = [
            self.min,
            MapPoint {
                x_mm: self.max.x_mm,
                y_mm: self.min.y_mm,
            },
            self.max,
            MapPoint {
                x_mm: self.min.x_mm,
                y_mm: self.max.y_mm,
            },
        ];

        for index in 0..corners.len() {
            let edge_start = corners[index];
            let edge_end = corners[(index + 1) % corners.len()];
            if segments_intersect(from, to, edge_start, edge_end) {
                return true;
            }
        }
        false
    }
}

impl CoverObject {
    pub fn from_shape(shape: &MapShape) -> Self {
        let volume = AxisAlignedVolume::from_shape(shape);
        let width_mm = volume.max.x_mm.saturating_sub(volume.min.x_mm).max(1);
        let slot_capacity = usize::try_from((width_mm / 1000).max(1)).unwrap_or(1);

        Self {
            id: shape.id.clone(),
            class_label: shape.class_label.clone(),
            volume,
            slot_capacity,
        }
    }
}

fn segments_intersect(a: MapPoint, b: MapPoint, c: MapPoint, d: MapPoint) -> bool {
    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);

    if o1 == 0 && on_segment(a, c, b) {
        return true;
    }
    if o2 == 0 && on_segment(a, d, b) {
        return true;
    }
    if o3 == 0 && on_segment(c, a, d) {
        return true;
    }
    if o4 == 0 && on_segment(c, b, d) {
        return true;
    }

    (o1 > 0) != (o2 > 0) && (o3 > 0) != (o4 > 0)
}

fn orientation(a: MapPoint, b: MapPoint, c: MapPoint) -> i64 {
    let ab_x = i64::from(b.x_mm) - i64::from(a.x_mm);
    let ab_y = i64::from(b.y_mm) - i64::from(a.y_mm);
    let ac_x = i64::from(c.x_mm) - i64::from(a.x_mm);
    let ac_y = i64::from(c.y_mm) - i64::from(a.y_mm);
    ab_x * ac_y - ab_y * ac_x
}

fn on_segment(a: MapPoint, b: MapPoint, c: MapPoint) -> bool {
    b.x_mm >= a.x_mm.min(c.x_mm)
        && b.x_mm <= a.x_mm.max(c.x_mm)
        && b.y_mm >= a.y_mm.min(c.y_mm)
        && b.y_mm <= a.y_mm.max(c.y_mm)
}

fn distance_sq(from: WorldPosition, to: WorldPosition) -> i64 {
    let dx = i64::from(to.x_mm) - i64::from(from.x_mm);
    let dy = i64::from(to.y_mm) - i64::from(from.y_mm);
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CapturePoint, MapBounds, MapShapeKind, SpawnPoint, MAP_DATA_SCHEMA};

    fn shape(id: &str, kind: crate::MapShapeKind, x_mm: i32, y_mm: i32) -> MapShape {
        MapShape {
            id: id.to_string(),
            kind,
            center: MapPoint { x_mm, y_mm },
            half_extents_mm: MapPoint {
                x_mm: 1000,
                y_mm: 1000,
            },
            class_label: match kind {
                MapShapeKind::Obstacle => "solid_blocker",
                MapShapeKind::Cover => "low_cover",
                MapShapeKind::NavigationHint => "nav_hint",
            }
            .to_string(),
        }
    }

    fn sample_map() -> MapDataImport {
        MapDataImport {
            schema: MAP_DATA_SCHEMA,
            map_id: "cover-test-map".to_string(),
            map_version: 3,
            checksum: "sum16:cover-test".to_string(),
            bounds_mm: MapBounds {
                min_x: 0,
                min_y: 0,
                max_x: 100_000,
                max_y: 100_000,
            },
            spawn_points: vec![SpawnPoint {
                id: "spawn_a".to_string(),
                point: MapPoint {
                    x_mm: 5000,
                    y_mm: 5000,
                },
                faction_label: "local_a".to_string(),
            }],
            capture_points: vec![CapturePoint {
                id: "cap_mid".to_string(),
                center: MapPoint {
                    x_mm: 50_000,
                    y_mm: 50_000,
                },
                radius_mm: 4000,
            }],
            obstacles: vec![shape("wall_mid", MapShapeKind::Obstacle, 50_000, 50_000)],
            cover_objects: vec![shape("cover_west", MapShapeKind::Cover, 20_000, 20_000)],
            navigation_hints: vec![shape(
                "nav_hint_east",
                MapShapeKind::NavigationHint,
                80_000,
                80_000,
            )],
        }
    }

    #[test]
    fn cover_map_imports_obstacles_cover_and_metadata() {
        let cover_map = CoverCombatMap::from_map_data(&sample_map()).expect("cover map imports");

        assert_eq!(COVER_SCHEMA, "millions_cover_v0");
        assert_eq!(cover_map.map_id(), "cover-test-map");
        assert_eq!(cover_map.map_version(), 3);
        assert_eq!(cover_map.checksum(), "sum16:cover-test");
        assert_eq!(cover_map.obstacle_count(), 1);
        assert_eq!(cover_map.cover_count(), 1);
        assert_eq!(
            cover_map
                .obstacle_at_point(MapPoint {
                    x_mm: 50_000,
                    y_mm: 50_000,
                })
                .map(|obstacle| obstacle.id.as_str()),
            Some("wall_mid")
        );
        assert_eq!(
            cover_map
                .cover_at_point(MapPoint {
                    x_mm: 20_000,
                    y_mm: 20_000,
                })
                .map(|cover| cover.id.as_str()),
            Some("cover_west")
        );
    }

    #[test]
    fn cover_map_rejects_invalid_map_data_before_authority() {
        let mut map = sample_map();
        map.cover_objects[0].kind = MapShapeKind::Obstacle;

        assert_eq!(
            CoverCombatMap::from_map_data(&map),
            Err(CoverError::InvalidMapData(
                MapDataValidationError::WrongShapeKind
            ))
        );
    }

    #[test]
    fn line_of_sight_and_fire_are_blocked_by_obstacles_only() {
        let cover_map = CoverCombatMap::from_map_data(&sample_map()).expect("cover map imports");

        let blocked = cover_map.line_of_sight(
            WorldPosition {
                x_mm: 10_000,
                y_mm: 50_000,
            },
            WorldPosition {
                x_mm: 90_000,
                y_mm: 50_000,
            },
        );
        assert!(!blocked.clear);
        assert_eq!(blocked.blocking_obstacle_id.as_deref(), Some("wall_mid"));

        let clear = cover_map.line_of_fire(
            WorldPosition {
                x_mm: 10_000,
                y_mm: 10_000,
            },
            WorldPosition {
                x_mm: 90_000,
                y_mm: 10_000,
            },
        );
        assert!(clear.clear);
        assert_eq!(clear.blocking_obstacle_id, None);
    }

    #[test]
    fn cover_occupancy_is_deterministic_bounded_and_idempotent() {
        let mut cover_map =
            CoverCombatMap::from_map_data(&sample_map()).expect("cover map imports");

        assert_eq!(
            cover_map.claim_cover_slot("cover_west", EntityId(101)),
            Ok(())
        );
        assert_eq!(
            cover_map.claim_cover_slot("cover_west", EntityId(101)),
            Ok(())
        );
        assert_eq!(
            cover_map.claim_cover_slot("cover_west", EntityId(102)),
            Ok(())
        );
        assert_eq!(
            cover_map.claim_cover_slot("cover_west", EntityId(103)),
            Err(CoverError::CoverFull)
        );
        assert_eq!(
            cover_map.occupancy("cover_west").map(|occupancy| occupancy
                .occupants
                .iter()
                .copied()
                .collect::<Vec<_>>()),
            Some(vec![EntityId(101), EntityId(102)])
        );
        assert_eq!(
            cover_map.release_cover_slot("cover_west", EntityId(101)),
            Ok(true)
        );
        assert_eq!(
            cover_map.claim_cover_slot("missing_cover", EntityId(201)),
            Err(CoverError::UnknownCoverId)
        );
    }

    #[test]
    fn range_first_targeting_reports_clear_cover_blocked_and_out_of_range() {
        let cover_map = CoverCombatMap::from_map_data(&sample_map()).expect("cover map imports");

        let clear = cover_map.evaluate_range_first_targeting(
            EntityId(1),
            WorldPosition {
                x_mm: 10_000,
                y_mm: 10_000,
            },
            EntityId(2),
            WorldPosition {
                x_mm: 15_000,
                y_mm: 10_000,
            },
            10_000,
        );
        assert_eq!(clear.result, TargetingResult::InRangeClear);
        assert!(clear.line_of_fire.clear);
        assert_eq!(clear.target_cover_id, None);

        let target_in_cover = cover_map.evaluate_range_first_targeting(
            EntityId(1),
            WorldPosition {
                x_mm: 10_000,
                y_mm: 20_000,
            },
            EntityId(3),
            WorldPosition {
                x_mm: 20_000,
                y_mm: 20_000,
            },
            15_000,
        );
        assert_eq!(
            target_in_cover.result,
            TargetingResult::InRangeTargetInCover
        );
        assert_eq!(
            target_in_cover.target_cover_id.as_deref(),
            Some("cover_west")
        );

        let blocked = cover_map.evaluate_range_first_targeting(
            EntityId(1),
            WorldPosition {
                x_mm: 10_000,
                y_mm: 50_000,
            },
            EntityId(4),
            WorldPosition {
                x_mm: 90_000,
                y_mm: 50_000,
            },
            100_000,
        );
        assert_eq!(blocked.result, TargetingResult::BlockedByObstacle);
        assert_eq!(
            blocked.line_of_fire.blocking_obstacle_id.as_deref(),
            Some("wall_mid")
        );

        let out_of_range = cover_map.evaluate_range_first_targeting(
            EntityId(1),
            WorldPosition {
                x_mm: 10_000,
                y_mm: 10_000,
            },
            EntityId(5),
            WorldPosition {
                x_mm: 90_000,
                y_mm: 90_000,
            },
            10_000,
        );
        assert_eq!(out_of_range.result, TargetingResult::OutOfRange);
        assert!(out_of_range.line_of_fire.clear);
        assert_eq!(out_of_range.line_of_fire.blocking_obstacle_id, None);
    }

    fn dense_positions(count: usize, x_start_mm: i32, y_start_mm: i32) -> Vec<WorldPosition> {
        (0..count)
            .map(|index| WorldPosition {
                x_mm: x_start_mm + i32::try_from(index % 16).unwrap_or(0) * 750,
                y_mm: y_start_mm + i32::try_from(index / 16).unwrap_or(0) * 750,
            })
            .collect()
    }

    #[test]
    fn cover_combat_perf_smoke_counts_dense_targeting_queries() {
        let cover_map = CoverCombatMap::from_map_data(&sample_map()).expect("cover map imports");
        let mut attackers = dense_positions(32, 10_000, 10_000);
        attackers.extend(dense_positions(32, 10_000, 50_000));
        let mut targets = dense_positions(64, 18_500, 18_500);
        targets.extend(dense_positions(16, 88_000, 50_000));
        targets.extend(dense_positions(16, 88_000, 88_000));

        let run = cover_map.run_dense_targeting_smoke(&attackers, &targets, 100_000);

        assert_eq!(run.query_count, 64 * 96);
        assert!(run.in_range_clear > 0);
        assert!(run.in_range_target_in_cover > 0);
        assert!(run.blocked_by_obstacle > 0);
        assert!(run.out_of_range > 0);
        assert_eq!(
            run.query_count,
            run.in_range_clear
                + run.in_range_target_in_cover
                + run.blocked_by_obstacle
                + run.out_of_range
        );
        assert_eq!(run.claim_scope, "informational_contract_only");
    }
}
