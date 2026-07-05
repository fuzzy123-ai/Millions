extends SceneTree

const SwarmReadabilityStress := preload("res://scripts/render/SwarmReadabilityStress.gd")
const EXPECTED_PROXY_COUNT := 1000


func _init() -> void:
	var stress_node: Node2D = SwarmReadabilityStress.new()
	root.add_child(stress_node)
	await process_frame

	var result: Dictionary = stress_node.run_stress(EXPECTED_PROXY_COUNT)
	if int(result.get("expected_proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		_fail("expected proxy count mismatch", result)
		return
	if int(result.get("proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		_fail("proxy count mismatch", result)
		return
	if int(result.get("full_lod_count", 0)) <= 0:
		_fail("full lod missing", result)
		return
	if int(result.get("reduced_lod_count", 0)) <= 0:
		_fail("reduced lod missing", result)
		return
	if int(result.get("aggregate_lod_count", 0)) <= 0:
		_fail("aggregate lod missing", result)
		return
	if int(result.get("direct_aggro_count", 0)) <= 0:
		_fail("direct aggro lane missing", result)
		return
	if int(result.get("memory_aggro_count", 0)) <= 0:
		_fail("memory aggro lane missing", result)
		return
	if int(result.get("route_intent_count", 0)) <= 0:
		_fail("route intent lane missing", result)
		return
	if int(result.get("collision_radius_count", 0)) != EXPECTED_PROXY_COUNT:
		_fail("collision radius metadata mismatch", result)
		return
	if int(result.get("occupied_readability_cells", 0)) <= 16:
		_fail("occupied readability cells too low", result)
		return
	if int(result.get("max_swarm_per_readability_cell", 0)) <= 0:
		_fail("max swarm per cell mismatch", result)
		return
	if str(result.get("claim_scope", "")) != "informational_contract_only":
		_fail("claim scope mismatch", result)
		return
	if float(result.get("bounds_max_x", 0.0)) <= float(result.get("bounds_min_x", 0.0)):
		_fail("horizontal bounds mismatch", result)
		return
	if float(result.get("bounds_max_y", 0.0)) <= float(result.get("bounds_min_y", 0.0)):
		_fail("vertical bounds mismatch", result)
		return

	print("swarm_readability_stress status=ok proxies=%d full=%d reduced=%d aggregate=%d cells=%d aggro=direct,memory,route collision_radius=ok" % [
		int(result.get("proxy_count", 0)),
		int(result.get("full_lod_count", 0)),
		int(result.get("reduced_lod_count", 0)),
		int(result.get("aggregate_lod_count", 0)),
		int(result.get("occupied_readability_cells", 0)),
	])
	quit(0)


func _fail(reason: String, result: Dictionary) -> void:
	push_error("%s: %s" % [reason, result])
	quit(1)
