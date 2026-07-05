# Protocol Fixtures

Fixture naming convention:

```text
protocol_v0_<scenario>_<expected>.bin
protocol_v0_<scenario>_<expected>.json
```

The JSON descriptor explains expected fields and checksums. The binary file is
the canonical packet bytes.

Initial scenarios:

- `server_hello_accept`: server accepts a local/mock session and advertises tick rate.
- `command_ready_batch`: client sends one ready intent command with seq/ack.
- `snapshot_full_minimal`: server sends one authoritative entity record.
- `local_mock_steam_bridge_handoff`: JSON-only facade-shaped Steam bridge
  handoff for local/mock STEAM-02 validation.
- `command_duplicate_ignored`: planned for reliability/idempotency slices.
- `snapshot_delta_minimal`: planned for delta/supersession slices.
- `reconnect_full_snapshot`: planned for reconnect slices.
- `malformed_rejected`: planned for hardening slices.

Binary fixtures are canonical packet bytes. JSON descriptors are explanatory and
must not contain secrets, Steam auth tickets, private account data, or raw
provider/session tokens.

Cross-side validation is documented in
`docs/protocol/godot-rust-fixture-contract.md`.

Build evidence for fixture/replay comparisons is documented in
`docs/architecture/reproducible-builds.md`.
