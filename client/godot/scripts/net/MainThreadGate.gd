extends Node

const SYSTEM_NAME := "MainThreadGate"

var pending_messages: Array[Dictionary] = []


func enqueue_main_thread_message(message: Dictionary) -> void:
	pending_messages.append(message.duplicate(true))
	_set_metric("main_thread_pending_messages", pending_messages.size())


func drain_messages() -> Array[Dictionary]:
	var drained := pending_messages.duplicate(true)
	pending_messages.clear()
	_set_metric("main_thread_pending_messages", 0)
	return drained


func pending_count() -> int:
	return pending_messages.size()


func _set_metric(metric_name: String, value: Variant) -> void:
	if not is_inside_tree():
		return
	var perf_ledger := get_node_or_null("/root/PerfLedger")
	if perf_ledger != null and perf_ledger.has_method("set_metric"):
		perf_ledger.set_metric(metric_name, value)
