extends SceneTree

const FIXTURE_PATH := "res://../../tests/fixtures/mapdata_v0_local_contract.json"
const CHECKSUM_PATH := "res://../../tests/fixtures/mapdata_v0_local_contract.checksum.json"
const EXPECTED_SCHEMA := "millions_mapdata_v0"
const EXPECTED_ALGORITHM := "sum16_bytes"


func _init() -> void:
	var failures: Array[String] = []
	var fixture_bytes: PackedByteArray = _read_bytes(FIXTURE_PATH, failures)
	var sidecar_bytes: PackedByteArray = _read_bytes(CHECKSUM_PATH, failures)

	if not fixture_bytes.is_empty() and not sidecar_bytes.is_empty():
		_check_fixture(fixture_bytes, sidecar_bytes, failures)

	if failures.is_empty():
		print("map_data_fixture_check status=ok fixtures=1 checksum=%s" % _sum16_checksum(fixture_bytes))
		quit(0)
		return

	for failure: String in failures:
		push_error(failure)
	print("map_data_fixture_check status=failed failures=%d" % failures.size())
	quit(1)


func _read_bytes(path: String, failures: Array[String]) -> PackedByteArray:
	var global_path: String = ProjectSettings.globalize_path(path)
	var bytes: PackedByteArray = FileAccess.get_file_as_bytes(global_path)
	if bytes.is_empty():
		failures.append("fixture unreadable: %s" % path)
	return bytes


func _check_fixture(fixture_bytes: PackedByteArray, sidecar_bytes: PackedByteArray, failures: Array[String]) -> void:
	var fixture_json: Variant = JSON.parse_string(fixture_bytes.get_string_from_utf8())
	var sidecar_json: Variant = JSON.parse_string(sidecar_bytes.get_string_from_utf8())

	if typeof(fixture_json) != TYPE_DICTIONARY:
		failures.append("mapdata fixture is not a JSON object")
		return
	if typeof(sidecar_json) != TYPE_DICTIONARY:
		failures.append("mapdata checksum sidecar is not a JSON object")
		return

	var fixture: Dictionary = fixture_json as Dictionary
	var sidecar: Dictionary = sidecar_json as Dictionary
	var checksum: String = _sum16_checksum(fixture_bytes)

	if str(fixture.get("schema", "")) != EXPECTED_SCHEMA:
		failures.append("schema mismatch: %s" % fixture.get("schema", null))
	if str(sidecar.get("algorithm", "")) != EXPECTED_ALGORITHM:
		failures.append("checksum algorithm mismatch: %s" % sidecar.get("algorithm", null))
	if str(sidecar.get("expected_checksum", "")) != checksum:
		failures.append("checksum mismatch: got %s expected %s" % [checksum, sidecar.get("expected_checksum", null)])

	for key: String in ["spawn_points", "capture_points", "obstacles", "cover_objects", "navigation_hints"]:
		var value: Variant = fixture.get(key, null)
		if typeof(value) != TYPE_ARRAY or value.is_empty():
			failures.append("%s missing or empty" % key)

	var non_claims: Variant = fixture.get("non_claims", [])
	if typeof(non_claims) != TYPE_ARRAY:
		failures.append("non_claims must be an array")
		return
	for required_claim: String in ["not_gameplay_authority", "not_live", "not_release_candidate"]:
		if not non_claims.has(required_claim):
			failures.append("missing non_claim: %s" % required_claim)


func _sum16_checksum(bytes: PackedByteArray) -> String:
	var sum: int = 0
	for byte: int in bytes:
		sum = (sum + byte) % 65535
	return "sum16:%04x" % sum
