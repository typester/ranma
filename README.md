# Ranma (欄間)

A programmable macOS menu bar overlay that fills the unused center space with custom widgets.

ranma places dynamically-sized floating windows on the menu bar that cover only their content, so click-through works correctly on uncovered regions. Widgets are simple scripts in any language that communicate via CLI over Unix Domain Socket IPC.

![yashiki-workspace](assets/statusbar-yashiki-workspace.png)
![unified](assets/statusbar-unified.png)
![unified-light](assets/statusbar-unified-light.png)
![unified-neon](assets/statusbar-unified-neon.png)
![unified-retro](assets/statusbar-unified-retro.png)
![unified-pastel](assets/statusbar-unified-pastel.png)
![unified-transparent](assets/statusbar-unified-transparent.png)
![island](assets/statusbar-island.png)
![all](assets/statusbar-all.png)

## Features

- Dynamically-sized floating windows that only cover content area
- Click-through works correctly on uncovered regions
- Multi-monitor support with per-display containers
- Notch-aware positioning (left/right alignment)
- Scriptable via CLI — write widgets in any language
- Multiple layout styles: individual pills, unified bar, Dynamic Island

## Architecture

- **Rust** — Core logic, state management, IPC server, CLI
- **Swift/AppKit** — Native macOS UI rendering
- **UniFFI** — Bridges Rust and Swift

Client-server model: `ranma-server` manages windows, `ranma` CLI sends JSON commands over a Unix Domain Socket.

## Installation

### Homebrew

```sh
brew install typester/ranma/ranma
```

This installs both `ranma-server` and `ranma` CLI.

### Building from Source

Prerequisites: Rust toolchain, Xcode Command Line Tools

```sh
./scripts/build.sh        # debug build
./scripts/build.sh release # release build
```

This produces two binaries:
- `ranma-server` — The status bar application
- `ranma` — CLI controller

## Usage

Start the server with an init script:

```sh
ranma-server start --init ./examples/unified/init
```

The init script launches widget processes that use the `ranma` CLI to add and update status bar items.

### CLI Commands

```sh
ranma add <name> --label "text" --icon "sf.symbol" --display N
ranma set <name> --label "new" --display N
ranma remove <name>
ranma query [name] --display N
ranma displays
```

## Examples

See [examples/README.md](examples/README.md) for available example widgets.

## Writing Your Own Widget

See [docs/widget-guide.md](docs/widget-guide.md) for a comprehensive guide covering the layout model, CLI reference, styling properties, and common patterns.

## License

MIT
