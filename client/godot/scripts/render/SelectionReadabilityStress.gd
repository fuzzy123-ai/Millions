extends Node2D

const DEFAULT_PROXY_COUNT := 1000
const DEFAULT_SELECTED_COUNT := 128
const GRID_COLUMNS := 50
const PROXY_SPACING_PX := 7.0
const READABILITY_CELL_PX := 28.0
const FEEDBACK_STATE_SELECTION := "selection_local"
const FEEDBACK_STATE_CONTEXT := "command_context_ready"

var proxy_host: Node2D
var overlay_host: Node2D


func _ready() -> void:
	_ensure_hosts()


func run_stress(
	proxy_count: int = DEFAULT_PROXY_COUNT,
	selected_count: int = DEFAULT_SELECTED_COUNT
) -> Dictionary:
	_ensure_hosts()
	_clear_children(proxy_host)
	_clear_children(overlay_host)

	var started_us: int = Time.get_ticks_usec()
	for index: int in range(proxy_count):
		var proxy := Node2D.new()
		proxy.name = "SelectableProxy_%04d" % index
		proxy.position = _proxy_position(index)
		proxy.set_meta("entity_id", index + 1)
		proxy.set_meta("visible_authoritative_proxy", true)
		proxy_host.add_child(proxy)

	var selected_ids := _deterministic_selection(proxy_count, selected_count)
	for entity_id: int in selected_ids:
		var overlay := Node2D.new()
		overlay.name = "SelectionOverlay_%04d" % entity_id
		overlay.position = _proxy_position(entity_id - 1)
		overlay.set_meta("selected_entity_id", entity_id)
		overlay.set_meta("feedback_state", FEEDBACK_STATE_SELECTION)
		overlay.set_meta("command_context_state", FEEDBACK_STATE_CONTEXT)
		overlay_host.add_child(overlay)
	var elapsed_us: int = Time.get_ticks_usec() - started_us

	return _readability_report(proxy_count, selected_count, elapsed_us)


func _ensure_hosts() -> void:
	if proxy_host == null:
		proxy_host = Node2D.new()
		proxy_host.name = "SelectionProxyHost"
		add_child(proxy_host)
	if overlay_host == null:
		overlay_host = Node2D.new()
		overlay_host.name = "SelectionOverlayHost"
		add_child(overlay_host)


func _proxy_position(index: int) -> Vector2:
	var column: int = index % GRID_COLUMNS
	var row: int = int(index / GRID_COLUMNS)
	return Vector2(float(column) * PROXY_SPACING_PX, float(row) * PROXY_SPACING_PX)


func _deterministic_selection(proxy_count: int, selected_count: int) -> Array[int]:
	var selected: Array[int] = []
	var used := {}
	var cursor := 0
	while selected.size() < selected_count and selected.size() < proxy_count:
		var entity_id := (cursor * 7) % proxy_count + 1
		if not used.has(entity_id):
			used[entity_id] = true
			selected.append(entity_id)
		cursor += 1
	return selected


func _readability_report(
	expected_proxy_count: int,
	expected_selected_count: int,
	elapsed_us: int
) -> Dictionary:
	var cell_counts := {}
	var selection_feedback_count := 0
	var context_ready_count := 0
	var bounds := Rect2()
	var first_overlay := true

	for child: Node in overlay_host.get_children():
		var overlay := child as Node2D
		if overlay == null:
			continue
		if str(overlay.get_meta("feedback_state", "")) == FEEDBACK_STATE_SELECTION:
			selection_feedback_count += 1
		if str(overlay.get_meta("command_context_state", "")) == FEEDBACK_STATE_CONTEXT:
			context_ready_count += 1
		if first_overlay:
			bounds = Rect2(overlay.position, Vector2.ZERO)
			first_overlay = false
		else:
			bounds = bounds.expand(overlay.position)
		var cell_key := _cell_key(overlay.position)
		cell_counts[cell_key] = int(cell_counts.get(cell_key, 0)) + 1

	var max_per_cell := 0
	for count: Variant in cell_counts.values():
		max_per_cell = maxi(max_per_cell, int(count))

	return {
		"expected_proxy_count": expected_proxy_count,
		"proxy_count": proxy_host.get_child_count(),
		"expected_selected_count": expected_selected_count,
		"selection_overlay_count": overlay_host.get_child_count(),
		"selection_feedback_count": selection_feedback_count,
		"command_context_ready_count": context_ready_count,
		"occupied_readability_cells": cell_counts.size(),
		"max_overlays_per_readability_cell": max_per_cell,
		"elapsed_us": elapsed_us,
		"bounds_min_x": bounds.position.x,
		"bounds_min_y": bounds.position.y,
		"bounds_max_x": bounds.end.x,
		"bounds_max_y": bounds.end.y,
		"claim_scope": "informational_contract_only",
	}


func _cell_key(position: Vector2) -> String:
	var cell_x: int = floori(position.x / READABILITY_CELL_PX)
	var cell_y: int = floori(position.y / READABILITY_CELL_PX)
	return "%d:%d" % [cell_x, cell_y]


func _clear_children(parent: Node) -> void:
	for child: Node in parent.get_children():
		parent.remove_child(child)
		child.free()

