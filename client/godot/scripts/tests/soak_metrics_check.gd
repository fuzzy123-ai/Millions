extends SceneTree

const SoakMetrics := preload("res://scripts/perf/SoakMetrics.gd")


func _init() -> void:
	var snapshot := SoakMetrics.build_snapshot({
		"memory_godot_start_mb": 40,
		"memory_godot_peak_mb": 52,
		"memory_godot_end_mb": 44,
		"allocation_bytes_total": 2048,
		"queue_depth_max": 3,
		"connections_active": 2,
		"snapshots_dropped": 1,
		"resends_queued": 5,
		"resends_sent": 4,
	})
	snapshot = SoakMetrics.record_shutdown(snapshot, true)

	if not SoakMetrics.has_required_metrics(snapshot):
		_fail("expected complete required metrics", snapshot)
		return
	if int(snapshot.get("shutdown_clean")) != 1:
		_fail("expected clean shutdown flag", snapshot)
		return
	if int(snapshot.get("memory_godot_peak_mb")) != 52:
		_fail("peak memory mismatch", snapshot)
		return

	var incomplete := {"connections_active": 2}
	var missing := SoakMetrics.missing_required(incomplete)
	if not missing.has("memory_godot_start_mb"):
		_fail("expected missing memory_godot_start_mb", incomplete)
		return

	print("soak_metrics_check status=ok required_metrics=%s" % SoakMetrics.REQUIRED_METRICS.size())
	quit(0)


func _fail(message: String, snapshot: Dictionary) -> void:
	push_error("%s: %s" % [message, snapshot])
	print("soak_metrics_check status=failed")
	quit(1)
