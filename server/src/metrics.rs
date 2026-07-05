use crate::simulation::Snapshot;

pub const SNAPSHOT_HEADER_BYTES: u64 = 24;
pub const SNAPSHOT_ENTITY_BYTES: u64 = 36;
pub const SNAPSHOT_REMOVED_ENTITY_BYTES: u64 = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerMetricSample {
    pub server_tick_ms: f64,
    pub snapshot_bytes: u64,
    pub bandwidth_kb_s_per_client: f64,
}

impl ServerMetricSample {
    pub fn from_snapshot(server_tick_ms: f64, snapshot: &Snapshot, elapsed_seconds: f64) -> Self {
        let snapshot_bytes = estimate_snapshot_bytes(snapshot);
        Self {
            server_tick_ms,
            snapshot_bytes,
            bandwidth_kb_s_per_client: bandwidth_kb_s(snapshot_bytes, elapsed_seconds),
        }
    }
}

pub fn estimate_snapshot_bytes(snapshot: &Snapshot) -> u64 {
    SNAPSHOT_HEADER_BYTES
        + snapshot.entities.len() as u64 * SNAPSHOT_ENTITY_BYTES
        + snapshot.removed_entities.len() as u64 * SNAPSHOT_REMOVED_ENTITY_BYTES
}

pub fn bandwidth_kb_s(bytes: u64, elapsed_seconds: f64) -> f64 {
    if elapsed_seconds <= 0.0 {
        return 0.0;
    }
    bytes as f64 / 1024.0 / elapsed_seconds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{EntityId, EntityState, SnapshotBuilder, Tick, WorldPosition};

    fn entity(entity_id: u64) -> EntityState {
        EntityState {
            entity_id: EntityId(entity_id),
            entity_kind: 1,
            faction_id: 1,
            flags: 0,
            position: WorldPosition { x_mm: 0, y_mm: 0 },
            facing_millirad: 0,
            health_q8: 256,
            state_id: 0,
            state_param_q8: 0,
        }
    }

    #[test]
    fn snapshot_size_counts_header_entities_and_removals() {
        let mut builder = SnapshotBuilder::delta(2, 1, Tick(20));
        builder.push_entity(entity(1));
        builder.push_entity(entity(2));
        builder.push_removed(EntityId(9));
        let snapshot = builder.build();

        assert_eq!(
            estimate_snapshot_bytes(&snapshot),
            SNAPSHOT_HEADER_BYTES + 2 * SNAPSHOT_ENTITY_BYTES + SNAPSHOT_REMOVED_ENTITY_BYTES
        );
    }

    #[test]
    fn bandwidth_is_kilobytes_per_second() {
        assert_eq!(bandwidth_kb_s(2048, 2.0), 1.0);
        assert_eq!(bandwidth_kb_s(2048, 0.0), 0.0);
    }

    #[test]
    fn metric_sample_records_tick_snapshot_and_bandwidth() {
        let mut builder = SnapshotBuilder::full(1, Tick(20));
        builder.push_entity(entity(1));
        let snapshot = builder.build();

        let sample = ServerMetricSample::from_snapshot(4.5, &snapshot, 0.5);

        assert_eq!(sample.server_tick_ms, 4.5);
        assert_eq!(
            sample.snapshot_bytes,
            SNAPSHOT_HEADER_BYTES + SNAPSHOT_ENTITY_BYTES
        );
        assert!(sample.bandwidth_kb_s_per_client > 0.0);
    }
}
