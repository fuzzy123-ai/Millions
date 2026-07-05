extends Node

const CONTEXT_CLIENT := "client"


func info(system_name: String, message: String, fields: Dictionary = {}) -> void:
	_write("info", system_name, message, fields)


func warn(system_name: String, message: String, fields: Dictionary = {}) -> void:
	_write("warn", system_name, message, fields)


func _write(level: String, system_name: String, message: String, fields: Dictionary) -> void:
	var record := {
		"level": level,
		"context": CONTEXT_CLIENT,
		"system": system_name,
		"message": message,
		"fields": fields,
	}
	print(JSON.stringify(record))
