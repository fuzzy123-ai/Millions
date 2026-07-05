extends SceneTree

const ClientWorldState := preload("res://scripts/net/ClientWorldState.gd")
const RenderAdapter := preload("res://scripts/render/RenderAdapter.gd")
const GCoreLocalMatchSmoke := preload("res://scripts/gameplay/GCoreLocalMatchSmoke.gd")


func _init() -> void:
	var world_state := ClientWorldState.new()
	var render_adapter := RenderAdapter.new()
	root.add_child(world_state)
	root.add_child(render_adapter)

	var result: Dictionary = GCoreLocalMatchSmoke.run_smoke(world_state, render_adapter)
	if str(result.get("schema", "")) != "millions_gcore_local_match_smoke_v0":
		_fail("schema mismatch", result)
		return
	if int(result.get("client_count", 0)) != 2:
		_fail("client count mismatch", result)
		return
	if int(result.get("intent_count", 0)) != 2:
		_fail("intent count mismatch", result)
		return
	if not bool(result.get("intent_only", false)):
		_fail("intent-only boundary mismatch", result)
		return
	if int(result.get("hq_count", 0)) != 2:
		_fail("hq count mismatch", result)
		return
	if int(result.get("squad_proxy_count", 0)) != 8:
		_fail("squad proxy count mismatch", result)
		return
	if int(result.get("entity_count", 0)) != 10:
		_fail("entity count mismatch", result)
		return
	if int(result.get("render_record_count", 0)) != 10:
		_fail("render record count mismatch", result)
		return
	if int(result.get("render_batch_count", 0)) != 4:
		_fail("render batch count mismatch", result)
		return
	if str(result.get("claim_scope", "")) != "local_abstract_smoke_only":
		_fail("claim scope mismatch", result)
		return

	print("gcore_local_match_smoke status=ok clients=2 hqs=2 squad_proxies=8 entities=10 render_batches=4 intents=2")
	quit(0)


func _fail(reason: String, result: Dictionary) -> void:
	push_error("%s: %s" % [reason, result])
	quit(1)
