extends RefCounted

const GCoreIntent := preload("res://scripts/gameplay/GCoreIntent.gd")

const HQ_ENTITY_KIND := 100
const BASIC_SQUAD_ENTITY_KIND := 101
const PLAYER_ONE_SESSION_ID := 101
const PLAYER_TWO_SESSION_ID := 202


static func run_smoke(world_state: Node, render_adapter: Node) -> Dictionary:
	var move_one: Dictionary = GCoreIntent.move_intent(42, 3, 50_000, 50_000)
	var move_two: Dictionary = GCoreIntent.move_intent(43, 8, 55_000, 55_000)
	var intents: Array[Dictionary] = [move_one, move_two]

	var entities: Array[Dictionary] = _abstract_entities()
	world_state.apply_snapshot(
		{
			"tick": 20,
			"payload": {
				"entities": entities,
				"removed_entities": [],
			},
		}
	)
	render_adapter.update_from_world_state(world_state)

	return {
		"schema": "millions_gcore_local_match_smoke_v0",
		"client_count": 2,
		"intent_count": intents.size(),
		"intent_only": _all_intents_are_intent_only(intents),
		"entity_count": world_state.entity_count(),
		"render_record_count": render_adapter.snapshot_records().size(),
		"render_batch_count": render_adapter.batch_count(),
		"hq_count": _count_kind(entities, HQ_ENTITY_KIND),
		"squad_proxy_count": _count_kind(entities, BASIC_SQUAD_ENTITY_KIND),
		"claim_scope": "local_abstract_smoke_only",
	}


static func _abstract_entities() -> Array[Dictionary]:
	var entities: Array[Dictionary] = []
	entities.append(_entity(1, HQ_ENTITY_KIND, 1, 10_000, 10_000))
	entities.append(_entity(2, HQ_ENTITY_KIND, 2, 90_000, 90_000))

	var player_one_positions: Array[Vector2i] = [
		Vector2i(11_500, 11_500),
		Vector2i(12_500, 11_500),
		Vector2i(11_500, 12_500),
		Vector2i(12_500, 12_500),
	]
	var player_two_positions: Array[Vector2i] = [
		Vector2i(87_500, 87_500),
		Vector2i(88_500, 87_500),
		Vector2i(87_500, 88_500),
		Vector2i(88_500, 88_500),
	]

	for index: int in range(player_one_positions.size()):
		var position: Vector2i = player_one_positions[index]
		entities.append(_entity(4 + index, BASIC_SQUAD_ENTITY_KIND, 1, position.x, position.y))

	for index: int in range(player_two_positions.size()):
		var position: Vector2i = player_two_positions[index]
		entities.append(_entity(9 + index, BASIC_SQUAD_ENTITY_KIND, 2, position.x, position.y))

	return entities


static func _entity(entity_id: int, entity_kind: int, faction_id: int, x_mm: int, y_mm: int) -> Dictionary:
	return {
		"entity_id": entity_id,
		"entity_kind": entity_kind,
		"faction_id": faction_id,
		"x_mm": x_mm,
		"y_mm": y_mm,
		"health_q8": 256,
	}


static func _all_intents_are_intent_only(intents: Array[Dictionary]) -> bool:
	for intent: Dictionary in intents:
		if not GCoreIntent.is_intent_only(intent):
			return false
	return true


static func _count_kind(entities: Array[Dictionary], entity_kind: int) -> int:
	var count := 0
	for entity: Dictionary in entities:
		if int(entity.get("entity_kind", 0)) == entity_kind:
			count += 1
	return count
