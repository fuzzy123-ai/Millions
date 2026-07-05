# Fixture Tests

Cross-language fixtures live here when they are not raw protocol packet fixtures.

Use this area for map data export/import checks, scenario descriptors, and
shared expected outputs.

`cross-side-fixture-validation-matrix.json` records the PROTO-03 local
Rust/Godot fixture parity plan. It is a plan and evidence matrix, not a live
networking check.

`mapdata_v0_local_contract.json` is the MAPDATA-03 local export/import contract
fixture for server and Godot parity checks. Its `.checksum.json` sidecar stores a
`sum16_bytes` checksum so both sides can prove they read the same authored map
data fixture. The checksum is a deterministic local guard only; it is not a
cryptographic hash, live export proof, gameplay authority claim, or release
candidate artifact.
