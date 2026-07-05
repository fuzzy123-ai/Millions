use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CommandId, EntityId, EntityState, MapBounds, MapPoint, PlayerSessionId, Snapshot,
    SnapshotBuilder, Tick, WorldPosition,
};

pub const GCORE_SCHEMA: &str = "millions_gcore_v0";
pub const HQ_ENTITY_KIND: u16 = 100;
pub const BASIC_SQUAD_ENTITY_KIND: u16 = 101;
pub const BASIC_SQUAD_MEMBER_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStart {
    pub session_id: PlayerSessionId,
    pub faction_id: u16,
    pub hq_position: WorldPosition,
    pub squad_spawn_position: WorldPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveIntent {
    pub command_id: CommandId,
    pub squad_id: EntityId,
    pub target_position: WorldPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnedSquad {
    pub squad_id: EntityId,
    pub owner_session_id: PlayerSessionId,
    pub member_entity_ids: [EntityId; BASIC_SQUAD_MEMBER_COUNT],
    pub spawn_position: WorldPosition,
    pub target_position: Option<WorldPosition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedMoveIntent {
    pub command_id: CommandId,
    pub squad_id: EntityId,
    pub target_position: WorldPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GCoreCommandError {
    PlayerAlreadyStarted,
    PlayerMissing,
    SquadAlreadySpawned,
    SquadMissing,
    WrongOwner,
    PositionOutOfBounds,
    DuplicateCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CommandKey {
    session_id: PlayerSessionId,
    command_id: CommandId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GCoreState {
    bounds: MapBounds,
    next_entity_id: u64,
    player_starts: BTreeMap<PlayerSessionId, PlayerStart>,
    hq_entities: BTreeMap<PlayerSessionId, EntityId>,
    owner_squads: BTreeMap<PlayerSessionId, EntityId>,
    squads: BTreeMap<EntityId, SpawnedSquad>,
    accepted_commands: BTreeSet<CommandKey>,
}

impl GCoreState {
    pub fn new(bounds: MapBounds) -> Self {
        Self {
            bounds,
            next_entity_id: 1,
            player_starts: BTreeMap::new(),
            hq_entities: BTreeMap::new(),
            owner_squads: BTreeMap::new(),
            squads: BTreeMap::new(),
            accepted_commands: BTreeSet::new(),
        }
    }

    pub fn bounds(&self) -> MapBounds {
        self.bounds
    }

    pub fn add_player_start(&mut self, start: PlayerStart) -> Result<EntityId, GCoreCommandError> {
        if self.player_starts.contains_key(&start.session_id) {
            return Err(GCoreCommandError::PlayerAlreadyStarted);
        }
        if !self.contains_position(start.hq_position)
            || !self.contains_position(start.squad_spawn_position)
        {
            return Err(GCoreCommandError::PositionOutOfBounds);
        }

        let hq_entity_id = self.allocate_entity_id();
        self.player_starts.insert(start.session_id, start);
        self.hq_entities.insert(start.session_id, hq_entity_id);
        Ok(hq_entity_id)
    }

    pub fn spawn_basic_squad(
        &mut self,
        session_id: PlayerSessionId,
    ) -> Result<SpawnedSquad, GCoreCommandError> {
        let start = *self
            .player_starts
            .get(&session_id)
            .ok_or(GCoreCommandError::PlayerMissing)?;

        if self.owner_squads.contains_key(&session_id) {
            return Err(GCoreCommandError::SquadAlreadySpawned);
        }

        let squad_id = self.allocate_entity_id();
        let member_entity_ids = [
            self.allocate_entity_id(),
            self.allocate_entity_id(),
            self.allocate_entity_id(),
            self.allocate_entity_id(),
        ];
        let squad = SpawnedSquad {
            squad_id,
            owner_session_id: session_id,
            member_entity_ids,
            spawn_position: start.squad_spawn_position,
            target_position: None,
        };

        self.owner_squads.insert(session_id, squad_id);
        self.squads.insert(squad_id, squad.clone());
        Ok(squad)
    }

    pub fn apply_move_intent(
        &mut self,
        session_id: PlayerSessionId,
        intent: MoveIntent,
    ) -> Result<AcceptedMoveIntent, GCoreCommandError> {
        let command_key = CommandKey {
            session_id,
            command_id: intent.command_id,
        };
        if self.accepted_commands.contains(&command_key) {
            return Err(GCoreCommandError::DuplicateCommand);
        }
        if !self.contains_position(intent.target_position) {
            return Err(GCoreCommandError::PositionOutOfBounds);
        }

        let squad = self
            .squads
            .get_mut(&intent.squad_id)
            .ok_or(GCoreCommandError::SquadMissing)?;
        if squad.owner_session_id != session_id {
            return Err(GCoreCommandError::WrongOwner);
        }

        squad.target_position = Some(intent.target_position);
        self.accepted_commands.insert(command_key);
        Ok(AcceptedMoveIntent {
            command_id: intent.command_id,
            squad_id: intent.squad_id,
            target_position: intent.target_position,
        })
    }

    pub fn player_count(&self) -> usize {
        self.player_starts.len()
    }

    pub fn squad_count(&self) -> usize {
        self.squads.len()
    }

    pub fn squad(&self, squad_id: EntityId) -> Option<&SpawnedSquad> {
        self.squads.get(&squad_id)
    }

    pub fn snapshot_entities(&self) -> Vec<EntityState> {
        let mut entities = Vec::new();
        for (session_id, start) in &self.player_starts {
            if let Some(hq_entity_id) = self.hq_entities.get(session_id) {
                entities.push(EntityState {
                    entity_id: *hq_entity_id,
                    entity_kind: HQ_ENTITY_KIND,
                    faction_id: start.faction_id,
                    flags: 1,
                    position: start.hq_position,
                    facing_millirad: 0,
                    health_q8: 256,
                    state_id: 1,
                    state_param_q8: 0,
                });
            }
        }
        for squad in self.squads.values() {
            let faction_id = self
                .player_starts
                .get(&squad.owner_session_id)
                .map(|start| start.faction_id)
                .unwrap_or(0);
            for (index, entity_id) in squad.member_entity_ids.iter().enumerate() {
                entities.push(EntityState {
                    entity_id: *entity_id,
                    entity_kind: BASIC_SQUAD_ENTITY_KIND,
                    faction_id,
                    flags: 1,
                    position: offset_member_position(squad.spawn_position, index),
                    facing_millirad: 0,
                    health_q8: 256,
                    state_id: 1,
                    state_param_q8: 0,
                });
            }
        }
        entities
    }

    pub fn build_full_snapshot(&self, snapshot_id: u64, tick: Tick) -> Snapshot {
        let mut builder = SnapshotBuilder::full(snapshot_id, tick);
        for entity in self.snapshot_entities() {
            builder.push_entity(entity);
        }
        builder.build()
    }

    fn allocate_entity_id(&mut self) -> EntityId {
        let entity_id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        entity_id
    }

    fn contains_position(&self, position: WorldPosition) -> bool {
        self.bounds.contains_point(MapPoint {
            x_mm: position.x_mm,
            y_mm: position.y_mm,
        })
    }
}

fn offset_member_position(origin: WorldPosition, index: usize) -> WorldPosition {
    const OFFSETS_MM: [(i32, i32); BASIC_SQUAD_MEMBER_COUNT] =
        [(-500, -500), (500, -500), (-500, 500), (500, 500)];
    let (dx_mm, dy_mm) = OFFSETS_MM[index % BASIC_SQUAD_MEMBER_COUNT];
    WorldPosition {
        x_mm: origin.x_mm.saturating_add(dx_mm),
        y_mm: origin.y_mm.saturating_add(dy_mm),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> MapBounds {
        MapBounds {
            min_x: 0,
            min_y: 0,
            max_x: 100_000,
            max_y: 100_000,
        }
    }

    fn player_one_start() -> PlayerStart {
        PlayerStart {
            session_id: PlayerSessionId(101),
            faction_id: 1,
            hq_position: WorldPosition {
                x_mm: 10_000,
                y_mm: 10_000,
            },
            squad_spawn_position: WorldPosition {
                x_mm: 12_000,
                y_mm: 12_000,
            },
        }
    }

    fn player_two_start() -> PlayerStart {
        PlayerStart {
            session_id: PlayerSessionId(202),
            faction_id: 2,
            hq_position: WorldPosition {
                x_mm: 90_000,
                y_mm: 90_000,
            },
            squad_spawn_position: WorldPosition {
                x_mm: 88_000,
                y_mm: 88_000,
            },
        }
    }

    #[test]
    fn gcore_adds_authoritative_player_hqs_and_squads() {
        let mut state = GCoreState::new(bounds());

        let hq_one = state
            .add_player_start(player_one_start())
            .expect("player one hq accepted");
        let hq_two = state
            .add_player_start(player_two_start())
            .expect("player two hq accepted");
        let squad_one = state
            .spawn_basic_squad(PlayerSessionId(101))
            .expect("player one squad accepted");
        let squad_two = state
            .spawn_basic_squad(PlayerSessionId(202))
            .expect("player two squad accepted");

        assert_eq!(GCORE_SCHEMA, "millions_gcore_v0");
        assert_eq!(hq_one, EntityId(1));
        assert_eq!(hq_two, EntityId(2));
        assert_eq!(squad_one.squad_id, EntityId(3));
        assert_eq!(
            squad_one.member_entity_ids,
            [EntityId(4), EntityId(5), EntityId(6), EntityId(7)]
        );
        assert_eq!(squad_two.squad_id, EntityId(8));
        assert_eq!(state.player_count(), 2);
        assert_eq!(state.squad_count(), 2);
    }

    #[test]
    fn gcore_move_intent_is_server_validated_and_idempotent() {
        let mut state = GCoreState::new(bounds());
        state
            .add_player_start(player_one_start())
            .expect("player start accepted");
        let squad = state
            .spawn_basic_squad(PlayerSessionId(101))
            .expect("squad accepted");
        let intent = MoveIntent {
            command_id: CommandId(42),
            squad_id: squad.squad_id,
            target_position: WorldPosition {
                x_mm: 50_000,
                y_mm: 50_000,
            },
        };

        let accepted = state
            .apply_move_intent(PlayerSessionId(101), intent)
            .expect("move intent accepted");
        assert_eq!(accepted.command_id, CommandId(42));
        assert_eq!(
            state.squad(squad.squad_id).and_then(|s| s.target_position),
            Some(WorldPosition {
                x_mm: 50_000,
                y_mm: 50_000,
            })
        );
        assert_eq!(
            state.apply_move_intent(PlayerSessionId(101), intent),
            Err(GCoreCommandError::DuplicateCommand)
        );
    }

    #[test]
    fn gcore_rejects_wrong_owner_and_out_of_bounds_moves() {
        let mut state = GCoreState::new(bounds());
        state
            .add_player_start(player_one_start())
            .expect("player one accepted");
        state
            .add_player_start(player_two_start())
            .expect("player two accepted");
        let squad = state
            .spawn_basic_squad(PlayerSessionId(101))
            .expect("squad accepted");

        assert_eq!(
            state.apply_move_intent(
                PlayerSessionId(202),
                MoveIntent {
                    command_id: CommandId(7),
                    squad_id: squad.squad_id,
                    target_position: WorldPosition {
                        x_mm: 20_000,
                        y_mm: 20_000,
                    },
                },
            ),
            Err(GCoreCommandError::WrongOwner)
        );
        assert_eq!(
            state.apply_move_intent(
                PlayerSessionId(101),
                MoveIntent {
                    command_id: CommandId(8),
                    squad_id: squad.squad_id,
                    target_position: WorldPosition {
                        x_mm: 120_000,
                        y_mm: 20_000,
                    },
                },
            ),
            Err(GCoreCommandError::PositionOutOfBounds)
        );
    }

    #[test]
    fn gcore_full_snapshot_contains_hqs_and_squad_members() {
        let mut state = GCoreState::new(bounds());
        state
            .add_player_start(player_one_start())
            .expect("player one accepted");
        state
            .add_player_start(player_two_start())
            .expect("player two accepted");
        state
            .spawn_basic_squad(PlayerSessionId(101))
            .expect("player one squad accepted");
        state
            .spawn_basic_squad(PlayerSessionId(202))
            .expect("player two squad accepted");

        let snapshot = state.build_full_snapshot(500, Tick(20));

        assert_eq!(snapshot.snapshot_id, 500);
        assert_eq!(snapshot.tick, Tick(20));
        assert_eq!(snapshot.entities.len(), 10);
        assert_eq!(snapshot.entities[0].entity_kind, HQ_ENTITY_KIND);
        assert_eq!(snapshot.entities[2].entity_kind, BASIC_SQUAD_ENTITY_KIND);
        assert_eq!(snapshot.entities[2].position.x_mm, 11_500);
        assert!(snapshot.removed_entities.is_empty());
    }
}
