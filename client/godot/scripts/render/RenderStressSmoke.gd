extends Node2D

const DEFAULT_PROXY_COUNT := 1000
const GRID_COLUMNS := 40
const PROXY_SPACING := 8.0

@onready var render_proxy_host: Node2D = $RuntimeEntities/RenderProxyHost


func run_stress(proxy_count: int = DEFAULT_PROXY_COUNT) -> Dictionary:
	_clear_render_proxies()

	var started_us := Time.get_ticks_usec()
	for index: int in range(proxy_count):
		var proxy := Node2D.new()
		proxy.name = "Proxy_%04d" % index
		proxy.position = _proxy_position(index)
		proxy.set_meta("entity_id", index + 1)
		proxy.set_meta("entity_kind", index % 8)
		render_proxy_host.add_child(proxy)
	var elapsed_us := Time.get_ticks_usec() - started_us

	return _stress_report(proxy_count, elapsed_us)


func _proxy_position(index: int) -> Vector2:
	var column := index % GRID_COLUMNS
	var row := int(index / GRID_COLUMNS)
	return Vector2(float(column) * PROXY_SPACING, float(row) * PROXY_SPACING)


func _stress_report(expected_proxy_count: int, elapsed_us: int) -> Dictionary:
	var bounds := Rect2()
	var first := true
	for child: Node in render_proxy_host.get_children():
		var node_2d := child as Node2D
		if node_2d == null:
			continue
		if first:
			bounds = Rect2(node_2d.position, Vector2.ZERO)
			first = false
		else:
			bounds = bounds.expand(node_2d.position)

	return {
		"expected_proxy_count": expected_proxy_count,
		"proxy_count": render_proxy_host.get_child_count(),
		"elapsed_us": elapsed_us,
		"grid_columns": GRID_COLUMNS,
		"proxy_spacing": PROXY_SPACING,
		"bounds_min_x": bounds.position.x,
		"bounds_min_y": bounds.position.y,
		"bounds_max_x": bounds.end.x,
		"bounds_max_y": bounds.end.y,
	}


func _clear_render_proxies() -> void:
	for child: Node in render_proxy_host.get_children():
		render_proxy_host.remove_child(child)
		child.free()
