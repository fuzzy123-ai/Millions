# Local Toolchain Setup

Date: 2026-07-02
Status: foundation toolchain gate clear, PATH refresh still recommended

## Current Local State

Observed from this workspace after setup on 2026-07-02:

- `rustc`: available at `C:\Users\nkatz\.cargo\bin\rustc.exe`, version `1.96.1`.
- `cargo`: available at `C:\Users\nkatz\.cargo\bin\cargo.exe`, version `1.96.1`.
- `godot`: available from the WinGet package path, version `4.7.stable.official.5b4e0cb0f`.
- `powershell`: available and required for repository scripts.
- `pwsh`: not found and optional.
- `dotnet`: found and optional.

The first Godot implementation path uses GDScript, so C#/.NET is not required
for the client spike.

If a shell was already open before installing Rust/Godot, restart it so the new
PATH entries and command aliases are visible. The repository environment check
also knows the default Rustup and WinGet install locations as a fallback and
reports whether each tool was found from `PATH` or a candidate path. No
repository script installs tools.

## Required Before Server Implementation

Rust is installed through Rustup on this machine. On a fresh machine, install
Rust using `rustup` from the official Rust project. After installation, open a
new terminal and run:

```powershell
rustc --version
cargo --version
cargo test
```

If `PATH` has not refreshed yet, use the installed absolute Cargo path for local
verification:

```powershell
C:\Users\nkatz\.cargo\bin\cargo.exe test
```

The repo pins stable Rust in `rust-toolchain.toml`.

## Required Before Godot Implementation

Godot 4.7 is installed through WinGet on this machine. On a fresh machine,
install Godot 4.x and make a `godot` command available in PATH, or record the
absolute executable path reported by `scripts/check_environment.ps1` in future
launcher config.

Initial checks after installation:

```powershell
godot --version
```

## Optional

PowerShell 7 (`pwsh`) is optional. The repository scripts are written for
Windows PowerShell 5.1 compatibility unless a later slice decides otherwise.

## Local Checks

Run environment discovery:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check_environment.ps1
```

Run foundation file checks:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check_foundation.ps1
```

Validate JSON planning files and their coupling:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\validate_plans.ps1
```

`check_environment.ps1` reports missing tools but does not install anything.
