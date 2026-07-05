extends Node

const SYSTEM_NAME := "ClientAdapter"
const FEEDBACK_STATE_QUEUED := "command_intent_queued"
const TARGET_KIND_NONE := "none"

var command_queue: Node
var snapshot_buffer: Node
var client_world_state: Node
var render_adapter: Node
var reconnect_phase: String = "connected"
var reconnect_player_session_id: String = ""
var reconnect_connection_id: String = ""
var reconnect_pending_full_snapshot: bool = false


func configure_components(
	next_command_queue: Node,
	next_snapshot_buffer: Node,
	next_client_world_state: Node,
	next_render_adapter: Node
) -> void:
	command_queue = next_command_queue
	snapshot_buffer = next_snapshot_buffer
	client_world_state = next_client_world_state
	render_adapter = next_render_adapter


func queue_ready_intent(target_tick: int = 0) -> Dictionary:
	_require_component(command_queue, "command_queue")
	return command_queue.queue_intent(1, {}, target_tick)


func queue_command_context(
	command_type: int,
	selected_entity_ids: Array = [],
	target: Dictionary = {},
	target_tick: int = 0
) -> Dictionary:
	_require_component(command_queue, "command_queue")
	var selected_ids := _copy_selected_entity_ids(selected_entity_ids)
	var normalized_target := _normalize_target(target)
	var payload := {
		"command_context": {
			"selected_entity_ids": selected_ids,
			"target": normalized_target,
			"feedback_state": FEEDBACK_STATE_QUEUED,
		}
	}
	var command: Dictionary = command_queue.queue_intent(command_type, payload, target_tick)
	command["feedback_state"] = FEEDBACK_STATE_QUEUED
	command["selected_entity_ids"] = selected_ids
	command["target"] = normalized_target
	return command


func accept_snapshot(header: Dictionary, payload: Dictionary) -> void:
	_require_component(snapshot_buffer, "snapshot_buffer")
	_require_component(client_world_state, "client_world_state")
	_require_component(render_adapter, "render_adapter")

	if bool(header.get("reconnect_full_snapshot", false)):
		snapshot_buffer.clear()
		client_world_state.clear()
		reconnect_pending_full_snapshot = false
		reconnect_phase = "delta_resume"

	snapshot_buffer.push_snapshot(header, payload)
	var snapshot: Dictionary = snapshot_buffer.latest()
	client_world_state.apply_snapshot(snapshot)
	render_adapter.update_from_world_state(client_world_state)


func begin_reconnect_resume(player_session_id: String, connection_id: String) -> Dictionary:
	reconnect_player_session_id = player_session_id
	reconnect_connection_id = connection_id
	reconnect_phase = "awaiting_full_snapshot"
	reconnect_pending_full_snapshot = true
	return reconnect_status()


func reconnect_status() -> Dictionary:
	return {
		"phase": reconnect_phase,
		"player_session_id": reconnect_player_session_id,
		"connection_id": reconnect_connection_id,
		"pending_full_snapshot": reconnect_pending_full_snapshot,
	}


func adapter_status() -> Dictionary:
	return {
		"pending_commands": command_queue.pending_count() if command_queue else 0,
		"buffered_snapshots": snapshot_buffer.snapshots.size() if snapshot_buffer else 0,
		"entities": client_world_state.entity_count() if client_world_state else 0,
		"render_records": render_adapter.snapshot_records().size() if render_adapter else 0,
		"reconnect_phase": reconnect_phase,
		"pending_full_snapshot": reconnect_pending_full_snapshot,
	}


func _require_component(component: Node, component_name: String) -> void:
	if component == null:
		push_error("%s missing %s" % [SYSTEM_NAME, component_name])


func _copy_selected_entity_ids(selected_entity_ids: Array) -> Array[int]:
	var copied: Array[int] = []
	for entity_id: Variant in selected_entity_ids:
		copied.append(int(entity_id))
	return copied


func _normalize_target(target: Dictionary) -> Dictionary:
	var normalized := target.duplicate(true)
	var kind := str(normalized.get("kind", TARGET_KIND_NONE))
	if not _is_allowed_target_kind(kind):
		return {"kind": TARGET_KIND_NONE}
	normalized["kind"] = kind
	return normalized


func _is_allowed_target_kind(kind: String) -> bool:
	return kind in [
		TARGET_KIND_NONE,
		"world_position",
		"entity_id",
		"map_marker_id",
		"cover_candidate_id",
	]
