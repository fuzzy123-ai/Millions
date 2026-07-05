extends RefCounted

const MAGIC := 0x4D4D
const PROTOCOL_VERSION := 0
const HEADER_LEN := 48

enum MessageType {
	CLIENT_HELLO = 1,
	SERVER_HELLO = 2,
	CLIENT_COMMAND_BATCH = 3,
	SERVER_COMMAND_ACK = 4,
	SERVER_FULL_SNAPSHOT = 5,
	SERVER_DELTA_SNAPSHOT = 6,
	PING = 7,
	DISCONNECT = 8,
}


static func decode_header(packet: PackedByteArray) -> Dictionary:
	if packet.size() < HEADER_LEN:
		return _error("too_short")

	var magic := _read_u16(packet, 0)
	if magic != MAGIC:
		return _error("bad_magic", {"magic": magic})

	var protocol_version := _read_u16(packet, 2)
	if protocol_version != PROTOCOL_VERSION:
		return _error("unsupported_protocol", {"protocol_version": protocol_version})

	var message_type := packet[4]
	if message_type < MessageType.CLIENT_HELLO or message_type > MessageType.DISCONNECT:
		return _error("unknown_message_type", {"message_type": message_type})

	var flags := packet[5]
	if flags != 0:
		return _error("non_zero_flags", {"flags": flags})

	var header_len := _read_u16(packet, 6)
	if header_len != HEADER_LEN:
		return _error("bad_header_len", {"header_len": header_len})

	var payload_len := _read_u32(packet, 8)
	var actual_payload_len := packet.size() - HEADER_LEN
	if payload_len != actual_payload_len:
		return _error("payload_len_mismatch", {
			"declared": payload_len,
			"actual": actual_payload_len,
		})

	return {
		"ok": true,
		"message_type": message_type,
		"payload_len": payload_len,
		"connection_id": _read_u64(packet, 12),
		"session_id": _read_u64(packet, 20),
		"client_seq": _read_u32(packet, 28),
		"server_seq": _read_u32(packet, 32),
		"ack_seq": _read_u32(packet, 36),
		"tick": _read_u64(packet, 40),
	}


static func _error(code: String, fields: Dictionary = {}) -> Dictionary:
	return {
		"ok": false,
		"error": code,
		"fields": fields,
	}


static func _read_u16(packet: PackedByteArray, offset: int) -> int:
	return packet[offset] | (packet[offset + 1] << 8)


static func _read_u32(packet: PackedByteArray, offset: int) -> int:
	return (
		packet[offset]
		| (packet[offset + 1] << 8)
		| (packet[offset + 2] << 16)
		| (packet[offset + 3] << 24)
	)


static func _read_u64(packet: PackedByteArray, offset: int) -> int:
	var low := _read_u32(packet, offset)
	var high := _read_u32(packet, offset + 4)
	return low | (high << 32)
