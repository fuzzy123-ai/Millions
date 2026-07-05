extends Node

signal connection_state_changed(state: String)

const INITIAL_STATE := "offline"

var connection_state: String = INITIAL_STATE


func set_connection_state(next_state: String) -> void:
	if connection_state == next_state:
		return

	connection_state = next_state
	connection_state_changed.emit(connection_state)
