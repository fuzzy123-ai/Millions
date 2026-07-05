pub mod build_info;
pub mod collision;
pub mod cover;
pub mod determinism;
pub mod faction_scale;
pub mod fixtures;
pub mod game_core;
pub mod hardening;
pub mod interest;
pub mod load_shed;
pub mod map_data;
pub mod metrics;
pub mod movement_scale;
pub mod observability;
pub mod perf_budget;
pub mod perf_history;
pub mod protocol;
pub mod reconnect;
pub mod reliability;
pub mod replay;
pub mod simulation;
pub mod simulation_scale;
pub mod soak_metrics;
pub mod steam_bridge;
pub mod swarm;
pub mod transport;

pub use build_info::{BuildEvidence, REPLAY_FORMAT_VERSION};
pub use collision::{
    collision_perf_scenarios, run_collision_perf_smoke, CollisionAdmission,
    CollisionAdmissionResult, CollisionBatchMovementCandidate, CollisionBatchMovementProbeReport,
    CollisionBatchMovementSample, CollisionBody, CollisionBodyKind, CollisionContact,
    CollisionCorrection, CollisionError, CollisionMovementDecision, CollisionMovementProbe,
    CollisionPerfRun, CollisionPerfScenario, CollisionPerfScenarioId, CollisionPhysicsStep,
    CollisionProbe, CollisionResolutionPlan, CollisionResolvedAdmission,
    CollisionResolvedAdmissionResult, CollisionWorld, COLLISION_PERF_SCENARIOS, COLLISION_SCHEMA,
};
pub use cover::{
    AxisAlignedVolume, CoverCombatMap, CoverCombatPerfRun, CoverError, CoverObject, CoverOccupancy,
    LineQuery, TargetingQuery, TargetingResult, COVER_SCHEMA,
};
pub use determinism::{
    derive_simulation_seed, DeterministicInput, DeterministicInputFrame, SimulationSeed,
};
pub use faction_scale::{
    faction_scale_scenarios, FactionScaleScenario, FactionScaleScenarioId, RoleMixCounts,
    FACTION_SCALE_SCENARIOS,
};
pub use game_core::{
    AcceptedMoveIntent, GCoreCommandError, GCoreState, MoveIntent, PlayerStart, SpawnedSquad,
    BASIC_SQUAD_ENTITY_KIND, BASIC_SQUAD_MEMBER_COUNT, GCORE_SCHEMA, HQ_ENTITY_KIND,
};
pub use hardening::{
    evaluate_auth_state, evaluate_command_freshness, validate_header_payload_limits,
    validate_packet_bounds, AuthCheckState, CommandFreshness, ProtocolHardeningAction,
    ProtocolHardeningDecision, ProtocolHardeningLimits, ProtocolRejectReason,
    MAX_CLIENT_COMMAND_BATCH_BYTES, MAX_PACKET_BYTES, MAX_PAYLOAD_BYTES,
    MAX_REDACTED_DIAGNOSTIC_BYTES,
};
pub use interest::{
    aggregate_far_state, build_interest_snapshot_delta, build_visible_delta_snapshot,
    visible_entities_for_region, AggregateFarState, AoiRegion, ClientInterestState,
    InterestManager, InterestSnapshotDelta, InterestUpdate,
};
pub use load_shed::{
    apply_slow_client_policy, evaluate_load_shed, ClientLoadSample, CommandAdmission,
    LoadShedAction, LoadShedDecision, LoadShedLimits, LoadShedReason, SlowClientPolicyDecision,
    SnapshotDegradeMode,
};
pub use map_data::{
    map_data_fixture_checksum, validate_map_data_import, CapturePoint, MapBounds, MapDataImport,
    MapDataValidationError, MapPoint, MapShape, MapShapeKind, SpawnPoint,
    MAP_DATA_FIXTURE_CHECKSUM_ALGORITHM, MAP_DATA_SCHEMA, MAX_MAP_EXTENT_MM,
    MAX_MAP_MARKERS_PER_KIND, MIN_MAP_EXTENT_MM,
};
pub use metrics::{
    bandwidth_kb_s, estimate_snapshot_bytes, ServerMetricSample, SNAPSHOT_ENTITY_BYTES,
    SNAPSHOT_HEADER_BYTES, SNAPSHOT_REMOVED_ENTITY_BYTES,
};
pub use movement_scale::{
    movement_scale_scenarios, run_movement_scale_scenario, FlowFieldBounds, FlowFieldCache,
    FlowFieldCacheStats, FlowFieldError, FlowFieldMap, FlowFieldRequest, FlowFieldStep,
    FlowFieldStepResult, MovementOptionFamily, MovementScaleRun, MovementScaleScenario,
    MovementScaleScenarioId, MAPDATA_V0_LOCAL_FIXTURE_CHECKSUM, MOVEMENT_SCALE_SCENARIOS,
};
pub use observability::{CounterSample, ObservabilityCounter, ObservabilityCounters};
pub use perf_budget::{
    evaluate_server_perf_budget, BudgetResult, ServerPerfBudgets, ServerPerfReport,
    ServerPerfScenario,
};
pub use perf_history::{
    build_ledger_id, PerfHistoryBudgetResult, PerfHistoryClaimScope,
    PerfHistoryLocalElapsedMetrics, PerfHistoryMetricPercentiles, PerfHistoryMetrics,
    PerfHistoryRedactionStatus, PerfHistoryRow, PerfHistoryScenarioFamily, PerfHistoryStatus,
};
pub use protocol::{
    synthetic_session_id, MessageType, PacketDecodeError, PacketHeader, PROTOCOL_HEADER_LEN,
    PROTOCOL_HEADER_LEN_U16, PROTOCOL_MAGIC, PROTOCOL_VERSION,
};
pub use reconnect::{
    RebindResult, SessionConnectionState, SessionRebindState, SnapshotResumeAction,
    DEFAULT_RECONNECT_GRACE_TICKS,
};
pub use reliability::{
    ClientSequenceResult, CommandId, CommandIdempotencyResult, CommandReliabilityState,
    PendingReliablePacket,
};
pub use replay::{RecordedCommandStream, ReplayReport};
pub use simulation::{
    EntityColumns, EntityId, EntityState, MovementDelta, PlayerSessionId, Snapshot,
    SnapshotBuilder, SpatialCell, SpatialGrid, Tick, TickConfig, TickLoop, WorldPosition,
    SERVER_TICK_HZ, SERVER_TICK_MILLIS,
};
pub use simulation_scale::{
    run_simulation_scale_scenario, simulation_scale_scenarios, SimulationScaleRun,
    SimulationScaleScenario, SimulationScaleScenarioId, SIMULATION_SCALE_SCENARIOS,
};
pub use soak_metrics::{SoakMetric, SoakMetricSample, SoakMetrics, REQUIRED_SOAK_METRICS};
pub use steam_bridge::{
    accept_local_mock_handoff, validate_local_mock_handoff, AcceptedSteamBridgeSession,
    SteamBridgeError, SteamBridgeHandoff, LOCAL_DIRECT_SERVER_MODE, LOCAL_MOCK_IDENTITY_MODE,
    STEAM_BRIDGE_PROTOCOL, STEAM_BRIDGE_SCHEMA,
};
pub use swarm::{
    run_swarm_batch_vs_single_movement_loop_measurement,
    run_swarm_configured_movement_loop_measurement, run_swarm_load_smoke, AggroStimulus,
    AggroTrailLane, AggroTrailSample, RoutePressureSample, SwarmAiLod, SwarmAiLodCounts,
    SwarmBatchVsSingleMovementLoopMeasurementRun, SwarmBehaviorReport, SwarmBehaviorSample,
    SwarmConfig, SwarmConfiguredMovementLoopMeasurementRun, SwarmEntity, SwarmError,
    SwarmIntentKind, SwarmLoadSmokeRun, SwarmMovementApplyReport, SwarmMovementApplySample,
    SwarmMovementDeltaSnapshotReport, SwarmMovementMode, SwarmMovementPreviewReport,
    SwarmMovementPreviewSample, SwarmMovementTickReport, SwarmSpawnPoint, SwarmState,
    SwarmTickReport, SWARM_ENTITY_KIND, SWARM_SCHEMA,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn foundation_constants_match_decisions() {
        assert_eq!(SERVER_TICK_HZ, 20);
        assert_eq!(SERVER_TICK_MILLIS, 50);
        assert_eq!(PROTOCOL_VERSION, 0);
        assert_eq!(PROTOCOL_MAGIC, 0x4D4D);
        assert_eq!(PROTOCOL_HEADER_LEN, 48);
    }

    #[test]
    fn module_boundaries_are_addressable() {
        let connection = transport::ConnectionId(0x1122_3344_5566_7788);
        let session = PlayerSessionId(0x0102_0304_0506_0708);
        let envelope = transport::TransportEnvelope {
            connection_id: connection,
            session_id: Some(session),
            kind: transport::TransportKind::Loopback,
        };

        assert_eq!(envelope.connection_id, connection);
        assert_eq!(envelope.session_id, Some(session));
        assert_eq!(fixtures::FIXTURE_DIR, "../protocol/fixtures");
    }

    #[test]
    fn tick_loop_steps_monotonically_at_foundation_rate() {
        let mut tick_loop = TickLoop::foundation_default();

        assert_eq!(tick_loop.config(), TickConfig::default());
        assert_eq!(tick_loop.current_tick(), Tick(0));
        assert_eq!(tick_loop.step(), Tick(1));
        assert_eq!(tick_loop.step(), Tick(2));
        assert_eq!(tick_loop.step_n(18), Tick(20));
    }

    #[test]
    fn abstract_entity_state_moves_and_builds_full_snapshot() {
        let mut entity = EntityState {
            entity_id: EntityId(1),
            entity_kind: 1,
            faction_id: 1,
            flags: 1,
            position: WorldPosition {
                x_mm: 1000,
                y_mm: -2000,
            },
            facing_millirad: 1571,
            health_q8: 256,
            state_id: 1,
            state_param_q8: 0,
        };

        entity.apply_movement_stub(MovementDelta {
            dx_mm: 250,
            dy_mm: -500,
        });

        let mut builder = SnapshotBuilder::full(100, Tick(20));
        builder.push_entity(entity);
        let snapshot = builder.build();

        assert_eq!(snapshot.snapshot_id, 100);
        assert_eq!(snapshot.baseline_snapshot_id, 0);
        assert_eq!(snapshot.tick, Tick(20));
        assert_eq!(snapshot.entities.len(), 1);
        assert_eq!(snapshot.entities[0].position.x_mm, 1250);
        assert_eq!(snapshot.entities[0].position.y_mm, -2500);
        assert!(snapshot.removed_entities.is_empty());
    }

    #[test]
    fn delta_snapshot_records_removed_entities() {
        let mut builder = SnapshotBuilder::delta(101, 100, Tick(21));
        builder.push_removed(EntityId(9));
        let snapshot = builder.build();

        assert_eq!(snapshot.snapshot_id, 101);
        assert_eq!(snapshot.baseline_snapshot_id, 100);
        assert_eq!(snapshot.tick, Tick(21));
        assert!(snapshot.entities.is_empty());
        assert_eq!(snapshot.removed_entities, vec![EntityId(9)]);
    }

    #[test]
    fn synthetic_identity_handshake_is_stable_and_non_zero() {
        let first = synthetic_session_id("local-abc123:Player 1");
        let second = synthetic_session_id("local-abc123:Player 1");
        let other = synthetic_session_id("local-abc123:Player 2");

        assert_ne!(first, 0);
        assert_eq!(first, second);
        assert_ne!(first, other);
    }

    #[test]
    fn protocol_v0_server_hello_fixture_decodes() {
        let packet = include_bytes!("../../protocol/fixtures/protocol_v0_server_hello_accept.bin");
        let header = PacketHeader::decode(packet).expect("server hello fixture decodes");

        assert_eq!(header.message_type, MessageType::ServerHello);
        assert_eq!(header.payload_len, 8);
        assert_eq!(header.connection_id, 0x1122_3344_5566_7788);
        assert_eq!(header.session_id, 0x0102_0304_0506_0708);
        assert_eq!(header.client_seq, 0);
        assert_eq!(header.server_seq, 1);
        assert_eq!(header.ack_seq, 0);
        assert_eq!(header.tick, 0);
        assert_eq!(
            u16::from_le_bytes([packet[48], packet[49]]),
            SERVER_TICK_HZ as u16
        );
        assert_eq!(u16::from_le_bytes([packet[50], packet[51]]), 0);
        assert_eq!(
            u32::from_le_bytes([packet[52], packet[53], packet[54], packet[55]]),
            1200
        );
    }

    #[test]
    fn protocol_v0_command_ready_fixture_decodes() {
        let packet =
            include_bytes!("../../protocol/fixtures/protocol_v0_command_ready_batch_ok.bin");
        let header = PacketHeader::decode(packet).expect("command fixture decodes");

        assert_eq!(header.message_type, MessageType::ClientCommandBatch);
        assert_eq!(header.payload_len, 24);
        assert_eq!(header.connection_id, 0x1122_3344_5566_7788);
        assert_eq!(header.session_id, 0x0102_0304_0506_0708);
        assert_eq!(header.client_seq, 1);
        assert_eq!(header.server_seq, 0);
        assert_eq!(header.ack_seq, 1);
        assert_eq!(header.tick, 10);
        assert_eq!(u16::from_le_bytes([packet[48], packet[49]]), 1);
        assert_eq!(u64::from_le_bytes(packet[52..60].try_into().unwrap()), 42);
        assert_eq!(u16::from_le_bytes([packet[60], packet[61]]), 1);
        assert_eq!(u16::from_le_bytes([packet[62], packet[63]]), 0);
        assert_eq!(u64::from_le_bytes(packet[64..72].try_into().unwrap()), 10);
    }

    #[test]
    fn protocol_v0_full_snapshot_fixture_decodes() {
        let packet =
            include_bytes!("../../protocol/fixtures/protocol_v0_snapshot_full_minimal_ok.bin");
        let header = PacketHeader::decode(packet).expect("full snapshot fixture decodes");

        assert_eq!(header.message_type, MessageType::ServerFullSnapshot);
        assert_eq!(header.payload_len, 60);
        assert_eq!(header.connection_id, 0x1122_3344_5566_7788);
        assert_eq!(header.session_id, 0x0102_0304_0506_0708);
        assert_eq!(header.client_seq, 0);
        assert_eq!(header.server_seq, 2);
        assert_eq!(header.ack_seq, 1);
        assert_eq!(header.tick, 20);
        assert_eq!(u64::from_le_bytes(packet[48..56].try_into().unwrap()), 100);
        assert_eq!(u64::from_le_bytes(packet[56..64].try_into().unwrap()), 0);
        assert_eq!(u32::from_le_bytes(packet[64..68].try_into().unwrap()), 1);
        assert_eq!(u64::from_le_bytes(packet[72..80].try_into().unwrap()), 1);
        assert_eq!(i32::from_le_bytes(packet[88..92].try_into().unwrap()), 1000);
        assert_eq!(
            i32::from_le_bytes(packet[92..96].try_into().unwrap()),
            -2000
        );
        assert_eq!(u16::from_le_bytes([packet[100], packet[101]]), 256);
    }

    #[test]
    fn protocol_v0_fixture_descriptors_exist() {
        for path in fixtures::DESCRIPTOR_PATHS {
            assert!(
                Path::new(path).exists(),
                "missing fixture descriptor {path}"
            );
        }
    }

    #[test]
    fn protocol_v0_rejects_bad_magic_before_payload() {
        let mut packet =
            *include_bytes!("../../protocol/fixtures/protocol_v0_server_hello_accept.bin");
        packet[0] = 0;

        assert_eq!(
            PacketHeader::decode(&packet),
            Err(PacketDecodeError::BadMagic(0x4D00))
        );
    }
}
