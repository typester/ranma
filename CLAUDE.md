# CLAUDE.md

## Language

- All generated artifacts (code, documentation, commit messages, etc.) MUST be in English unless explicitly specified otherwise.

## Version Control

- This project uses **jj** (Jujutsu) instead of git for version control.
- Write operations (commit, push, etc.) should be performed by the user unless explicitly instructed otherwise.

## Workflow

- **ALWAYS re-read CLAUDE.md before starting any task.** This ensures you follow the latest project rules and conventions.
- **ALWAYS plan before coding.** Unless explicitly instructed otherwise, use plan mode to propose changes and get user approval before editing any files.
- **When writing a plan file in plan mode, the first step MUST always be "Step 0: Re-read CLAUDE.md and verify all workflow rules."** This step must appear in the plan file itself, not just be done mentally. No plan is complete without it.
- Do NOT start writing or editing code without a plan approved by the user.
- **When the user reports a problem with generated output**, investigate the issue first, then propose a fix via plan mode. Do NOT edit any files until the fix plan is approved by the user. This applies equally to bug reports, visual issues, and any other feedback on previously generated code.

## Code Style

- Keep code comments minimal. Only add comments where the logic is genuinely unclear.
- NEVER add comments that merely restate what the immediately following code does.
- **ALWAYS run `cargo fmt` after modifying any Rust files.** CI checks formatting via `cargo fmt --all --check`.
- **ALWAYS run `cargo clippy` before considering any task complete.** Fix all warnings before finishing.

## Project Architecture

- **ranma** is a programmable macOS status bar overlay (inspired by SketchyBar).
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

### Multi-Monitor
- Each item has a `display: u32` field — the `CGDirectDisplayID` of the target screen.
- `BarState` stores items per-display: `HashMap<u32, Vec<BarItem>>`.
- Windows are created lazily: no bar is visible until the first item is added for a display.
- Windows are destroyed when all items for a display are removed.
- `RanmaAppDelegate` observes `didChangeScreenParametersNotification` and calls `setDisplays()` to sync the display list to Rust.
- `StateChangeEvent::ItemMoved` handles cross-display item moves.

### IPC Protocol
- Unix Domain Socket at `$TMPDIR/ranma_<uid>.sock`.
- Newline-delimited JSON. Commands: `add`, `set`, `remove`, `query`, `displays`.

### CLI
- Uses **argh** for argument parsing (user preference). Subcommand-based, no `key=value` syntax.
- `ranma add <name> --label "text" --icon "sf.symbol" --display N`
- `ranma set <name> --label "new" --display N`
- `ranma remove <name>`
- `ranma query [name] --display N`
- `ranma displays`

### UniFFI Details
- Version: 0.29.x (proc-macro based, `uniffi::setup_scaffolding!()` in lib.rs).
- Binding generation: `cargo run -p uniffi-bindgen -- generate --library <path> --language swift --out-dir <dir>`.
- `StateChangeHandler` trait uses `#[uniffi::export(with_foreign)]` — Swift implements the protocol.
- Error type `RanmaError` must impl `From<uniffi::UnexpectedUniFFICallbackError>`.

## Memory

- Any information that should persist across sessions MUST be recorded by updating this CLAUDE.md file.
