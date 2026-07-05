extends Node

const SYSTEM_NAME := "SteamLobbyFacade"
const BRIDGE_SCHEMA := "steam_bridge_v0"
const PROTOCOL_LABEL := "protocol_v0"
const DEFAULT_BUILD_ID := "local-uncommitted"

enum LobbyState {
	OFFLINE,
	LOCAL_MOCK,
	CREATING,
	JOINABLE,
	JOINING,
	JOINED,
	READY_PENDING,
	READY,
	LAUNCHING,
	IN_MATCH,
	LEFT,
	ERROR,
}

var lobby_state: LobbyState = LobbyState.OFFLINE
var lobby_id: String = ""
var local_player_name: String = "Player 1"
var server_endpoint: String = "127.0.0.1:0"
var player_session_id: String = ""
var server_mode: String = "pending"
var endpoint_epoch: int = 0
var ready_epoch: int = 0
var build_id: String = DEFAULT_BUILD_ID
var host_slot: String = "0"


func create_local_mock_lobby(next_lobby_id: String, player_name: String = "Player 1") -> Dictionary:
	lobby_id = next_lobby_id
	local_player_name = player_name
	_reset_handoff_state()
	lobby_state = LobbyState.JOINABLE
	return lobby_status()


func join_local_mock_lobby(next_lobby_id: String, player_name: String = "Player 1") -> Dictionary:
	lobby_id = next_lobby_id
	local_player_name = player_name
	_reset_handoff_state()
	lobby_state = LobbyState.JOINED
	return lobby_status()


func queue_ready() -> Dictionary:
	if lobby_state != LobbyState.JOINABLE and lobby_state != LobbyState.JOINED:
		lobby_state = LobbyState.ERROR
		return lobby_status("ready requires joinable or joined lobby")

	lobby_state = LobbyState.READY_PENDING
	ready_epoch += 1
	return lobby_status()


func accept_ready_from_mock_server(next_endpoint: String, next_session_id: String) -> Dictionary:
	if lobby_state != LobbyState.READY_PENDING:
		lobby_state = LobbyState.ERROR
		return lobby_status("ready acceptance requires ready_pending")

	server_endpoint = next_endpoint
	player_session_id = next_session_id
	server_mode = "local_direct"
	endpoint_epoch += 1
	lobby_state = LobbyState.READY
	return lobby_status()


func accept_ready_with_synthetic_session(next_endpoint: String) -> Dictionary:
	return accept_ready_from_mock_server(next_endpoint, synthetic_session_id(lobby_id, local_player_name))


func supersede_ready_endpoint_from_mock_server(next_endpoint: String, next_session_id: String = "") -> Dictionary:
	if lobby_state != LobbyState.READY:
		lobby_state = LobbyState.ERROR
		return lobby_status("endpoint supersession requires ready")

	if next_endpoint != server_endpoint:
		endpoint_epoch += 1
	server_endpoint = next_endpoint
	if not next_session_id.is_empty():
		player_session_id = next_session_id
	return lobby_status()


func begin_dedicated_server_handoff() -> Dictionary:
	if lobby_state != LobbyState.READY:
		lobby_state = LobbyState.ERROR
		return lobby_status("handoff requires ready")

	var metadata := lobby_metadata()
	lobby_state = LobbyState.LAUNCHING
	return {
		"ok": true,
		"schema": BRIDGE_SCHEMA,
		"lobby_id": lobby_id,
		"lobby_state": String(metadata.get("millions.lobby_state", "ready")),
		"identity_mode": "local_mock",
		"protocol": PROTOCOL_LABEL,
		"server_mode": server_mode,
		"server_endpoint": server_endpoint,
		"endpoint_epoch": endpoint_epoch,
		"ready_epoch": ready_epoch,
		"build_id": build_id,
		"host_slot": host_slot,
		"player_display_name": local_player_name,
		"player_session_id": player_session_id,
		"metadata": metadata,
		"state": lobby_state_name(),
	}


func mark_in_match() -> Dictionary:
	if lobby_state == LobbyState.LAUNCHING:
		lobby_state = LobbyState.IN_MATCH
	return lobby_status()


func leave_lobby() -> Dictionary:
	lobby_state = LobbyState.LEFT
	lobby_id = ""
	_reset_handoff_state()
	return lobby_status()


func lobby_status(error_message: String = "") -> Dictionary:
	return {
		"ok": error_message.is_empty(),
		"state": lobby_state_name(),
		"lobby_id": lobby_id,
		"identity_mode": "local_mock" if lobby_state != LobbyState.OFFLINE else "none",
		"server_endpoint": server_endpoint,
		"server_mode": server_mode,
		"endpoint_epoch": endpoint_epoch,
		"ready_epoch": ready_epoch,
		"build_id": build_id,
		"host_slot": host_slot,
		"player_display_name": local_player_name,
		"player_session_id": player_session_id,
		"metadata": lobby_metadata(),
		"error": error_message,
	}


func lobby_metadata() -> Dictionary:
	return {
		"millions.schema": BRIDGE_SCHEMA,
		"millions.identity_mode": "local_mock" if lobby_state != LobbyState.OFFLINE else "none",
		"millions.lobby_state": lobby_state_name(),
		"millions.protocol": PROTOCOL_LABEL,
		"millions.server_mode": server_mode,
		"millions.endpoint": server_endpoint if server_mode != "pending" else "",
		"millions.endpoint_epoch": str(endpoint_epoch),
		"millions.ready_epoch": str(ready_epoch),
		"millions.build_id": build_id,
		"millions.host_slot": host_slot,
	}


func lobby_state_name() -> String:
	match lobby_state:
		LobbyState.OFFLINE:
			return "offline"
		LobbyState.LOCAL_MOCK:
			return "local_mock"
		LobbyState.CREATING:
			return "creating"
		LobbyState.JOINABLE:
			return "joinable"
		LobbyState.JOINING:
			return "joining"
		LobbyState.JOINED:
			return "joined"
		LobbyState.READY_PENDING:
			return "ready_pending"
		LobbyState.READY:
			return "ready"
		LobbyState.LAUNCHING:
			return "launching"
		LobbyState.IN_MATCH:
			return "in_match"
		LobbyState.LEFT:
			return "left"
		LobbyState.ERROR:
			return "error"
		_:
			return "unknown"


static func synthetic_session_id(next_lobby_id: String, player_name: String) -> String:
	return "local-session-%s-%s" % [
		_sanitize_identity(next_lobby_id),
		_sanitize_identity(player_name),
	]


static func _sanitize_identity(value: String) -> String:
	var sanitized := value.to_lower().strip_edges()
	var output := ""
	for index: int in range(sanitized.length()):
		var character := sanitized.substr(index, 1)
		var code := character.unicode_at(0)
		var is_lower_ascii := code >= 97 and code <= 122
		var is_digit := code >= 48 and code <= 57
		if is_lower_ascii or is_digit:
			output += character
		elif character == "-" or character == "_":
			output += character
		else:
			output += "-"
	return output


func _reset_handoff_state() -> void:
	server_endpoint = "127.0.0.1:0"
	player_session_id = ""
	server_mode = "pending"
	endpoint_epoch = 0
	ready_epoch = 0
