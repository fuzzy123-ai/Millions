extends SceneTree

const PerfHistoryRow := preload("res://scripts/perf/PerfHistoryRow.gd")


func _init() -> void:
	var schema_text := FileAccess.get_file_as_string(
		"res://../../tests/perf/performance-history-ledger.schema.json"
	)
	var schema: Dictionary = JSON.parse_string(schema_text)
	if schema.is_empty():
		_fail("schema did not parse", {})
		return

	var row := PerfHistoryRow.make_godot_render_contract_row(
		"2026-07-03",
		"local-dev-01",
		"local-uncommitted",
		"godot_render_1k_visible",
		"tests/perf/render-stress-smoke-report.json",
		{"proxy_count": 1000}
	)

	for field in schema.get("required_fields", []):
		if not row.has(field):
			_fail("missing required field %s" % field, row)
			return

	if row.get("ledger_id") != "local-dev-01__godot_render_1k_visible__2026-07-03__local-uncommitted":
		_fail("ledger id mismatch", row)
		return
	if row.get("scenario_family") != "godot_render":
		_fail("scenario family mismatch", row)
		return
	if row.get("status") != "informational":
		_fail("status mismatch", row)
		return
	if row.get("budget_result") != "blocked":
		_fail("budget result mismatch", row)
		return
	if row.get("claim_scope") != "informational_contract_only":
		_fail("claim scope mismatch", row)
		return
	if row.get("redaction_status") != "pass":
		_fail("redaction mismatch", row)
		return

	var metrics: Dictionary = row.get("metrics")
	if metrics.get("proxy_count") != 1000:
		_fail("proxy count metric mismatch", row)
		return
	if (metrics.get("frame_ms") as Dictionary).get("p95") != null:
		_fail("unmeasured frame p95 should remain null", row)
		return

	print("perf_history_row_check status=ok required_fields=%s" % schema.get("required_fields", []).size())
	quit(0)


func _fail(message: String, row: Dictionary) -> void:
	push_error("%s: %s" % [message, row])
	print("perf_history_row_check status=failed")
	quit(1)
