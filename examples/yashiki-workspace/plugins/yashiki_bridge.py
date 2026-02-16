#!/usr/bin/env python3

"""Multi-monitor event bridge between yashiki and ranma.

Subscribes to yashiki state events, tracks per-display workspace state,
and updates ranma workspace indicators via `ranma set` commands.

Each workspace tag is rendered as a Box containing layered Rows:
  - bg:  pill background (visible when active)
  - c:   centered label
  - ul:  underline/dot indicator (bottom-aligned)

Runs as a long-lived background process, started from the init script.
"""

import json
import os
import signal
import subprocess
import sys
import time

MAX_DISPLAYS = 3
NUM_TAGS = 10
INTERNAL_DISPLAY_ID = "1"

_BASE = {
    "box_w": "22", "box_h": "18",
    "font_size": "10",
    "corner_radius": "8",
    "padding_h": "4", "padding_v": "2",
    "gap": "2",
    "pill_radius": "5",
    "ul_bar_w": "8", "ul_bar_h": "2",
    "dot_w": "3", "dot_h": "3",
    "margin_h": "12",
    "show_mode": True,
    "mode_h": "14", "mode_pad_h": "4", "mode_icon_size": "8",
    "sep_font_size": "11",
}

SIZES = {
    "default": {**_BASE},
    "compact": {**_BASE, "padding_h": "2", "gap": "1", "margin_h": "4", "show_mode": False},
}


def parse_hex(color):
    """Parse #RRGGBB or #RRGGBBAA to (r, g, b) tuple (0-255)."""
    c = color.lstrip("#")
    return (int(c[0:2], 16), int(c[2:4], 16), int(c[4:6], 16))


def hex_rgb(r, g, b):
    """Format (r, g, b) as #RRGGBB."""
    return f"#{int(r):02x}{int(g):02x}{int(b):02x}"


def hex_rgba(r, g, b, a):
    """Format (r, g, b, a) as #RRGGBBAA. a is 0-255."""
    return f"#{int(r):02x}{int(g):02x}{int(b):02x}{int(a):02x}"


def lerp(a, b, t):
    """Linear interpolation."""
    return a + (b - a) * t


def build_palette(accent_hex):
    """Derive full color palette from accent color.

    Blends accent hue into greys so the palette feels cohesive.
    accent="#ffffff" → pure greys, accent="#fac800" → warm greys, etc.
    """
    ar, ag, ab = parse_hex(accent_hex)

    # Normalize accent to unit brightness for tinting
    max_c = max(ar, ag, ab, 1)
    tr, tg, tb = ar / max_c, ag / max_c, ab / max_c

    def tinted_grey(brightness, tint_strength=0.15):
        """Mix a grey value with the accent tint."""
        r = lerp(brightness, brightness * tr, tint_strength)
        g = lerp(brightness, brightness * tg, tint_strength)
        b = lerp(brightness, brightness * tb, tint_strength)
        return (r, g, b)

    occ_r, occ_g, occ_b = tinted_grey(180)
    vac_r, vac_g, vac_b = tinted_grey(100)
    dot_r, dot_g, dot_b = tinted_grey(180)

    return {
        # Focused display
        "label_active": "#ffffff",
        "label_occupied": hex_rgb(occ_r, occ_g, occ_b),
        "label_vacant": hex_rgb(vac_r, vac_g, vac_b),
        "bg_active_pill": "#ffffff18",
        "dot_color": hex_rgba(dot_r, dot_g, dot_b, 128),
        # Unfocused display (half alpha)
        "label_active_unfocused": "#ffffff80",
        "label_occupied_unfocused": hex_rgba(occ_r, occ_g, occ_b, 128),
        "label_vacant_unfocused": hex_rgba(vac_r, vac_g, vac_b, 128),
        "bg_active_pill_unfocused": "#ffffff10",
        "dot_color_unfocused": hex_rgba(dot_r, dot_g, dot_b, 64),
    }


def load_config():
    """Load config from config.json next to the init script."""
    config_path = os.path.join(os.path.dirname(__file__), "..", "config.json")
    defaults = {"accent_color": "#ffffffcc", "font_family": "Hack Nerd Font"}
    try:
        with open(config_path) as f:
            user = json.load(f)
            defaults.update(user)
    except (FileNotFoundError, json.JSONDecodeError):
        pass
    return defaults


def kill_old_instances():
    """Kill any existing yashiki_bridge.py processes (except ourselves)."""
    my_pid = os.getpid()
    try:
        result = subprocess.run(
            ["pgrep", "-f", "yashiki_bridge.py"],
            capture_output=True,
            text=True,
        )
        for line in result.stdout.strip().split("\n"):
            if not line:
                continue
            pid = int(line)
            if pid != my_pid:
                try:
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
    except Exception:
        pass


