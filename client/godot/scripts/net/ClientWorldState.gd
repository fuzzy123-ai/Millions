extends Node

const SYSTEM_NAME := "ClientWorldState"

var entities: Dictionary = {}
var last_server_tick: int = 0


func apply_snapshot(snapshot: Dictionary) -> void:
	var payload: Dictionary = snapshot.get("payload", {})
	for entity: Dictionary in payload.get("entities", []):
		var entity_id := int(entity.get("entity_id", 0))
		if entity_id == 0:
			continue
		entities[entity_id] = entity.duplicate(true)

	for removed_id: Variant in payload.get("removed_entities", []):
		entities.erase(int(removed_id))

	last_server_tick = int(snapshot.get("tick", last_server_tick))
	_set_metric("authoritative_entity_count", entities.size())


func entity_count() -> int:
	return entities.size()


func render_records() -> Array[Dictionary]:
	var records: Array[Dictionary] = []
	for entity_id: Variant in entities.keys():
		var entity: Dictionary = entities[entity_id]
		records.append(entity.duplicate(true))
	return records


func clear() -> void:
	entities.clear()
	last_server_tick = 0
	_set_metric("authoritative_entity_count", 0)


func _set_metric(metric_name: String, value: Variant) -> void:
	if not is_inside_tree():
		return
	var perf_ledger := get_node_or_null("/root/PerfLedger")
	if perf_ledger != null and perf_ledger.has_method("set_metric"):
		perf_ledger.set_metric(metric_name, value)
