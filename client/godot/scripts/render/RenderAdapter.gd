extends Node

const SYSTEM_NAME := "RenderAdapter"

var latest_records: Array[Dictionary] = []
var latest_batches: Array[Dictionary] = []


func update_from_world_state(world_state: Node) -> void:
	if not world_state.has_method("render_records"):
		_warn("world state missing render_records method")
		return

	latest_records = _records_for_render(world_state.render_records())
	latest_batches = _batches_for_records(latest_records)
	_set_metric("render_proxy_count", latest_records.size())
	_set_metric("render_batch_count", latest_batches.size())


func snapshot_records() -> Array[Dictionary]:
	return latest_records.duplicate(true)


func render_batches() -> Array[Dictionary]:
	return latest_batches.duplicate(true)


func batch_count() -> int:
	return latest_batches.size()


func clear() -> void:
	latest_records.clear()
	latest_batches.clear()
	_set_metric("render_proxy_count", 0)
	_set_metric("render_batch_count", 0)


func _records_for_render(raw_records: Array[Dictionary]) -> Array[Dictionary]:
	var records: Array[Dictionary] = []
	for raw_record: Dictionary in raw_records:
		var entity_id := int(raw_record.get("entity_id", 0))
		if entity_id == 0:
			continue
		var entity_kind := int(raw_record.get("entity_kind", 0))
		var faction_id := int(raw_record.get("faction_id", 0))
		var x_mm := int(raw_record.get("x_mm", 0))
		var y_mm := int(raw_record.get("y_mm", 0))
		records.append(
			{
				"entity_id": entity_id,
				"entity_kind": entity_kind,
				"faction_id": faction_id,
				"render_key": _render_key(entity_kind, faction_id),
				"x_mm": x_mm,
				"y_mm": y_mm,
				"x_m": float(x_mm) / 1000.0,
				"y_m": float(y_mm) / 1000.0,
				"position_m": Vector2(float(x_mm) / 1000.0, float(y_mm) / 1000.0),
				"health_q8": int(raw_record.get("health_q8", 0)),
			}
		)
	records.sort_custom(_sort_render_records)
	return records


func _batches_for_records(records: Array[Dictionary]) -> Array[Dictionary]:
	var batches: Array[Dictionary] = []
	var by_key: Dictionary = {}
	for record: Dictionary in records:
		var render_key := String(record.get("render_key", "kind:0:faction:0"))
		if not by_key.has(render_key):
			var batch: Dictionary = {
				"render_key": render_key,
				"entity_kind": int(record.get("entity_kind", 0)),
				"faction_id": int(record.get("faction_id", 0)),
				"proxy_count": 0,
				"records": [],
			}
			by_key[render_key] = batch
			batches.append(batch)
		var target_batch: Dictionary = by_key[render_key]
		var batch_records: Array = target_batch["records"]
		batch_records.append(record.duplicate(true))
		target_batch["proxy_count"] = batch_records.size()
	batches.sort_custom(_sort_render_batches)
	return batches


func _render_key(entity_kind: int, faction_id: int) -> String:
	return "kind:%d:faction:%d" % [entity_kind, faction_id]


func _sort_render_records(left: Dictionary, right: Dictionary) -> bool:
	var left_key := String(left.get("render_key", ""))
	var right_key := String(right.get("render_key", ""))
	if left_key == right_key:
		return int(left.get("entity_id", 0)) < int(right.get("entity_id", 0))
	return left_key < right_key


func _sort_render_batches(left: Dictionary, right: Dictionary) -> bool:
	return String(left.get("render_key", "")) < String(right.get("render_key", ""))


func _set_metric(metric_name: String, value: Variant) -> void:
	if not is_inside_tree():
		return
	var perf_ledger := get_node_or_null("/root/PerfLedger")
	if perf_ledger != null and perf_ledger.has_method("set_metric"):
		perf_ledger.set_metric(metric_name, value)


func _warn(message: String) -> void:
	if not is_inside_tree():
		push_warning("%s: %s" % [SYSTEM_NAME, message])
		return
	var client_log := get_node_or_null("/root/ClientLog")
	if client_log != null and client_log.has_method("warn"):
		client_log.warn(SYSTEM_NAME, message)
	else:
		push_warning("%s: %s" % [SYSTEM_NAME, message])
