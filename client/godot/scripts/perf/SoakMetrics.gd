extends RefCounted

const REQUIRED_METRICS: Array[String] = [
	"memory_godot_start_mb",
	"memory_godot_peak_mb",
	"memory_godot_end_mb",
	"allocation_bytes_total",
	"queue_depth_max",
	"connections_active",
	"snapshots_dropped",
	"resends_queued",
	"resends_sent",
	"shutdown_clean",
]


static func build_snapshot(values: Dictionary) -> Dictionary:
	var snapshot := {}
	for metric_name in REQUIRED_METRICS:
		snapshot[metric_name] = int(values.get(metric_name, 0))
	return snapshot


static func missing_required(snapshot: Dictionary) -> Array[String]:
	var missing: Array[String] = []
	for metric_name in REQUIRED_METRICS:
		if not snapshot.has(metric_name):
			missing.append(metric_name)
	return missing


static func has_required_metrics(snapshot: Dictionary) -> bool:
	return missing_required(snapshot).is_empty()


static func record_shutdown(snapshot: Dictionary, clean: bool) -> Dictionary:
	var next := snapshot.duplicate(true)
	next["shutdown_clean"] = 1 if clean else 0
	return next
