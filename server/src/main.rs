fn main() {
    if std::env::args().any(|arg| arg == "--smoke") {
        run_smoke();
        return;
    }

    let tick_loop = millions_server::TickLoop::foundation_default();
    println!(
        "millions-server foundation: protocol_v{}, {} Hz tick target, {} ms step, current tick {}",
        millions_server::PROTOCOL_VERSION,
        millions_server::SERVER_TICK_HZ,
        tick_loop.config().tick_millis,
        tick_loop.current_tick().0
    );
}

fn run_smoke() {
    let mut tick_loop = millions_server::TickLoop::foundation_default();
    let tick = tick_loop.step_n(3);
    let mut entity = millions_server::EntityState {
        entity_id: millions_server::EntityId(1),
        entity_kind: 1,
        faction_id: 1,
        flags: 1,
        position: millions_server::WorldPosition { x_mm: 0, y_mm: 0 },
        facing_millirad: 0,
        health_q8: 256,
        state_id: 1,
        state_param_q8: 0,
    };
    entity.apply_movement_stub(millions_server::MovementDelta {
        dx_mm: 100,
        dy_mm: 50,
    });

    let mut snapshot_builder = millions_server::SnapshotBuilder::full(1, tick);
    snapshot_builder.push_entity(entity);
    let snapshot = snapshot_builder.build();

    println!(
        "event=server_smoke status=ok protocol_version={} tick_hz={} tick={} snapshot_id={} entities={} x_mm={} y_mm={}",
        millions_server::PROTOCOL_VERSION,
        millions_server::SERVER_TICK_HZ,
        snapshot.tick.0,
        snapshot.snapshot_id,
        snapshot.entities.len(),
        snapshot.entities[0].position.x_mm,
        snapshot.entities[0].position.y_mm
    );
}
