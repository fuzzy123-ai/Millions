extends SceneTree

const ClientAdapter := preload("res://scripts/net/ClientAdapter.gd")
const CommandQueue := preload("res://scripts/net/CommandQueue.gd")
const SnapshotBuffer := preload("res://scripts/net/SnapshotBuffer.gd")
const ClientWorldState := preload("res://scripts/net/ClientWorldState.gd")
const RenderAdapter := preload("res://scripts/render/RenderAdapter.gd")


func _init() -> void:
	var adapter := ClientAdapter.new()
	var command_queue := CommandQueue.new()
	var snapshot_buffer := SnapshotBuffer.new()
	var world_state := ClientWorldState.new()
	var render_adapter := RenderAdapter.new()

	root.add_child(adapter)
	root.add_child(command_queue)
	root.add_child(snapshot_buffer)
	root.add_child(world_state)
	root.add_child(render_adapter)

	adapter.configure_components(command_queue, snapshot_buffer, world_state, render_adapter)

	var move_context: Dictionary = adapter.queue_command_context(
		3,
		[1001, 1002],
		{
			"kind": "world_position",
			"x_mm": 1250,
			"y_mm": -750,
		},
		42
	)
	if int(move_context.get("client_seq", 0)) != 1:
		_fail("client seq mismatch", move_context)
		return
	if int(move_context.get("command_id", 0)) != 1:
		_fail("command id mismatch", move_context)
		return
	if str(move_context.get("feedback_state", "")) != "command_intent_queued":
		_fail("feedback state mismatch", move_context)
		return
	if move_context.has("accepted") or move_context.has("authoritative_position") or move_context.has("server_tick"):
		_fail("adapter leaked authoritative result fields", move_context)
		return

	var payload: Dictionary = move_context.get("payload", {})
	var command_context: Dictionary = payload.get("command_context", {})
	if str(command_context.get("feedback_state", "")) != "command_intent_queued":
		_fail("payload feedback state mismatch", command_context)
		return
	var selected: Array = command_context.get("selected_entity_ids", [])
	if selected.size() != 2 or int(selected[0]) != 1001 or int(selected[1]) != 1002:
		_fail("selected entity ids mismatch", command_context)
		return
	var target: Dictionary = command_context.get("target", {})
	if str(target.get("kind", "")) != "world_position":
		_fail("target kind mismatch", target)
		return
	if int(target.get("x_mm", 0)) != 1250 or int(target.get("y_mm", 0)) != -750:
		_fail("target coordinates mismatch", target)
		return

	var pending: Array[Dictionary] = command_queue.snapshot_pending()
	if pending.size() != 1:
		_fail("pending count mismatch", {"pending": pending.size()})
		return
	var pending_command: Dictionary = pending[0]
	if str(pending_command.get("feedback_state", "")) != "command_intent_queued":
		_fail("pending feedback state mismatch", pending_command)
		return
	if pending_command.has("accepted") or pending_command.has("authoritative_position") or pending_command.has("server_tick"):
		_fail("pending command leaked authoritative result fields", pending_command)
		return

	var fallback_context: Dictionary = adapter.queue_command_context(
		4,
		[1001],
		{
			"kind": "unsafe_scene_path",
			"path": "/root/App/Unit",
		},
		43
	)
	var fallback_target: Dictionary = fallback_context.get("target", {})
	if str(fallback_target.get("kind", "")) != "none":
		_fail("unsafe target kind was not normalized", fallback_context)
		return
	if fallback_target.has("path"):
		_fail("unsafe target data survived normalization", fallback_target)
		return

	print("command_feedback_context_check status=ok queued=2 feedback=command_intent_queued authority=server target_kinds=world_position,none")
	quit(0)


func _fail(message: String, details: Dictionary) -> void:
	push_error("%s: %s" % [message, str(details)])
	quit(1)

