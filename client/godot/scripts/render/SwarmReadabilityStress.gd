extends Node2D

const DEFAULT_PROXY_COUNT := 1000
const DEFAULT_LOD_BUCKETS := ["full", "reduced", "aggregate"]
const DEFAULT_AGGRO_LANES := ["direct", "memory", "route"]
const CELL_SIZE_PX := 24.0
const ENTITY_SPACING_PX := 5.0
const COLLISION_RADIUS_PX := 3.5

var proxy_host: Node2D


func _ready() -> void:
	if proxy_host == null:
		proxy_host = Node2D.new()
		proxy_host.name = "SwarmProxyHost"
		add_child(proxy_host)


func run_stress(proxy_count: int = DEFAULT_PROXY_COUNT) -> Dictionary:
	_ensure_host()
	_clear_proxies()

	var started_us: int = Time.get_ticks_usec()
	for index: int in range(proxy_count):
		var proxy: Node2D = Node2D.new()
		proxy.name = "SwarmProxy_%04d" % index
		proxy.position = _proxy_position(index)
		proxy.set_meta("entity_id", index + 50000)
		proxy.set_meta("ai_lod", _lod_for_index(index))
		proxy.set_meta("aggro_lane", _aggro_lane_for_index(index))
		proxy.set_meta("collision_radius_px", COLLISION_RADIUS_PX)
		proxy_host.add_child(proxy)
	var elapsed_us: int = Time.get_ticks_usec() - started_us

	return _readability_report(proxy_count, elapsed_us)


func _ensure_host() -> void:
	if proxy_host != null:
		return
	proxy_host = Node2D.new()
	proxy_host.name = "SwarmProxyHost"
	add_child(proxy_host)


func _proxy_position(index: int) -> Vector2:
	var lane: int = index % 25
	var row: int = int(index / 25)
	var wave_offset: int = int(row % 5)
	return Vector2(
		float(lane) * ENTITY_SPACING_PX + float(wave_offset) * 2.0,
		float(row) * ENTITY_SPACING_PX
	)


func _lod_for_index(index: int) -> String:
	if index < 250:
		return "full"
	if index < 750:
		return "reduced"
	return "aggregate"


func _aggro_lane_for_index(index: int) -> String:
	if index % 16 == 0:
		return "direct"
	if index % 16 == 1:
		return "memory"
	return "route"


func _readability_report(expected_proxy_count: int, elapsed_us: int) -> Dictionary:
	var bounds: Rect2 = Rect2()
	var first_proxy := true
	var cell_counts: Dictionary = {}
	var lod_counts: Dictionary = {"full": 0, "reduced": 0, "aggregate": 0}
	var aggro_counts: Dictionary = {"direct": 0, "memory": 0, "route": 0}
	var collision_radius_count := 0

	for child: Node in proxy_host.get_children():
		var proxy := child as Node2D
		if proxy == null:
			continue
		if first_proxy:
			bounds = Rect2(proxy.position, Vector2.ZERO)
			first_proxy = false
		else:
			bounds = bounds.expand(proxy.position)

		var cell_key: String = _cell_key(proxy.position)
		cell_counts[cell_key] = int(cell_counts.get(cell_key, 0)) + 1
		var lod: String = str(proxy.get_meta("ai_lod", ""))
		lod_counts[lod] = int(lod_counts.get(lod, 0)) + 1
		var aggro_lane: String = str(proxy.get_meta("aggro_lane", ""))
		aggro_counts[aggro_lane] = int(aggro_counts.get(aggro_lane, 0)) + 1
		if float(proxy.get_meta("collision_radius_px", 0.0)) > 0.0:
			collision_radius_count += 1

	var max_per_cell := 0
	for count: Variant in cell_counts.values():
		max_per_cell = maxi(max_per_cell, int(count))

	return {
		"expected_proxy_count": expected_proxy_count,
		"proxy_count": proxy_host.get_child_count(),
		"occupied_readability_cells": cell_counts.size(),
		"max_swarm_per_readability_cell": max_per_cell,
		"full_lod_count": int(lod_counts.get("full", 0)),
		"reduced_lod_count": int(lod_counts.get("reduced", 0)),
		"aggregate_lod_count": int(lod_counts.get("aggregate", 0)),
		"direct_aggro_count": int(aggro_counts.get("direct", 0)),
		"memory_aggro_count": int(aggro_counts.get("memory", 0)),
		"route_intent_count": int(aggro_counts.get("route", 0)),
		"collision_radius_count": collision_radius_count,
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


func _clear_proxies() -> void:
	for child: Node in proxy_host.get_children():
		proxy_host.remove_child(child)
		child.free()
