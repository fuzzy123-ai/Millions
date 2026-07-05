extends SceneTree

const MovementReadabilityStress := preload("res://scripts/render/MovementReadabilityStress.gd")
const EXPECTED_PROXY_COUNT := 1000
const EXPECTED_GROUP_COUNT := 20
const EXPECTED_LANE_COUNT := 10


func _init() -> void:
	var stress_node: Node2D = MovementReadabilityStress.new()
	root.add_child(stress_node)
	await process_frame

	var result: Dictionary = stress_node.run_stress(
		EXPECTED_PROXY_COUNT,
		EXPECTED_GROUP_COUNT,
		EXPECTED_LANE_COUNT
	)

	if int(result.get("expected_proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		_fail("expected proxy count mismatch", result)
		return
	if int(result.get("proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		_fail("proxy count mismatch", result)
		return
	if int(result.get("group_count", -1)) != EXPECTED_GROUP_COUNT:
		_fail("group count mismatch", result)
		return
	if int(result.get("lane_count", -1)) != EXPECTED_LANE_COUNT:
		_fail("lane count mismatch", result)
		return
	if int(result.get("groups_with_members", -1)) != EXPECTED_GROUP_COUNT:
		_fail("groups with members mismatch", result)
		return
	if int(result.get("occupied_readability_cells", 0)) <= EXPECTED_LANE_COUNT:
		_fail("occupied readability cells too low", result)
		return
	if int(result.get("max_entities_per_readability_cell", 0)) <= 0:
		_fail("max entities per readability cell mismatch", result)
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

	print("movement_readability_stress status=ok proxies=%d groups=%d cells=%d max_per_cell=%d" % [
		int(result.get("proxy_count", 0)),
		int(result.get("groups_with_members", 0)),
		int(result.get("occupied_readability_cells", 0)),
		int(result.get("max_entities_per_readability_cell", 0)),
	])
	quit(0)


func _fail(reason: String, result: Dictionary) -> void:
	push_error("%s: %s" % [reason, result])
	quit(1)
