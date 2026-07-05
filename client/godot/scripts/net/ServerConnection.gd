extends Node

const SYSTEM_NAME := "ServerConnection"

var target_host: String = "127.0.0.1"
var target_port: int = 0
var connected: bool = false
var last_header: Dictionary = {}


func configure(host: String, port: int) -> void:
	target_host = host
	target_port = port


func connect_local() -> void:
	connected = true
	Online.set_connection_state("local")
	ClientLog.info(SYSTEM_NAME, "local connection placeholder active", {
		"host": target_host,
		"port": target_port,
	})


func disconnect_local() -> void:
	if not connected:
		return

	connected = false
	Online.set_connection_state("offline")
	ClientLog.info(SYSTEM_NAME, "local connection placeholder stopped")


func record_decoded_header(header: Dictionary) -> void:
	last_header = header.duplicate(true)
	_set_metric("last_server_tick", header.get("tick", 0))
	ClientLog.info(SYSTEM_NAME, "decoded protocol header recorded", {
		"message_type": header.get("message_type", 0),
		"server_seq": header.get("server_seq", 0),
		"ack_seq": header.get("ack_seq", 0),
		"tick": header.get("tick", 0),
	})


func _set_metric(metric_name: String, value: Variant) -> void:
	if not is_inside_tree():
		return
	var perf_ledger := get_node_or_null("/root/PerfLedger")
	if perf_ledger != null and perf_ledger.has_method("set_metric"):
		perf_ledger.set_metric(metric_name, value)
