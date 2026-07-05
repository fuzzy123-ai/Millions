extends RefCounted

const CLIENT_FRAME_P95_MS_MAX := 16.7
const GODOT_DECODE_P95_MS_MAX := 2.0
const GODOT_SNAPSHOT_APPLY_P95_MS_MAX := 3.0
const GODOT_RENDER_UPDATE_P95_MS_MAX := 5.0


static func evaluate_godot_report(report: Dictionary) -> Dictionary:
	var missing: Array[String] = []
	var failures: Array[String] = []

	_check_required_metric(report, "decode_p95", GODOT_DECODE_P95_MS_MAX, missing, failures)
	_check_required_metric(report, "snapshot_apply_p95", GODOT_SNAPSHOT_APPLY_P95_MS_MAX, missing, failures)
	_check_required_metric(report, "render_update_p95", GODOT_RENDER_UPDATE_P95_MS_MAX, missing, failures)
	_check_required_metric(report, "frame_p95", CLIENT_FRAME_P95_MS_MAX, missing, failures)

	if not failures.is_empty():
		return {
			"budget_result": "fail",
			"missing": missing,
			"failures": failures,
		}
	if not missing.is_empty():
		return {
			"budget_result": "blocked",
			"missing": missing,
			"failures": failures,
		}
	return {
		"budget_result": "pass",
		"missing": missing,
		"failures": failures,
	}


static func _check_required_metric(
	report: Dictionary,
	metric_name: String,
	max_value: float,
	missing: Array[String],
	failures: Array[String]
) -> void:
	if not report.has(metric_name):
		missing.append(metric_name)
		return
	var value := float(report.get(metric_name))
	if value > max_value:
		failures.append(metric_name)
