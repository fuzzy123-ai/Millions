extends Node

const SYSTEM_NAME := "CommandQueue"

var next_client_seq: int = 1
var next_command_id: int = 1
var pending_commands: Array[Dictionary] = []


func queue_intent(command_type: int, payload: Dictionary = {}, target_tick: int = 0) -> Dictionary:
	var command := {
		"client_seq": next_client_seq,
		"command_id": next_command_id,
		"command_type": command_type,
		"payload": payload.duplicate(true),
		"resend_count": 0,
		"target_tick": target_tick,
	}
	next_client_seq += 1
	next_command_id += 1
	pending_commands.append(command)
	_set_metric("pending_command_count", pending_commands.size())
	return command


func ack_through(acked_client_seq: int) -> int:
	var kept: Array[Dictionary] = []
	var removed := 0
	for command: Dictionary in pending_commands:
		if int(command.get("client_seq", 0)) <= acked_client_seq:
			removed += 1
		else:
			kept.append(command)
	pending_commands = kept
	_set_metric("pending_command_count", pending_commands.size())
	return removed


func commands_for_resend(acked_client_seq: int) -> Array[Dictionary]:
	var resend: Array[Dictionary] = []
	for command: Dictionary in pending_commands:
		if int(command.get("client_seq", 0)) > acked_client_seq:
			resend.append(command.duplicate(true))
	return resend


func mark_resend_attempt(client_seq: int) -> Dictionary:
	for index: int in range(pending_commands.size()):
		var command: Dictionary = pending_commands[index]
		if int(command.get("client_seq", 0)) == client_seq:
			command["resend_count"] = int(command.get("resend_count", 0)) + 1
			pending_commands[index] = command
			return command.duplicate(true)
	return {}


func pending_count() -> int:
	return pending_commands.size()


func snapshot_pending() -> Array[Dictionary]:
	return pending_commands.duplicate(true)


func clear_for_local_reset() -> void:
	pending_commands.clear()
	next_client_seq = 1
	next_command_id = 1
	_set_metric("pending_command_count", 0)


func _set_metric(metric_name: String, value: Variant) -> void:
	if not is_inside_tree():
		return
	var perf_ledger := get_node_or_null("/root/PerfLedger")
	if perf_ledger != null and perf_ledger.has_method("set_metric"):
		perf_ledger.set_metric(metric_name, value)
