extends SceneTree

const PerfBudget := preload("res://scripts/perf/PerfBudget.gd")


func _init() -> void:
	var pass_result: Dictionary = PerfBudget.evaluate_godot_report({
		"decode_p95": 1.5,
		"snapshot_apply_p95": 2.5,
		"render_update_p95": 4.5,
		"frame_p95": 16.0,
	})
	if pass_result.get("budget_result") != "pass":
		_fail("expected pass", pass_result)
		return

	var blocked_result: Dictionary = PerfBudget.evaluate_godot_report({
		"decode_p95": 1.5,
		"snapshot_apply_p95": 2.5,
		"render_update_p95": 4.5,
	})
	if blocked_result.get("budget_result") != "blocked":
		_fail("expected blocked", blocked_result)
		return
	if not (blocked_result.get("missing", []) as Array).has("frame_p95"):
		_fail("expected missing frame_p95", blocked_result)
		return

	var fail_result: Dictionary = PerfBudget.evaluate_godot_report({
		"decode_p95": 2.5,
		"snapshot_apply_p95": 2.5,
		"render_update_p95": 4.5,
		"frame_p95": 16.0,
	})
	if fail_result.get("budget_result") != "fail":
		_fail("expected fail", fail_result)
		return
	if not (fail_result.get("failures", []) as Array).has("decode_p95"):
		_fail("expected failed decode_p95", fail_result)
		return

	print("perf_budget_check status=ok pass=1 blocked=1 fail=1")
	quit(0)


func _fail(message: String, status: Dictionary) -> void:
	push_error("%s: %s" % [message, status])
	print("perf_budget_check status=failed")
	quit(1)
