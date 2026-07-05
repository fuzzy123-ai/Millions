extends SceneTree

const GCoreIntent := preload("res://scripts/gameplay/GCoreIntent.gd")


func _init() -> void:
	var move_intent: Dictionary = GCoreIntent.move_intent(42, 3, 50_000, 60_000)
	if int(move_intent.get("command_type", 0)) != GCoreIntent.COMMAND_TYPE_MOVE:
		_fail("move command type mismatch", move_intent)
		return
	if int(move_intent.get("command_id", 0)) != 42:
		_fail("move command id mismatch", move_intent)
		return
	if int(move_intent.get("squad_id", 0)) != 3:
		_fail("move squad id mismatch", move_intent)
		return
	if int(move_intent.get("target_x_mm", 0)) != 50_000:
		_fail("move target x mismatch", move_intent)
		return
	if not GCoreIntent.is_intent_only(move_intent):
		_fail("move intent authority boundary mismatch", move_intent)
		return

	var spawn_intent: Dictionary = GCoreIntent.spawn_basic_squad_intent(43, 101)
	if int(spawn_intent.get("command_type", 0)) != GCoreIntent.COMMAND_TYPE_SPAWN_BASIC_SQUAD:
		_fail("spawn command type mismatch", spawn_intent)
		return
	if int(spawn_intent.get("player_session_id", 0)) != 101:
		_fail("spawn player session mismatch", spawn_intent)
		return
	if not GCoreIntent.is_intent_only(spawn_intent):
		_fail("spawn intent authority boundary mismatch", spawn_intent)
		return

	print("gcore_intent_check status=ok move=ok spawn=ok authority=server")
	quit(0)


func _fail(reason: String, intent: Dictionary) -> void:
	push_error("%s: %s" % [reason, intent])
	quit(1)
