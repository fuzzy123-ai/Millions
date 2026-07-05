extends SceneTree

const SelectionReadabilityStress := preload("res://scripts/render/SelectionReadabilityStress.gd")
const EXPECTED_PROXY_COUNT := 1000
const EXPECTED_SELECTED_COUNT := 128


func _init() -> void:
	var stress_node: Node2D = SelectionReadabilityStress.new()
	root.add_child(stress_node)
	await process_frame

	var result: Dictionary = stress_node.run_stress(EXPECTED_PROXY_COUNT, EXPECTED_SELECTED_COUNT)
	if int(result.get("expected_proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		_fail("expected proxy count mismatch", result)
		return
	if int(result.get("proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		_fail("proxy count mismatch", result)
		return
	if int(result.get("expected_selected_count", -1)) != EXPECTED_SELECTED_COUNT:
		_fail("expected selected count mismatch", result)
		return
	if int(result.get("selection_overlay_count", -1)) != EXPECTED_SELECTED_COUNT:
		_fail("selection overlay count mismatch", result)
		return
	if int(result.get("selection_feedback_count", -1)) != EXPECTED_SELECTED_COUNT:
		_fail("selection feedback count mismatch", result)
		return
	if int(result.get("command_context_ready_count", -1)) != EXPECTED_SELECTED_COUNT:
		_fail("command context ready count mismatch", result)
		return
	if int(result.get("occupied_readability_cells", 0)) <= 8:
		_fail("occupied readability cells too low", result)
		return
	if int(result.get("max_overlays_per_readability_cell", 0)) <= 0:
		_fail("max overlays per cell mismatch", result)
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

	print("selection_readability_stress status=ok proxies=%d selected=%d cells=%d max_per_cell=%d feedback=selection_local,command_context_ready" % [
		int(result.get("proxy_count", 0)),
		int(result.get("selection_overlay_count", 0)),
		int(result.get("occupied_readability_cells", 0)),
		int(result.get("max_overlays_per_readability_cell", 0)),
	])
	quit(0)


func _fail(reason: String, result: Dictionary) -> void:
	push_error("%s: %s" % [reason, result])
	quit(1)

