extends SceneTree

const RenderStressSmokeScene := preload("res://scenes/dev/render_stress_smoke.tscn")
const EXPECTED_PROXY_COUNT := 1000


func _init() -> void:
	var scene: Node = RenderStressSmokeScene.instantiate()
	root.add_child(scene)
	await process_frame

	var result: Dictionary = scene.run_stress(EXPECTED_PROXY_COUNT)
	if int(result.get("expected_proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		push_error("expected proxy count mismatch: %s" % result)
		quit(1)
		return
	if int(result.get("proxy_count", -1)) != EXPECTED_PROXY_COUNT:
		push_error("proxy count mismatch: %s" % result)
		quit(1)
		return
	if int(result.get("elapsed_us", -1)) < 0:
		push_error("elapsed time mismatch: %s" % result)
		quit(1)
		return
	if float(result.get("bounds_max_x", 0.0)) <= float(result.get("bounds_min_x", 0.0)):
		push_error("horizontal bounds mismatch: %s" % result)
		quit(1)
		return
	if float(result.get("bounds_max_y", 0.0)) <= float(result.get("bounds_min_y", 0.0)):
		push_error("vertical bounds mismatch: %s" % result)
		quit(1)
		return

	print("render_stress_smoke status=ok proxies=%d elapsed_us=%d" % [
		int(result.get("proxy_count", 0)),
		int(result.get("elapsed_us", 0)),
	])
	quit(0)
