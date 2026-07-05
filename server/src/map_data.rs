pub const MAP_DATA_SCHEMA: &str = "millions_mapdata_v0";
pub const MAP_DATA_FIXTURE_CHECKSUM_ALGORITHM: &str = "sum16_bytes";
pub const MIN_MAP_EXTENT_MM: i32 = 1000;
pub const MAX_MAP_EXTENT_MM: i32 = 1_000_000;
pub const MAX_MAP_MARKERS_PER_KIND: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDataImport {
    pub schema: &'static str,
    pub map_id: String,
    pub map_version: u32,
    pub checksum: String,
    pub bounds_mm: MapBounds,
    pub spawn_points: Vec<SpawnPoint>,
    pub capture_points: Vec<CapturePoint>,
    pub obstacles: Vec<MapShape>,
    pub cover_objects: Vec<MapShape>,
    pub navigation_hints: Vec<MapShape>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl MapBounds {
    pub fn contains_point(self, point: MapPoint) -> bool {
        point.x_mm >= self.min_x
            && point.x_mm <= self.max_x
            && point.y_mm >= self.min_y
            && point.y_mm <= self.max_y
    }

    pub fn width_mm(self) -> i32 {
        self.max_x.saturating_sub(self.min_x)
    }

    pub fn height_mm(self) -> i32 {
        self.max_y.saturating_sub(self.min_y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapPoint {
    pub x_mm: i32,
    pub y_mm: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnPoint {
    pub id: String,
    pub point: MapPoint,
    pub faction_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturePoint {
    pub id: String,
    pub center: MapPoint,
    pub radius_mm: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapShapeKind {
    Obstacle,
    Cover,
    NavigationHint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapShape {
    pub id: String,
    pub kind: MapShapeKind,
    pub center: MapPoint,
    pub half_extents_mm: MapPoint,
    pub class_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapDataValidationError {
    UnsupportedSchema,
    MissingMapId,
    MissingChecksum,
    InvalidBounds,
    TooManyMarkers,
    DuplicateId,
    PointOutOfBounds,
    InvalidRadius,
    InvalidShape,
    WrongShapeKind,
}

impl MapDataValidationError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSchema => "unsupported_schema",
            Self::MissingMapId => "missing_map_id",
            Self::MissingChecksum => "missing_checksum",
            Self::InvalidBounds => "invalid_bounds",
            Self::TooManyMarkers => "too_many_markers",
            Self::DuplicateId => "duplicate_id",
            Self::PointOutOfBounds => "point_out_of_bounds",
            Self::InvalidRadius => "invalid_radius",
            Self::InvalidShape => "invalid_shape",
            Self::WrongShapeKind => "wrong_shape_kind",
        }
    }
}

pub fn validate_map_data_import(map: &MapDataImport) -> Result<(), MapDataValidationError> {
    if map.schema != MAP_DATA_SCHEMA {
        return Err(MapDataValidationError::UnsupportedSchema);
    }
    if map.map_id.trim().is_empty() {
        return Err(MapDataValidationError::MissingMapId);
    }
    if map.checksum.trim().is_empty() {
        return Err(MapDataValidationError::MissingChecksum);
    }

    validate_bounds(map.bounds_mm)?;
    validate_count(map.spawn_points.len())?;
    validate_count(map.capture_points.len())?;
    validate_count(map.obstacles.len())?;
    validate_count(map.cover_objects.len())?;
    validate_count(map.navigation_hints.len())?;

    let mut ids = Vec::new();
    for spawn in &map.spawn_points {
        validate_id(&spawn.id, &mut ids)?;
        validate_point(map.bounds_mm, spawn.point)?;
    }
    for capture in &map.capture_points {
        validate_id(&capture.id, &mut ids)?;
        validate_point(map.bounds_mm, capture.center)?;
        if capture.radius_mm == 0 {
            return Err(MapDataValidationError::InvalidRadius);
        }
    }
    for shape in &map.obstacles {
        validate_shape(map.bounds_mm, shape, MapShapeKind::Obstacle, &mut ids)?;
    }
    for shape in &map.cover_objects {
        validate_shape(map.bounds_mm, shape, MapShapeKind::Cover, &mut ids)?;
    }
    for shape in &map.navigation_hints {
        validate_shape(map.bounds_mm, shape, MapShapeKind::NavigationHint, &mut ids)?;
    }

    Ok(())
}

pub fn map_data_fixture_checksum(bytes: &[u8]) -> String {
    let sum = bytes
        .iter()
        .fold(0u32, |acc, byte| (acc + u32::from(*byte)) % 65_535);
    format!("sum16:{sum:04x}")
}

fn validate_bounds(bounds: MapBounds) -> Result<(), MapDataValidationError> {
    let width = bounds.width_mm();
    let height = bounds.height_mm();
    if width < MIN_MAP_EXTENT_MM
        || height < MIN_MAP_EXTENT_MM
        || width > MAX_MAP_EXTENT_MM
        || height > MAX_MAP_EXTENT_MM
    {
        return Err(MapDataValidationError::InvalidBounds);
    }
    Ok(())
}

fn validate_count(count: usize) -> Result<(), MapDataValidationError> {
    if count > MAX_MAP_MARKERS_PER_KIND {
        return Err(MapDataValidationError::TooManyMarkers);
    }
    Ok(())
}

fn validate_id(id: &str, ids: &mut Vec<String>) -> Result<(), MapDataValidationError> {
    if id.trim().is_empty() {
        return Err(MapDataValidationError::DuplicateId);
    }
    if ids.iter().any(|existing| existing == id) {
        return Err(MapDataValidationError::DuplicateId);
    }
    ids.push(id.to_string());
    Ok(())
}

fn validate_point(bounds: MapBounds, point: MapPoint) -> Result<(), MapDataValidationError> {
    if !bounds.contains_point(point) {
        return Err(MapDataValidationError::PointOutOfBounds);
    }
    Ok(())
}

fn validate_shape(
    bounds: MapBounds,
    shape: &MapShape,
    expected_kind: MapShapeKind,
    ids: &mut Vec<String>,
) -> Result<(), MapDataValidationError> {
    validate_id(&shape.id, ids)?;
    if shape.kind != expected_kind {
        return Err(MapDataValidationError::WrongShapeKind);
    }
    validate_point(bounds, shape.center)?;
    if shape.half_extents_mm.x_mm <= 0 || shape.half_extents_mm.y_mm <= 0 {
        return Err(MapDataValidationError::InvalidShape);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> MapDataImport {
        MapDataImport {
            schema: MAP_DATA_SCHEMA,
            map_id: "local-dev-map".to_string(),
            map_version: 1,
            checksum: "sum16:fixture-sidecar".to_string(),
            bounds_mm: MapBounds {
                min_x: 0,
                min_y: 0,
                max_x: 100_000,
                max_y: 100_000,
            },
            spawn_points: vec![SpawnPoint {
                id: "spawn_a".to_string(),
                point: MapPoint {
                    x_mm: 10_000,
                    y_mm: 10_000,
                },
                faction_label: "local_a".to_string(),
            }],
            capture_points: vec![CapturePoint {
                id: "cap_center".to_string(),
                center: MapPoint {
                    x_mm: 50_000,
                    y_mm: 50_000,
                },
                radius_mm: 5000,
            }],
            obstacles: vec![MapShape {
                id: "obstacle_1".to_string(),
                kind: MapShapeKind::Obstacle,
                center: MapPoint {
                    x_mm: 20_000,
                    y_mm: 20_000,
                },
                half_extents_mm: MapPoint {
                    x_mm: 1000,
                    y_mm: 1500,
                },
                class_label: "blocker".to_string(),
            }],
            cover_objects: vec![MapShape {
                id: "cover_1".to_string(),
                kind: MapShapeKind::Cover,
                center: MapPoint {
                    x_mm: 30_000,
                    y_mm: 30_000,
                },
                half_extents_mm: MapPoint {
                    x_mm: 800,
                    y_mm: 400,
                },
                class_label: "low_cover".to_string(),
            }],
            navigation_hints: vec![MapShape {
                id: "nav_hint_1".to_string(),
                kind: MapShapeKind::NavigationHint,
                center: MapPoint {
                    x_mm: 40_000,
                    y_mm: 40_000,
                },
                half_extents_mm: MapPoint {
                    x_mm: 2000,
                    y_mm: 2000,
                },
                class_label: "blocked_cell_hint".to_string(),
            }],
        }
    }

    #[test]
    fn map_data_import_accepts_valid_local_contract() {
        assert_eq!(validate_map_data_import(&sample_map()), Ok(()));
    }

    #[test]
    fn map_data_import_rejects_wrong_schema() {
        let mut map = sample_map();
        map.schema = "other_schema";

        assert_eq!(
            validate_map_data_import(&map),
            Err(MapDataValidationError::UnsupportedSchema)
        );
        assert_eq!(
            MapDataValidationError::UnsupportedSchema.as_str(),
            "unsupported_schema"
        );
    }

    #[test]
    fn map_data_import_rejects_duplicate_ids_across_marker_kinds() {
        let mut map = sample_map();
        map.capture_points[0].id = "spawn_a".to_string();

        assert_eq!(
            validate_map_data_import(&map),
            Err(MapDataValidationError::DuplicateId)
        );
    }

    #[test]
    fn map_data_import_rejects_out_of_bounds_points() {
        let mut map = sample_map();
        map.spawn_points[0].point.x_mm = 200_000;

        assert_eq!(
            validate_map_data_import(&map),
            Err(MapDataValidationError::PointOutOfBounds)
        );
    }

    #[test]
    fn map_data_import_rejects_zero_capture_radius() {
        let mut map = sample_map();
        map.capture_points[0].radius_mm = 0;

        assert_eq!(
            validate_map_data_import(&map),
            Err(MapDataValidationError::InvalidRadius)
        );
    }

    #[test]
    fn map_data_import_rejects_wrong_shape_kind() {
        let mut map = sample_map();
        map.obstacles[0].kind = MapShapeKind::Cover;

        assert_eq!(
            validate_map_data_import(&map),
            Err(MapDataValidationError::WrongShapeKind)
        );
    }

    #[test]
    fn map_data_fixture_checksum_matches_sidecar() {
        let fixture = include_bytes!("../../tests/fixtures/mapdata_v0_local_contract.json");
        let sidecar = include_str!("../../tests/fixtures/mapdata_v0_local_contract.checksum.json");

        assert_eq!(MAP_DATA_FIXTURE_CHECKSUM_ALGORITHM, "sum16_bytes");
        assert_eq!(
            map_data_fixture_checksum(fixture),
            extract_json_string_value(sidecar, "expected_checksum")
                .expect("sidecar includes expected_checksum")
        );
    }

    #[test]
    fn map_data_fixture_declares_non_live_non_gameplay_claims() {
        let fixture = include_str!("../../tests/fixtures/mapdata_v0_local_contract.json");

        assert!(fixture.contains("\"schema\": \"millions_mapdata_v0\""));
        assert!(fixture.contains("\"not_gameplay_authority\""));
        assert!(fixture.contains("\"not_live\""));
        assert!(fixture.contains("\"not_release_candidate\""));
        assert!(!fixture.to_ascii_lowercase().contains("steam"));
        assert!(!fixture.to_ascii_lowercase().contains("ticket"));
    }

    fn extract_json_string_value(document: &str, key: &str) -> Option<String> {
        let quoted_key = format!("\"{key}\"");
        let key_start = document.find(&quoted_key)?;
        let after_key = &document[key_start + quoted_key.len()..];
        let colon = after_key.find(':')?;
        let after_colon = after_key[colon + 1..].trim_start();
        if !after_colon.starts_with('"') {
            return None;
        }
        let value_start = 1;
        let value_end = after_colon[value_start..].find('"')? + value_start;
        Some(after_colon[value_start..value_end].to_string())
    }
}
