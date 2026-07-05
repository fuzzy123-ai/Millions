extends Node

const SYSTEM_NAME := "SnapshotBuffer"

var snapshots: Array[Dictionary] = []


func push_snapshot(header: Dictionary, payload: Dictionary = {}) -> void:
	var snapshot := {
		"server_seq": header.get("server_seq", 0),
		"tick": header.get("tick", 0),
		"message_type": header.get("message_type", 0),
		"payload": payload.duplicate(true),
	}
	snapshots.append(snapshot)
	snapshots.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return int(a.get("server_seq", 0)) < int(b.get("server_seq", 0))
	)
	_set_metric("buffered_snapshot_count", snapshots.size())


func latest() -> Dictionary:
	if snapshots.is_empty():
		return {}

	return snapshots[snapshots.size() - 1].duplicate(true)


func clear() -> void:
	snapshots.clear()
	_set_metric("buffered_snapshot_count", 0)


func _set_metric(metric_name: String, value: Variant) -> void:
	if not is_inside_tree():
		return
	var perf_ledger := get_node_or_null("/root/PerfLedger")
	if perf_ledger != null and perf_ledger.has_method("set_metric"):
		perf_ledger.set_metric(metric_name, value)
