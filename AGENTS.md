# AGENTS.md

## Language

- All generated artifacts (code, documentation, commit messages, etc.) MUST be in English unless explicitly specified otherwise.

## Version Control

- This project uses **jj** (Jujutsu) instead of git for version control.
- Write operations (commit, push, etc.) should be performed by the user unless explicitly instructed otherwise.

## Code Style

- Keep code comments minimal. Only add comments where the logic is genuinely unclear.
- NEVER add comments that merely restate what the immediately following code does.

## Project Architecture

- **ranma** is a programmable macOS status bar replacement (inspired by SketchyBar).
- Key innovation: dynamically-sized floating window covering only content, solving the click-through problem.
- Architecture: Client-Server via Unix Domain Socket IPC.

### Tech Stack
- **Rust** (core logic, state, IPC server, CLI) — Cargo workspace at repo root.
- **Swift/AppKit** (native macOS UI) — Swift Package at `app/`.
- **UniFFI** (proc-macro based, NOT UDL) bridges Rust and Swift.

### Crate Layout
- `crates/ranma-core/` — Library (`staticlib` + `lib`). State management, IPC server, UniFFI exports.
- `crates/ranma-cli/` — Binary (`ranma`). CLI controller that sends JSON commands over UDS.
- `crates/uniffi-bindgen/` — Binary for generating Swift bindings from the compiled static library.

### Swift App (`app/`)
- SPM package with a `systemLibrary` target (`CRanmaCore`) for the UniFFI C header/modulemap.
- `BarWindow` — borderless NSWindow at level 25 (kCGStatusWindowLevel), above menu bar.
- `BarViewModel` — implements UniFFI-generated `StateChangeHandler` protocol (`@unchecked Sendable`).
- `BarContentView` — draws rounded pill background, SF Symbol icons, text labels.
- Never set `ignoresMouseEvents` — preserves default per-pixel hit testing.

### Build System
- Single entry point: `./scripts/build.sh [debug|release]`
- Builds Rust → generates UniFFI Swift bindings → copies headers → builds Swift app.
- Generated files go to `app/Sources/Generated/` and `app/Sources/CRanmaCore/include/`.

### IPC Protocol
- Unix Domain Socket at `$TMPDIR/ranma_<uid>.sock`.
- Newline-delimited JSON. Commands: `add`, `set`, `remove`, `query`.
- CLI usage: `ranma --add <name> [key=value ...]`, `ranma --set <name> key=value ...`, `ranma --remove <name>`, `ranma --query [name]`.

### UniFFI Details
- Version: 0.29.x (proc-macro based, `uniffi::setup_scaffolding!()` in lib.rs).
- Binding generation: `cargo run -p uniffi-bindgen -- generate --library <path> --language swift --out-dir <dir>`.
- `StateChangeHandler` trait uses `#[uniffi::export(with_foreign)]` — Swift implements the protocol.
- Error type `RanmaError` must impl `From<uniffi::UnexpectedUniFFICallbackError>`.

## Memory

- Any information that should persist across sessions MUST be recorded by updating this AGENTS.md file.
