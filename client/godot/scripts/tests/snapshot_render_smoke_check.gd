extends SceneTree

const SnapshotRenderSmokeScene := preload("res://scenes/dev/snapshot_render_smoke.tscn")


func _init() -> void:
	var scene: Node = SnapshotRenderSmokeScene.instantiate()
	root.add_child(scene)
	await process_frame

	var result: Dictionary = scene.run_smoke()
	if int(result.get("proxy_count", -1)) != 1:
		push_error("proxy count mismatch: %s" % result)
		quit(1)
		return
	if int(result.get("entity_count", -1)) != 1:
		push_error("entity count mismatch: %s" % result)
		quit(1)
		return
	if int(result.get("batch_count", -1)) != 1:
		push_error("batch count mismatch: %s" % result)
		quit(1)
		return
	if int(result.get("pending_commands", -1)) != 1:
		push_error("pending command count mismatch: %s" % result)
		quit(1)
		return

	print("snapshot_render_smoke status=ok proxies=1 batches=1 entities=1 pending=1")
	quit(0)
