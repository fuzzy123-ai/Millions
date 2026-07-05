extends SceneTree

const REQUIRED_DIRS := [
	"res://scenes/app",
	"res://scenes/lobby",
	"res://scenes/match",
	"res://scenes/ui",
	"res://scenes/gameplay",
	"res://scenes/dev",
	"res://resources/players",
	"res://resources/protocol",
	"res://resources/render",
	"res://resources/ui",
]

const REQUIRED_FILES := [
	"res://README.md",
	"res://../../docs/architecture/godot-scene-node-contract.md",
	"res://../../docs/architecture/godot-reusable-scene-resource-contracts.md",
	"res://../../docs/runbooks/godot-slice-scene-checklist.md",
]

const REQUIRED_CONTRACT_TERMS := [
	"Reusable Scene Rules",
	"Resource Rules",
	"Runtime Instancing Rules",
	"Protocol And Authority Boundary",
	"GSCENE-03 Check Surface",
]


func _init() -> void:
	var failures: Array[String] = []
	for path: String in REQUIRED_DIRS:
		if not DirAccess.dir_exists_absolute(ProjectSettings.globalize_path(path)):
			failures.append("missing required Godot directory: %s" % path)

	for path: String in REQUIRED_FILES:
		if not FileAccess.file_exists(ProjectSettings.globalize_path(path)):
			failures.append("missing required contract file: %s" % path)

	var contract_path := ProjectSettings.globalize_path("res://../../docs/architecture/godot-reusable-scene-resource-contracts.md")
	var contract_text := FileAccess.get_file_as_string(contract_path)
	if contract_text.is_empty():
		failures.append("reusable scene/resource contract unreadable")
	else:
		for term: String in REQUIRED_CONTRACT_TERMS:
			if contract_text.find(term) == -1:
				failures.append("missing reusable contract term: %s" % term)

	if failures.is_empty():
		print("scene_resource_contract_check status=ok dirs=%d files=%d terms=%d" % [
			REQUIRED_DIRS.size(),
			REQUIRED_FILES.size(),
			REQUIRED_CONTRACT_TERMS.size(),
		])
		quit(0)
		return

	for failure: String in failures:
		push_error(failure)
	print("scene_resource_contract_check status=failed failures=%d" % failures.size())
	quit(1)
