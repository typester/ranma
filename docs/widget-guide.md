# Widget Development Guide

This guide covers everything you need to build your own ranma widgets.

## How It Works

ranma uses a client-server architecture. The `ranma-server` process manages windows. Widgets are separate processes that use the `ranma` CLI to add, update, and remove nodes in the status bar.

A widget's lifecycle is simple:

1. Create your node tree (`ranma add`)
2. Update nodes in a loop (`ranma set`)
3. Exit when done — the `reap` tool handles cleanup

## Node Types

There are 4 node types that form a tree:

| Type | Description |
|------|-------------|
| `item` | Leaf node. Displays a text label and/or SF Symbol icon. |
| `row` | Container. Lays out children **horizontally**. |
| `column` | Container. Lays out children **vertically**. |
| `box` | Container. **Stacks** children on top of each other (z-stack). |

**Top-level nodes** (those without a `--parent`) become separate floating windows on the menu bar.

## Layout Model

### Row (horizontal)

```
┌─────────────────────────────┐
│ [child1] [child2] [child3]  │
└─────────────────────────────┘
```

Children are placed left to right with `--gap` spacing between them.

- `--align-items` controls **vertical** alignment: `start` (top), `center`, `end` (bottom)
- `--justify-content` controls **horizontal** alignment: `start`, `center`, `end`

### Column (vertical)

```
┌──────────┐
│ [child1]  │
│ [child2]  │
│ [child3]  │
└──────────┘
```

Children are placed top to bottom with `--gap` spacing.

- `--align-items` controls **horizontal** alignment: `start` (left), `center`, `end` (right)
- `--justify-content` controls **vertical** alignment: `start`, `center`, `end`

### Box (overlay/z-stack)

```
┌──────────┐
│ [child3]  │  ← drawn on top
│ [child2]  │
│ [child1]  │  ← drawn first (bottom)
└──────────┘
```

All children share the same origin. Higher `--position` values are drawn on top. Useful for layered UIs like workspace indicators with a background pill, centered label, and underline indicator.

### Sizing

- Containers auto-size to fit their children plus padding.
- Set `--width` and/or `--height` to override auto-sizing.
- Items auto-size to fit their label/icon plus padding.

## Quick Start: Minimal Widget

Here's the simplest possible widget — a static label:

```sh
#!/bin/sh
ranma add hello --label "Hello, World!" \
  --background-color "#0a0a0f" \
  --label-color "#ffffff" \
  --corner-radius 8 \
  --padding-horizontal 10 \
  --padding-vertical 4
```

A more realistic widget with live updates:

```python
#!/usr/bin/env python3
import subprocess, time

def ranma_add(name, **kwargs):
    cmd = ["ranma", "add", name]
    for k, v in kwargs.items():
        cmd += [f"--{k.replace('_', '-')}", str(v)]
    subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def ranma_set(name, **kwargs):
    cmd = ["ranma", "set", name]
    for k, v in kwargs.items():
        cmd += [f"--{k.replace('_', '-')}", str(v)]
    subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

# Create the structure once
ranma_add("mybar", type="row",
          background_color="#0a0a0f", corner_radius="8",
          padding_horizontal="10", padding_vertical="4",
          gap="4", align_items="center")

ranma_add("mybar.label", parent="mybar",
          label="Loading...", label_color="#ffffff", font_size="11")

# Update in a loop
while True:
    ranma_set("mybar.label", label=time.strftime("%H:%M:%S"))
    time.sleep(1)
```

## Init Script

The init script is executed by `ranma-server` on startup. It should launch widget processes in the background:

```sh
#!/bin/zsh
SCRIPT_DIR="${0:A:h}"
reap --watch $PPID "$SCRIPT_DIR/plugins/my_widget.py" &
```

`reap` watches the parent process (ranma-server) and terminates the widget when the server exits. Install it with `brew install typester/reap/reap`.

Start the server with your init script:

```sh
ranma start --init ./path/to/init
```

The default init script location is `~/.config/ranma/init`.

## CLI Reference

### `ranma add <name>`

Creates a new node.

```sh
ranma add <name> [options]
```

### `ranma set <name>`

Updates properties of an existing node. Accepts the same options as `add` except `--type`.

```sh
ranma set <name> [options]
```

Pass an empty string to clear a property: `--label ""`

### `ranma remove <name>`

Removes a node. If it's a container, all children are also removed.

### `ranma query [name] [--display N]`

Queries node state. Returns JSON.

### `ranma displays`

Lists connected displays. Returns JSON with display IDs, names, and which is the main display.

## Property Reference

### Content

| Option | Type | Description |
|--------|------|-------------|
| `--type` | string | `item` (default), `row`, `column`, `box` |
| `--parent` | string | Parent container name |
| `--label` | string | Text content |
| `--icon` | string | SF Symbol name (e.g. `wifi`, `battery.100percent`) |
| `--position` | int | Sort order among siblings (default 0, lower = first) |

### Colors

All colors are hex strings: `#RRGGBB` or `#RRGGBBAA` (with alpha).

| Option | Scope | Description |
|--------|-------|-------------|
| `--background-color` | all | Fill color |
| `--label-color` | items | Text color |
| `--icon-color` | items | SF Symbol tint color |
| `--border-color` | all | Border stroke color |
| `--shadow-color` | all | Drop shadow color |

### Dimensions

| Option | Type | Description |
|--------|------|-------------|
| `--width` | float | Fixed width in points |
| `--height` | float | Fixed height in points |
| `--corner-radius` | float | Rounded corner radius |
| `--border-width` | float | Border stroke width |
| `--shadow-radius` | float | Shadow blur radius |
| `--gap` | float | Spacing between children (containers only) |

