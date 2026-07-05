extends Node2D

const ClientAdapter := preload("res://scripts/net/ClientAdapter.gd")
const CommandQueue := preload("res://scripts/net/CommandQueue.gd")
const SnapshotBuffer := preload("res://scripts/net/SnapshotBuffer.gd")
const ClientWorldState := preload("res://scripts/net/ClientWorldState.gd")
const RenderAdapter := preload("res://scripts/render/RenderAdapter.gd")

@onready var render_proxy_host: Node2D = $RuntimeEntities/RenderProxyHost


func run_smoke() -> Dictionary:
	var adapter := ClientAdapter.new()
	var command_queue := CommandQueue.new()
	var snapshot_buffer := SnapshotBuffer.new()
	var world_state := ClientWorldState.new()
	var render_adapter := RenderAdapter.new()

	add_child(adapter)
	add_child(command_queue)
	add_child(snapshot_buffer)
	add_child(world_state)
	add_child(render_adapter)

	adapter.configure_components(command_queue, snapshot_buffer, world_state, render_adapter)
	adapter.queue_ready_intent(10)
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

	_clear_render_proxies()
	for batch: Dictionary in render_adapter.render_batches():
		for record: Dictionary in batch.get("records", []):
			var proxy := Node2D.new()
			proxy.name = "Proxy_%s" % int(record.get("entity_id", 0))
			proxy.position = record.get("position_m", Vector2.ZERO)
			proxy.set_meta("render_key", String(batch.get("render_key", "")))
			render_proxy_host.add_child(proxy)

	return {
		"proxy_count": render_proxy_host.get_child_count(),
		"batch_count": render_adapter.batch_count(),
		"entity_count": world_state.entity_count(),
		"pending_commands": command_queue.pending_count(),
	}


func _clear_render_proxies() -> void:
	for child: Node in render_proxy_host.get_children():
		child.queue_free()
