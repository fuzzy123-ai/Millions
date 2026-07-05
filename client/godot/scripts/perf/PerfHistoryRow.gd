extends RefCounted

const SCHEMA_VERSION := 1
const SOURCE_SLICE := "PHIST-02"


static func make_godot_render_contract_row(
	date: String,
	machine_label: String,
	build_id: String,
	scenario_id: String,
	source_artifact: String,
	metrics: Dictionary = {},
	why_changed: String = "godot render contract row emitted without measured frame percentiles"
) -> Dictionary:
	return {
		"schema_version": SCHEMA_VERSION,
		"ledger_id": build_ledger_id(machine_label, scenario_id, date, build_id),
		"date": date,
		"machine_label": machine_label,
		"build_id": build_id,
		"source_slice": SOURCE_SLICE,
		"scenario_id": scenario_id,
		"scenario_family": scenario_family_for(scenario_id),
		"status": "informational",
		"budget_result": "blocked",
		"budget_keys": [
			"godot_decode_p95_ms_max",
			"godot_snapshot_apply_p95_ms_max",
			"godot_render_update_p95_ms_max",
			"client_frame_p95_ms_max",
		],
		"metrics": _normalize_render_metrics(metrics),
		"source_artifact": source_artifact,
		"why_changed": why_changed,
		"claim_scope": "informational_contract_only",
		"redaction_status": "pass",
		"notes": "",
	}


static func build_ledger_id(
	machine_label: String,
	scenario_id: String,
	date: String,
	build_id: String
) -> String:
	return "%s__%s__%s__%s" % [machine_label, scenario_id, date, build_id]


static func scenario_family_for(scenario_id: String) -> String:
	if scenario_id.begins_with("godot_render_"):
		return "godot_render"
	if scenario_id.begins_with("sim_"):
		return "simulation_scale"
	if scenario_id.begins_with("gload_"):
		return "faction_scale"
	if scenario_id.begins_with("int_"):
		return "interest_bandwidth"
	if scenario_id.begins_with("reconnect_"):
		return "reconnect"
	if scenario_id.begins_with("loss_jitter_"):
		return "loss_jitter"
	return "godot_render"


static func _normalize_render_metrics(metrics: Dictionary) -> Dictionary:
	var normalized := {
		"decode_ms": _percentiles_or_null(metrics.get("decode_ms", {})),
		"snapshot_apply_ms": _percentiles_or_null(metrics.get("snapshot_apply_ms", {})),
		"render_update_ms": _percentiles_or_null(metrics.get("render_update_ms", {})),
		"frame_ms": _percentiles_or_null(metrics.get("frame_ms", {})),
	}
	if metrics.has("proxy_count"):
		normalized["proxy_count"] = int(metrics.get("proxy_count"))
	return normalized


static func _percentiles_or_null(value: Variant) -> Dictionary:
	var source := {}
	if typeof(value) == TYPE_DICTIONARY:
		source = value
	return {
		"p50": source.get("p50", null),
		"p95": source.get("p95", null),
		"p99": source.get("p99", null),
	}
