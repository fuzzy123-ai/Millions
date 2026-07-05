extends SceneTree

const ClientWorldState := preload("res://scripts/net/ClientWorldState.gd")
const RenderAdapter := preload("res://scripts/render/RenderAdapter.gd")


func _init() -> void:
	var world_state := ClientWorldState.new()
	var render_adapter := RenderAdapter.new()

	root.add_child(world_state)
	root.add_child(render_adapter)

	world_state.apply_snapshot(
		{
			"tick": 40,
			"payload": {
				"entities": [
					{
						"entity_id": 3,
						"entity_kind": 2,
						"faction_id": 1,
						"x_mm": 3000,
						"y_mm": 1200,
						"health_q8": 200,
					},
					{
						"entity_id": 1,
						"entity_kind": 1,
						"faction_id": 1,
						"x_mm": 1000,
						"y_mm": 500,
						"health_q8": 256,
					},
					{
						"entity_id": 2,
						"entity_kind": 1,
						"faction_id": 1,
						"x_mm": 2000,
						"y_mm": 800,
						"health_q8": 128,
					},
					{
						"entity_id": 4,
						"entity_kind": 1,
						"faction_id": 2,
						"x_mm": 4000,
						"y_mm": 900,
						"health_q8": 64,
					},
				],
				"removed_entities": [],
			},
		}
	)

	render_adapter.update_from_world_state(world_state)

	var records: Array[Dictionary] = render_adapter.snapshot_records()
	if records.size() != 4:
		push_error("render record count mismatch: %s" % records)
		quit(1)
		return
	if int(records[0].get("entity_id", 0)) != 1:
		push_error("render record sort mismatch: %s" % records)
		quit(1)
		return
	if float(records[0].get("x_m", 0.0)) != 1.0:
		push_error("render record metres mismatch: %s" % records[0])
		quit(1)
		return

	var batches: Array[Dictionary] = render_adapter.render_batches()
	if batches.size() != 3:
		push_error("render batch count mismatch: %s" % batches)
		quit(1)
		return
	if String(batches[0].get("render_key", "")) != "kind:1:faction:1":
		push_error("first render batch key mismatch: %s" % batches)
		quit(1)
		return
	if int(batches[0].get("proxy_count", 0)) != 2:
		push_error("first render batch proxy count mismatch: %s" % batches[0])
		quit(1)
		return

	var first_batch_records: Array = batches[0].get("records", [])
	var copied_record: Dictionary = first_batch_records[0]
	copied_record["x_mm"] = 999999
	if int(world_state.entities[1].get("x_mm", 0)) == 999999:
		push_error("render batch leaked mutation into authoritative world state")
		quit(1)
		return

	world_state.apply_snapshot(
		{
			"tick": 41,
			"payload": {
				"entities": [],
				"removed_entities": [2],
			},
		}
	)
	render_adapter.update_from_world_state(world_state)
	if render_adapter.snapshot_records().size() != 3:
		push_error("removed entity still present in render records")
		quit(1)
		return
	if render_adapter.batch_count() != 3:
		push_error("batch count mismatch after removal")
		quit(1)
		return

	render_adapter.clear()
	if render_adapter.batch_count() != 0:
		push_error("batch count mismatch after clear")
		quit(1)
		return

	print("render_batch_check status=ok records=3 batches=3 copied=isolated")
	quit(0)
