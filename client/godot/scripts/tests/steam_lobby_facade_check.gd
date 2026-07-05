extends SceneTree

const SteamLobbyFacade := preload("res://scripts/net/SteamLobbyFacade.gd")


func _init() -> void:
	var facade := SteamLobbyFacade.new()
	root.add_child(facade)

	var status: Dictionary = facade.create_local_mock_lobby("local-abc123", "Player 1")
	if status.get("state") != "joinable":
		_fail("create local mock lobby failed", status)
		return
	var metadata: Dictionary = status.get("metadata", {})
	if metadata.get("millions.schema") != "steam_bridge_v0":
		_fail("metadata schema mismatch", status)
		return
	if metadata.get("millions.server_mode") != "pending":
		_fail("initial server mode mismatch", status)
		return

	status = facade.queue_ready()
	if status.get("state") != "ready_pending":
		_fail("queue ready failed", status)
		return
	metadata = status.get("metadata", {})
	if metadata.get("millions.ready_epoch") != "1":
		_fail("ready epoch mismatch", status)
		return

	status = facade.accept_ready_with_synthetic_session("127.0.0.1:7777")
	if status.get("state") != "ready":
		_fail("ready acceptance failed", status)
		return
	metadata = status.get("metadata", {})
	if metadata.get("millions.endpoint_epoch") != "1":
		_fail("endpoint epoch mismatch", status)
		return
	if metadata.get("millions.server_mode") != "local_direct":
		_fail("server mode mismatch", status)
		return

	status = facade.supersede_ready_endpoint_from_mock_server("127.0.0.1:7778")
	if status.get("server_endpoint") != "127.0.0.1:7778":
		_fail("endpoint supersession failed", status)
		return
	if int(status.get("endpoint_epoch", 0)) != 2:
		_fail("endpoint supersession epoch mismatch", status)
		return

	status = facade.supersede_ready_endpoint_from_mock_server("127.0.0.1:7777")
	if int(status.get("endpoint_epoch", 0)) != 3:
		_fail("endpoint restore epoch mismatch", status)
		return

	var handoff: Dictionary = facade.begin_dedicated_server_handoff()
	if not bool(handoff.get("ok", false)):
		_fail("handoff failed", handoff)
		return
	if handoff.get("schema") != "steam_bridge_v0":
		_fail("handoff schema mismatch", handoff)
		return
	if handoff.get("identity_mode") != "local_mock":
		_fail("handoff identity mode mismatch", handoff)
		return
	if handoff.get("protocol") != "protocol_v0":
		_fail("handoff protocol mismatch", handoff)
		return
	if handoff.get("server_endpoint") != "127.0.0.1:7777":
		_fail("handoff endpoint mismatch", handoff)
		return
	if int(handoff.get("endpoint_epoch", 0)) != 3:
		_fail("handoff endpoint epoch mismatch", handoff)
		return
	if handoff.get("player_session_id") != "local-session-local-abc123-player-1":
		_fail("handoff session mismatch", handoff)
		return
	if handoff.get("lobby_state") != "ready":
		_fail("handoff lobby state mismatch", handoff)
		return

	var bad_facade := SteamLobbyFacade.new()
	root.add_child(bad_facade)
	status = bad_facade.create_local_mock_lobby("local-skip", "Player 2")
	status = bad_facade.accept_ready_with_synthetic_session("127.0.0.1:7777")
	if bool(status.get("ok", true)) or status.get("state") != "error":
		_fail("ready skip should fail", status)
		return

	status = facade.mark_in_match()
	if status.get("state") != "in_match":
		_fail("mark in match failed", status)
		return

	print("steam_lobby_facade_check status=ok state=in_match endpoint=127.0.0.1:7777")
	quit(0)


func _fail(message: String, status: Dictionary) -> void:
	push_error("%s: %s" % [message, status])
	print("steam_lobby_facade_check status=failed")
	quit(1)
