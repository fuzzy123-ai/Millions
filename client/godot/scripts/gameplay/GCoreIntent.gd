extends RefCounted

const SCHEMA := "millions_gcore_intent_v0"
const AUTHORITY := "server"
const COMMAND_TYPE_MOVE := 3
const COMMAND_TYPE_SPAWN_BASIC_SQUAD := 6


static func move_intent(command_id: int, squad_id: int, target_x_mm: int, target_y_mm: int) -> Dictionary:
	return {
		"schema": SCHEMA,
		"authority": AUTHORITY,
		"command_type": COMMAND_TYPE_MOVE,
		"command_id": command_id,
		"squad_id": squad_id,
		"target_x_mm": target_x_mm,
		"target_y_mm": target_y_mm,
	}


static func spawn_basic_squad_intent(command_id: int, player_session_id: int) -> Dictionary:
	return {
		"schema": SCHEMA,
		"authority": AUTHORITY,
		"command_type": COMMAND_TYPE_SPAWN_BASIC_SQUAD,
		"command_id": command_id,
		"player_session_id": player_session_id,
	}


static func is_intent_only(intent: Dictionary) -> bool:
	return (
		str(intent.get("schema", "")) == SCHEMA
		and str(intent.get("authority", "")) == AUTHORITY
		and not intent.has("accepted")
		and not intent.has("authoritative_position")
		and not intent.has("server_tick")
	)
