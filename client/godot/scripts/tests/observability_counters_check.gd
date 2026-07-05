extends SceneTree

const PerfLedger := preload("res://scripts/autoload/PerfLedger.gd")


func _init() -> void:
	var ledger := PerfLedger.new()
	root.add_child(ledger)

	if ledger.counter_value("commands_accepted") != 0:
		_fail("default counter should be zero")
		return

	if ledger.increment_counter("commands_accepted") != 1:
		_fail("first increment mismatch")
		return

	if ledger.increment_counter("commands_accepted", 2) != 3:
		_fail("delta increment mismatch")
		return

	ledger.set_metric("visible_entities", 128)
	if ledger.counter_value("visible_entities") != 128:
		_fail("set metric counter value mismatch")
		return

	if not ledger.has_counter("visible_entities"):
		_fail("expected visible_entities counter")
		return

	var snapshot: Dictionary = ledger.snapshot()
	if int(snapshot.get("commands_accepted", -1)) != 3:
		_fail("snapshot commands_accepted mismatch")
		return
	if int(snapshot.get("visible_entities", -1)) != 128:
		_fail("snapshot visible_entities mismatch")
		return

	ledger.reset()
	if ledger.has_counter("commands_accepted"):
		_fail("reset should clear commands_accepted")
		return

	print("observability_counters_check status=ok commands_accepted=3 visible_entities=128")
	quit(0)


func _fail(message: String) -> void:
	push_error(message)
	print("observability_counters_check status=failed")
	quit(1)