def assign_slots(state):
    """Assign display slots (1-MAX_DISPLAYS) to yashiki displays."""
    current_displays = set(state["displays"].keys())
    assignment = state["slot_assignment"]

    for slot, did in list(assignment.items()):
        if did not in current_displays:
            del assignment[slot]

    assigned = set(assignment.values())
    free_slots = [s for s in range(1, MAX_DISPLAYS + 1) if s not in assignment]
    for did in sorted(current_displays - assigned):
        if not free_slots:
            break
        assignment[free_slots.pop(0)] = did


def ranma_set(name, **kwargs):
    """Run `ranma set <name> --key value ...`."""
    args = ["ranma", "set", name]
    for key, value in kwargs.items():
        flag = "--" + key.replace("_", "-")
        args.extend([flag, str(value)])
    subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def ranma_add(name, **kwargs):
    """Run `ranma add <name> --key value ...`."""
    args = ["ranma", "add", name]
    for key, value in kwargs.items():
        flag = "--" + key.replace("_", "-")
        args.extend([flag, str(value)])
    subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def ranma_remove(name):
    """Run `ranma remove <name>`."""
    subprocess.run(
        ["ranma", "remove", name],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def sizes_for_display(did):
    """Return size parameters based on display ID."""
    return SIZES["compact"] if did == INTERNAL_DISPLAY_ID else SIZES["default"]


def ensure_display_items(state):
    """Ensure ranma containers and items exist for all active slots."""
    config = state["config"]
    active_slots = set(state["slot_assignment"].keys())
    existing_slots = state.get("created_slots", set())

    for slot in active_slots - existing_slots:
        did = state["slot_assignment"][slot]
        sz = sizes_for_display(did)
        state["slot_sizes"][slot] = sz
        ranma_add(
            f"ws.d{slot}",
            type="row",
            align_items="center",
            background_color="#0a0a0f",
            corner_radius=sz["corner_radius"],
            shadow_color="#00000080",
            shadow_radius="8",
            margin_horizontal=sz["margin_h"],
            margin_vertical="4",
            padding_horizontal=sz["padding_h"],
            padding_vertical=sz["padding_v"],
            gap=sz["gap"],
            notch_align="right",
            display=did,
        )
        for tag_num in range(1, NUM_TAGS + 1):
            base = f"ws.d{slot}.{tag_num}"
            bitmask = 1 << (tag_num - 1)
            ranma_add(
                base,
                type="box",
                parent=f"ws.d{slot}",
                width=sz["box_w"],
                height=sz["box_h"],
                position=str(tag_num),
                on_click=f"yashiki tag-view {bitmask} --output {did}",
            )
            ranma_add(
                f"{base}.bg",
                type="row",
                parent=base,
                corner_radius=sz["pill_radius"],
                width=sz["box_w"],
                height=sz["box_h"],
            )
            ranma_add(
                f"{base}.c",
                type="row",
                parent=base,
                justify_content="center",
                align_items="center",
                width=sz["box_w"],
                height=sz["box_h"],
                position="2",
            )
            ranma_add(
                f"{base}.c.label",
                parent=f"{base}.c",
                label=str(tag_num),
                label_color=state["palette"]["label_vacant"],
                font_family=config["font_family"],
                font_size=sz["font_size"],
            )
            ranma_add(
                f"{base}.ul",
                type="row",
                parent=base,
                justify_content="center",
                align_items="end",
                width=sz["box_w"],
                height=sz["box_h"],
                position="3",
            )
            ranma_add(
                f"{base}.ul.ind",
                type="row",
                parent=f"{base}.ul",
                corner_radius="1",
            )
        if sz["show_mode"]:
            ranma_add(
                f"ws.d{slot}.sep",
                parent=f"ws.d{slot}",
                label="|",
                label_color="#565f8930",
                font_size=sz["sep_font_size"],
                position="11",
            )
            ranma_add(
                f"ws.d{slot}.mode",
                type="row",
                parent=f"ws.d{slot}",
                background_color="#ffffff18",
                corner_radius="4",
                padding_horizontal=sz["mode_pad_h"],
                align_items="center",
                height=sz["mode_h"],
                position="12",
            )
            ranma_add(
                f"ws.d{slot}.mode.icon",
                parent=f"ws.d{slot}.mode",
                icon="rectangle.split.3x1",
                icon_color="#ffffff",
                font_size=sz["mode_icon_size"],
            )

    for slot in existing_slots - active_slots:
        ranma_remove(f"ws.d{slot}")
        state["slot_sizes"].pop(slot, None)

    state["created_slots"] = set(active_slots)


def update_all(state):
    """Re-render all items based on current state."""
    config = state["config"]
    accent = config["accent_color"]
    p = state["palette"]

    occupied_per_display = {}
    for winfo in state["windows"].values():
        did = winfo["output_id"]
        occupied_per_display[did] = occupied_per_display.get(did, 0) | winfo["tags"]

    active_slots = set(state["slot_assignment"].keys())

    for slot in range(1, MAX_DISPLAYS + 1):
        if slot not in active_slots:
            continue

        did = state["slot_assignment"][slot]
        sz = state["slot_sizes"].get(slot, SIZES["default"])
        display_info = state["displays"].get(did, {})
        visible_tags = display_info.get("visible_tags", 0)
        display_occupied = occupied_per_display.get(did, 0)
        focused = did == state["focused_display"]

        for tag_num in range(1, NUM_TAGS + 1):
            bitmask = 1 << (tag_num - 1)
            base = f"ws.d{slot}.{tag_num}"
            is_active = (visible_tags & bitmask) != 0
            is_occupied = (display_occupied & bitmask) != 0

            if is_active:
                ranma_set(
                    f"{base}.c.label",
                    label_color=p["label_active"] if focused else p["label_active_unfocused"],
                )
                ranma_set(
                    f"{base}.bg",
                    background_color=p["bg_active_pill"] if focused else p["bg_active_pill_unfocused"],
                )
                ranma_set(
                    f"{base}.ul.ind",
                    background_color=accent,
                    width=sz["ul_bar_w"],
                    height=sz["ul_bar_h"],
                    corner_radius="1",
                )
            elif is_occupied:
                ranma_set(
                    f"{base}.c.label",
                    label_color=p["label_occupied"] if focused else p["label_occupied_unfocused"],
                )
                ranma_set(f"{base}.bg", background_color="")
                ranma_set(
                    f"{base}.ul.ind",
                    background_color=p["dot_color"] if focused else p["dot_color_unfocused"],
                    width=sz["dot_w"],
                    height=sz["dot_h"],
                    corner_radius="2",
                )
            else:
                ranma_set(
                    f"{base}.c.label",
                    label_color=p["label_vacant"] if focused else p["label_vacant_unfocused"],
                )
                ranma_set(f"{base}.bg", background_color="")
                ranma_set(f"{base}.ul.ind", background_color="")


def process_snapshot(event, state):
    """Initialize state from snapshot event."""
    state["displays"] = {
        str(d["id"]): {"visible_tags": d["visible_tags"]}
        for d in event["displays"]
    }
    state["windows"] = {
        str(w["id"]): {"tags": w["tags"], "output_id": str(w["output_id"])}
        for w in event["windows"]
    }
    state["focused_display"] = str(event["focused_display_id"])


def process_event(event, state):
    """Process an incremental event. Returns True if state changed."""
    t = event["type"]
    if t == "tags_changed":
        did = str(event["display_id"])
        if did in state["displays"]:
            state["displays"][did]["visible_tags"] = event["visible_tags"]
    elif t == "display_focused":
        state["focused_display"] = str(event["display_id"])
    elif t in ("window_created", "window_updated"):
        w = event["window"]
        state["windows"][str(w["id"])] = {
            "tags": w["tags"],
            "output_id": str(w["output_id"]),
        }
    elif t == "window_destroyed":
        state["windows"].pop(str(event["window_id"]), None)
    elif t in ("display_added", "display_updated"):
        d = event["display"]
        state["displays"][str(d["id"])] = {"visible_tags": d["visible_tags"]}
        if d.get("is_focused"):
            state["focused_display"] = str(d["id"])
        assign_slots(state)
        ensure_display_items(state)
    elif t == "display_removed":
        removed_id = str(event["display_id"])
        state["displays"].pop(removed_id, None)
        if state["focused_display"] == removed_id and state["displays"]:
            state["focused_display"] = next(iter(state["displays"]))
        assign_slots(state)
        ensure_display_items(state)
    else:
        return False
    return True


def run():
    kill_old_instances()
    config = load_config()

    while True:
        try:
            proc = subprocess.Popen(
                [
                    "reap",
                    "yashiki",
                    "subscribe",
                    "--snapshot",
                    "--filter",
                    "tags,focus,window,display",
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            state = {
                "displays": {},
                "windows": {},
                "focused_display": "0",
                "slot_assignment": {},
                "created_slots": set(),
                "slot_sizes": {},
                "config": config,
                "palette": build_palette(config["accent_color"]),
            }

            for line in proc.stdout:
                line = line.strip()
                if not line:
                    continue
                try:
                    event = json.loads(line)
                except json.JSONDecodeError:
                    continue

                if event["type"] == "snapshot":
                    process_snapshot(event, state)
                    assign_slots(state)
                    ensure_display_items(state)
                else:
                    if not process_event(event, state):
                        continue

                update_all(state)

        except FileNotFoundError:
            print("yashiki_bridge: yashiki not found in PATH", file=sys.stderr)
        except Exception as e:
            print(f"yashiki_bridge: {e}", file=sys.stderr)

        time.sleep(2)


if __name__ == "__main__":
    run()
