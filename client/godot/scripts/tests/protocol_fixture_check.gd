extends SceneTree

const ProtocolCodec := preload("res://scripts/net/ProtocolCodec.gd")

const FIXTURES := [
	{
		"path": "res://../../protocol/fixtures/protocol_v0_server_hello_accept.bin",
		"message_type": ProtocolCodec.MessageType.SERVER_HELLO,
		"payload_len": 8,
		"client_seq": 0,
		"server_seq": 1,
		"ack_seq": 0,
		"tick": 0,
	},
	{
		"path": "res://../../protocol/fixtures/protocol_v0_command_ready_batch_ok.bin",
		"message_type": ProtocolCodec.MessageType.CLIENT_COMMAND_BATCH,
		"payload_len": 24,
		"client_seq": 1,
		"server_seq": 0,
		"ack_seq": 1,
		"tick": 10,
	},
	{
		"path": "res://../../protocol/fixtures/protocol_v0_snapshot_full_minimal_ok.bin",
		"message_type": ProtocolCodec.MessageType.SERVER_FULL_SNAPSHOT,
		"payload_len": 60,
		"client_seq": 0,
		"server_seq": 2,
		"ack_seq": 1,
		"tick": 20,
	},
]


func _init() -> void:
	var failures: Array[String] = []
	for fixture: Dictionary in FIXTURES:
		_check_fixture(fixture, failures)

	if failures.is_empty():
		print("godot_fixture_check status=ok fixtures=%d" % FIXTURES.size())
		quit(0)
		return

	for failure: String in failures:
		push_error(failure)
	print("godot_fixture_check status=failed failures=%d" % failures.size())
	quit(1)


func _check_fixture(fixture: Dictionary, failures: Array[String]) -> void:
	var global_path := ProjectSettings.globalize_path(fixture["path"])
	var packet := FileAccess.get_file_as_bytes(global_path)
	if packet.is_empty():
		failures.append("fixture unreadable: %s" % fixture["path"])
		return

	var header: Dictionary = ProtocolCodec.decode_header(packet)
	if not bool(header.get("ok", false)):
		failures.append("decode failed for %s: %s" % [fixture["path"], header])
		return

	for key: String in ["message_type", "payload_len", "client_seq", "server_seq", "ack_seq", "tick"]:
		if int(header.get(key, -1)) != int(fixture[key]):
			failures.append("%s mismatch for %s: got %s expected %s" % [
				key,
				fixture["path"],
				header.get(key, null),
				fixture[key],
			])
