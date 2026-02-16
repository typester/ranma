#!/usr/bin/env python3

"""Multi-monitor event bridge between yashiki and ranma.

Subscribes to yashiki state events, tracks per-display workspace state,
and updates ranma item label colors via `ranma set` commands.

Slot assignment is managed by the bridge so that display connect/disconnect
is handled correctly without requiring a ranma restart.

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

# Colors for focused display (matching sketchybar reference)
FOCUSED_ACTIVE_LABEL = "#ffffff"
FOCUSED_ACTIVE_BG = "#ffffff40"
FOCUSED_OCCUPIED_LABEL = "#ffffff"
FOCUSED_VACANT_LABEL = "#888888"

# Colors for unfocused display
UNFOCUSED_ACTIVE_LABEL = "#ffffff80"
UNFOCUSED_ACTIVE_BG = "#ffffff20"
UNFOCUSED_OCCUPIED_LABEL = "#ffffff80"
UNFOCUSED_VACANT_LABEL = "#88888840"


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
    """Assign display slots (1-MAX_DISPLAYS) to yashiki displays.

    Maintains existing assignments. New displays get the lowest free slot.
    Removed displays free their slot.
    """
    current_displays = set(state["displays"].keys())
    assignment = state["slot_assignment"]

    # Free slots of removed displays
    for slot, did in list(assignment.items()):
        if did not in current_displays:
            del assignment[slot]

    # Assign new displays to free slots
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


def ensure_display_items(state):
    """Ensure ranma containers and items exist for all active slots.

    Creates containers/items for newly assigned slots. Removes items
    for slots that are no longer assigned.
    """
    active_slots = set(state["slot_assignment"].keys())
    existing_slots = state.get("created_slots", set())

    # Create items for new slots
    for slot in active_slots - existing_slots:
        did = state["slot_assignment"][slot]
        ranma_add(
            f"ws.d{slot}",
            type="container",
            background_color="#00000080",
            corner_radius="8",
            shadow_color="#000000ff",
            shadow_radius="10",
            padding_left="6",
            padding_right="6",
            gap="3",
            height="22",
            notch_align="right",
            display=did,
        )
        for tag_num in range(1, NUM_TAGS + 1):
            ranma_add(
                f"space.d{slot}.{tag_num}",
                parent=f"ws.d{slot}",
                label=str(tag_num),
                label_color=FOCUSED_VACANT_LABEL,
                font_family="Hack Nerd Font",
                font_weight="bold",
                font_size="12",
                padding_left="2",
                padding_right="2",
                corner_radius="3",
                height="18",
                position=str(tag_num),
            )

    # Remove items for removed slots
    for slot in existing_slots - active_slots:
        for tag_num in range(1, NUM_TAGS + 1):
            ranma_remove(f"space.d{slot}.{tag_num}")
        ranma_remove(f"ws.d{slot}")

    state["created_slots"] = set(active_slots)


def update_all(state):
    """Re-render all items based on current state."""
    # Per-display occupied tags (keyed by display_id string)
    occupied_per_display = {}
    for winfo in state["windows"].values():
        did = winfo["output_id"]
        occupied_per_display[did] = occupied_per_display.get(did, 0) | winfo["tags"]

    active_slots = set(state["slot_assignment"].keys())

    for slot in range(1, MAX_DISPLAYS + 1):
        if slot not in active_slots:
            continue

        did = state["slot_assignment"][slot]
        display_info = state["displays"].get(did, {})
        visible_tags = display_info.get("visible_tags", 0)
        display_occupied = occupied_per_display.get(did, 0)
        focused = did == state["focused_display"]

        for tag_num in range(1, NUM_TAGS + 1):
            bitmask = 1 << (tag_num - 1)
            item = f"space.d{slot}.{tag_num}"
            is_active = (visible_tags & bitmask) != 0
            is_occupied = (display_occupied & bitmask) != 0

            if is_active:
                label_color = FOCUSED_ACTIVE_LABEL if focused else UNFOCUSED_ACTIVE_LABEL
                bg_color = FOCUSED_ACTIVE_BG if focused else UNFOCUSED_ACTIVE_BG
                ranma_set(item, label_color=label_color, background_color=bg_color)
            elif is_occupied:
                label_color = FOCUSED_OCCUPIED_LABEL if focused else UNFOCUSED_OCCUPIED_LABEL
                ranma_set(item, label_color=label_color, background_color="")
            else:
                label_color = FOCUSED_VACANT_LABEL if focused else UNFOCUSED_VACANT_LABEL
                ranma_set(item, label_color=label_color, background_color="")


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

    while True:
        try:
            proc = subprocess.Popen(
                [
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
