extends Node2D

const DEFAULT_ENTITY_COUNT := 1000
const DEFAULT_GROUP_COUNT := 20
const DEFAULT_LANE_COUNT := 10
const CELL_SIZE_PX := 24.0
const ENTITY_SPACING_PX := 6.0

var proxy_host: Node2D


func _ready() -> void:
	if proxy_host == null:
		proxy_host = Node2D.new()
		proxy_host.name = "MovementProxyHost"
		add_child(proxy_host)


func run_stress(
	entity_count: int = DEFAULT_ENTITY_COUNT,
	group_count: int = DEFAULT_GROUP_COUNT,
	lane_count: int = DEFAULT_LANE_COUNT
) -> Dictionary:
	_ensure_host()
	_clear_proxies()

	var started_us: int = Time.get_ticks_usec()
	for index: int in range(entity_count):
		var proxy: Node2D = Node2D.new()
		proxy.name = "MoveProxy_%04d" % index
		proxy.position = _proxy_position(index, group_count, lane_count)
		proxy.set_meta("entity_id", index + 1)
		proxy.set_meta("movement_group", index % group_count)
		proxy.set_meta("lane", index % lane_count)
		proxy_host.add_child(proxy)
	var elapsed_us: int = Time.get_ticks_usec() - started_us

	return _readability_report(entity_count, group_count, lane_count, elapsed_us)


func _ensure_host() -> void:
	if proxy_host != null:
		return
	proxy_host = Node2D.new()
	proxy_host.name = "MovementProxyHost"
	add_child(proxy_host)


func _proxy_position(index: int, group_count: int, lane_count: int) -> Vector2:
	var group: int = index % group_count
	var lane: int = index % lane_count
	var row: int = int(index / group_count)
	var group_band: int = int(group / lane_count)
	return Vector2(
		float(lane) * CELL_SIZE_PX + float(row % 4) * ENTITY_SPACING_PX,
		float(row) * ENTITY_SPACING_PX + float(group_band) * CELL_SIZE_PX
	)


func _readability_report(
	expected_proxy_count: int,
	group_count: int,
	lane_count: int,
	elapsed_us: int
) -> Dictionary:
	var bounds: Rect2 = Rect2()
	var first_proxy := true
	var cell_counts: Dictionary = {}
	var group_counts: Array[int] = []
	for group_index: int in range(group_count):
		group_counts.append(0)

	for child: Node in proxy_host.get_children():
		var proxy := child as Node2D
		if proxy == null:
			continue
		if first_proxy:
			bounds = Rect2(proxy.position, Vector2.ZERO)
			first_proxy = false
		else:
			bounds = bounds.expand(proxy.position)

		var group: int = int(proxy.get_meta("movement_group", 0))
		group_counts[group] += 1
		var cell_key: String = _cell_key(proxy.position)
		cell_counts[cell_key] = int(cell_counts.get(cell_key, 0)) + 1

	var max_per_cell := 0
	for count: Variant in cell_counts.values():
		max_per_cell = maxi(max_per_cell, int(count))

	return {
		"expected_proxy_count": expected_proxy_count,
		"proxy_count": proxy_host.get_child_count(),
		"group_count": group_count,
		"lane_count": lane_count,
		"occupied_readability_cells": cell_counts.size(),
		"max_entities_per_readability_cell": max_per_cell,
		"groups_with_members": _non_empty_group_count(group_counts),
		"elapsed_us": elapsed_us,
		"bounds_min_x": bounds.position.x,
		"bounds_min_y": bounds.position.y,
		"bounds_max_x": bounds.end.x,
		"bounds_max_y": bounds.end.y,
		"claim_scope": "informational_contract_only",
	}


func _cell_key(position: Vector2) -> String:
	var cell_x: int = floori(position.x / CELL_SIZE_PX)
	var cell_y: int = floori(position.y / CELL_SIZE_PX)
	return "%d:%d" % [cell_x, cell_y]


func _non_empty_group_count(group_counts: Array[int]) -> int:
	var non_empty := 0
	for count: int in group_counts:
		if count > 0:
			non_empty += 1
	return non_empty


func _clear_proxies() -> void:
	for child: Node in proxy_host.get_children():
		proxy_host.remove_child(child)
		child.free()
