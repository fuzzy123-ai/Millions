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

	var ready_command: Dictionary = adapter.queue_ready_intent(10)
	if int(ready_command.get("client_seq", 0)) != 1:
		push_error("ready command seq mismatch")
		quit(1)
		return
	if int(ready_command.get("command_id", 0)) != 1:
		push_error("ready command id mismatch")
		quit(1)
		return

	adapter.accept_snapshot(
		{
			"server_seq": 2,
			"tick": 20,
			"message_type": 5,
		},
		{
			"entities": [
				{
					"entity_id": 1,
					"entity_kind": 1,
					"faction_id": 1,
					"x_mm": 100,
					"y_mm": 50,
					"health_q8": 256,
				}
			],
			"removed_entities": [],
		}
	)

	var status: Dictionary = adapter.adapter_status()
	if int(status.get("pending_commands", -1)) != 1:
		push_error("pending command count mismatch")
		quit(1)
		return
	if int(status.get("buffered_snapshots", -1)) != 1:
		push_error("buffered snapshot count mismatch")
		quit(1)
		return
	if int(status.get("entities", -1)) != 1:
		push_error("entity count mismatch")
		quit(1)
		return
	if int(status.get("render_records", -1)) != 1:
		push_error("render record count mismatch")
		quit(1)
		return
	if render_adapter.batch_count() != 1:
		push_error("render batch count mismatch")
		quit(1)
		return

	var reconnect: Dictionary = adapter.begin_reconnect_resume("local-session-local-abc123-player-1", "connection-2")
	if reconnect.get("phase") != "awaiting_full_snapshot":
		push_error("reconnect phase mismatch before full snapshot")
		quit(1)
		return
	if not bool(reconnect.get("pending_full_snapshot", false)):
		push_error("reconnect pending full snapshot mismatch")
		quit(1)
		return

	adapter.accept_snapshot(
		{
			"server_seq": 3,
			"tick": 30,
			"message_type": 5,
			"reconnect_full_snapshot": true,
		},
		{
			"entities": [
				{
					"entity_id": 2,
					"entity_kind": 1,
					"faction_id": 1,
					"x_mm": 500,
					"y_mm": 250,
					"health_q8": 256,
				}
			],
			"removed_entities": [],
		}
	)

	reconnect = adapter.reconnect_status()
	if reconnect.get("phase") != "delta_resume":
		push_error("reconnect phase mismatch after full snapshot")
		quit(1)
		return
	if bool(reconnect.get("pending_full_snapshot", true)):
		push_error("reconnect pending full snapshot should be false")
		quit(1)
		return
	if world_state.entities.has(1):
		push_error("reconnect full snapshot did not clear stale entity")
		quit(1)
		return

	adapter.accept_snapshot(
		{
			"server_seq": 4,
			"tick": 31,
			"message_type": 6,
		},
		{
			"entities": [
				{
					"entity_id": 3,
					"entity_kind": 1,
					"faction_id": 1,
					"x_mm": 700,
					"y_mm": 300,
					"health_q8": 256,
				}
			],
			"removed_entities": [],
		}
	)

	status = adapter.adapter_status()
	if int(status.get("entities", -1)) != 2:
		push_error("delta resume entity count mismatch")
		quit(1)
		return
	if status.get("reconnect_phase") != "delta_resume":
		push_error("adapter status reconnect phase mismatch")
		quit(1)
		return

	var resend: Array[Dictionary] = command_queue.commands_for_resend(0)
	if resend.size() != 1:
		push_error("resend command count mismatch")
		quit(1)
		return
	var resent: Dictionary = command_queue.mark_resend_attempt(1)
	if int(resent.get("resend_count", 0)) != 1:
		push_error("resend count mismatch")
		quit(1)
		return
	if command_queue.ack_through(1) != 1:
		push_error("ack removal mismatch")
		quit(1)
		return
	status = adapter.adapter_status()
	if int(status.get("pending_commands", -1)) != 0:
		push_error("pending command count after ack mismatch")
		quit(1)
		return

	print("client_adapter_check status=ok pending=0 snapshots=2 entities=2 render_records=2 render_batches=%d reconnect=delta_resume resend=1 acked=1" % render_adapter.batch_count())
	quit(0)