### Padding

| Option | Description |
|--------|-------------|
| `--padding` | All 4 sides |
| `--padding-horizontal` | Left + right |
| `--padding-vertical` | Top + bottom |
| `--padding-left` | Left only |
| `--padding-right` | Right only |
| `--padding-top` | Top only |
| `--padding-bottom` | Bottom only |

Specific sides override shorthands: `--padding-left` takes precedence over `--padding-horizontal` which takes precedence over `--padding`.

### Margin

Same structure as padding: `--margin`, `--margin-horizontal`, `--margin-vertical`, `--margin-left`, `--margin-right`, `--margin-top`, `--margin-bottom`.

### Typography

| Option | Type | Description |
|--------|------|-------------|
| `--font-size` | float | Font size in points (default 13) |
| `--font-weight` | string | `ultralight`, `thin`, `light`, `regular`, `medium`, `semibold`, `bold`, `heavy`, `black` |
| `--font-family` | string | Font family name (e.g. `"Hack Nerd Font"`) |

### Layout

| Option | Type | Description |
|--------|------|-------------|
| `--align-items` | string | Cross-axis alignment: `start`, `center`, `end` |
| `--justify-content` | string | Main-axis alignment: `start`, `center`, `end` |

### Interaction

| Option | Type | Description |
|--------|------|-------------|
| `--on-click` | string | Shell command executed on click (via `/bin/sh -c`) |
| `--hover-background-color` | string | Background color on mouse hover |
| `--hover-label-color` | string | Label color on hover (applied to children) |
| `--hover-icon-color` | string | Icon color on hover (applied to children) |

### Display

| Option | Type | Description |
|--------|------|-------------|
| `--display` | int | Target display ID (use `ranma displays` to list) |
| `--notch-align` | string | `left` or `right` — which side of the notch (default `right`) |

## Patterns

### Naming Convention

Use dot-separated names to organize your node tree:

```
mybar              ← top-level container
mybar.icon         ← child item
mybar.label        ← child item
mybar.sep          ← separator item
mybar.detail       ← child item
```

### Pill-Shaped Container

The most common pattern — a dark rounded container with content:

```sh
ranma add mypill --type row \
  --background-color "#0a0a0fdd" \
  --corner-radius 8 \
  --padding-horizontal 10 --padding-vertical 4 \
  --gap 4 --align-items center

ranma add mypill.icon --parent mypill \
  --icon "star.fill" --icon-color "#ffcc00" --font-size 10

ranma add mypill.label --parent mypill \
  --label "Hello" --label-color "#ffffff" --font-size 11
```

### Separator

A thin vertical divider between sections:

```sh
ranma add mybar.sep --parent mybar \
  --label "|" --label-color "#ffffff30" --font-size 12
```

### Dynamic Show/Hide

Create and remove containers based on state:

```python
visible = False

while True:
    data = get_data()

    if data and not visible:
        ranma_add("widget", type="row", ...)
        ranma_add("widget.label", parent="widget", ...)
        visible = True
    elif not data and visible:
        ranma_remove("widget")  # removes children too
        visible = False
    elif visible:
        ranma_set("widget.label", label=data)

    time.sleep(3)
```

### Dynamic Island Style

Attach the bar to the top edge of the screen by using a column with top padding that pushes the content above the screen edge:

```sh
# Outer column — no background, acts as positioning wrapper
ranma add island --type column --corner-radius 14

# Content row — the visible bar
ranma add island.bar --type row --parent island \
  --background-color "#0a0a0f" --corner-radius 14 \
  --padding-top 14 --padding-bottom 2 --padding-horizontal 10 \
  --gap 6 --align-items center

# Spacer — pushes the column down so island.bar overlaps the top edge
ranma add island.spacer --parent island \
  --label " " --label-color "#00000000" --font-size 1 \
  --height 27 --width 40 --position 2

# Add items inside island.bar
ranma add island.bar.label --parent island.bar \
  --label "Hello" --label-color "#ffffff"
```

### Box Layout (Layered UI)

Use `box` to stack elements for complex indicators:

```sh
# Fixed-size box container
ranma add tag --type box --width 22 --height 18 --on-click "echo clicked"

# Layer 1: background (hidden by default, shown when active)
ranma add tag.bg --type row --parent tag \
  --corner-radius 5 --width 22 --height 18

# Layer 2: centered label
ranma add tag.center --type row --parent tag \
  --justify-content center --align-items center \
  --width 22 --height 18 --position 2
ranma add tag.center.label --parent tag.center \
  --label "1" --label-color "#888888" --font-size 10

# Layer 3: bottom-aligned dot indicator
ranma add tag.dot --type row --parent tag \
  --justify-content center --align-items end \
  --width 22 --height 18 --position 3
ranma add tag.dot.ind --type row --parent tag.dot \
  --corner-radius 1 --width 4 --height 2 \
  --background-color "#666666"
```

To activate the tag, update colors:

```sh
ranma set tag.bg --background-color "#ffffff"
ranma set tag.center.label --label-color "#000000"
```

## Multi-Monitor

Use `ranma displays` to discover connected displays:

```sh
$ ranma displays
{"status":"display_list","displays":[
  {"id":1,"name":"Built-in Display","is_main":false},
  {"id":2,"name":"DELL U2723QE","is_main":true}
]}
```

Target a specific display with `--display`:

```sh
ranma add mybar --type row --display 2 ...
```

Child nodes inherit the parent's display. On notched displays (MacBook), use `--notch-align left` or `--notch-align right` (default) to choose which side of the notch.
