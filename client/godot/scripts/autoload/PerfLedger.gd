extends Node

var counters: Dictionary = {}


func set_metric(metric_name: String, value: Variant) -> void:
	counters[metric_name] = value


func increment_counter(counter_name: String, delta: int = 1) -> int:
	var next_value := int(counters.get(counter_name, 0)) + delta
	counters[counter_name] = next_value
	return next_value


func counter_value(counter_name: String) -> int:
	return int(counters.get(counter_name, 0))


func has_counter(counter_name: String) -> bool:
	return counters.has(counter_name)


func snapshot() -> Dictionary:
	return counters.duplicate(true)


func reset() -> void:
	counters.clear()
