#!/usr/bin/env python3

"""Battery plugin for ranma status bar.

Displays battery icon (SF Symbol) and percentage in a dark pill.
Icon changes based on charge level and charging state.
Polls every 30 seconds via `pmset -g batt`.
"""

import re
import subprocess
import time

POLL_INTERVAL = 30


def ranma_add(name, **kwargs):
    args = ["ranma", "add", name]
    for key, value in kwargs.items():
        flag = "--" + key.replace("_", "-")
        args.extend([flag, str(value)])
    subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def ranma_set(name, **kwargs):
    args = ["ranma", "set", name]
    for key, value in kwargs.items():
        flag = "--" + key.replace("_", "-")
        args.extend([flag, str(value)])
    subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def get_battery_info():
    """Parse pmset output. Returns (percent, charging, on_ac)."""
    try:
        result = subprocess.run(
            ["pmset", "-g", "batt"], capture_output=True, text=True
        )
        output = result.stdout
        on_ac = "AC Power" in output
        match = re.search(r"(\d+)%;\s*(charging|charged|discharging|finishing charge)", output)
        if match:
            pct = int(match.group(1))
            state = match.group(2)
            charging = state in ("charging", "finishing charge")
            return pct, charging, on_ac
    except Exception:
        pass
    return None, False, False


def battery_icon(pct, charging):
    if charging:
        return "battery.100percent.bolt"
    if pct is None:
        return "battery.0percent"
    if pct >= 75:
        return "battery.100percent"
    if pct >= 50:
        return "battery.75percent"
    if pct >= 25:
        return "battery.50percent"
    if pct >= 10:
        return "battery.25percent"
    return "battery.0percent"


def icon_color(pct, charging):
    if charging:
        return "#34c759"
    if pct is not None and pct < 10:
        return "#ff3b30"
    if pct is not None and pct < 25:
        return "#ff9500"
    return "#ffffff"


def setup():
    ranma_add(
        "bat",
        type="row",
        align_items="center",
        background_color="#0a0a0f",
        corner_radius="8",
        shadow_color="#00000080",
        shadow_radius="8",
        margin_horizontal="4",
        margin_vertical="4",
        padding_horizontal="6",
        padding_vertical="2",
        gap="4",
        notch_align="right",
    )
    ranma_add(
        "bat.icon",
        parent="bat",
        icon="battery.100percent",
        icon_color="#ffffff",
        font_size="12",
        position="1",
    )
    ranma_add(
        "bat.pct",
        parent="bat",
        label="--%",
        label_color="#ffffff",
        font_size="10",
        position="2",
    )


def update():
    pct, charging, on_ac = get_battery_info()
    icon = battery_icon(pct, charging)
    color = icon_color(pct, charging)
    label = f"{pct}%" if pct is not None else "--%"

    ranma_set("bat.icon", icon=icon, icon_color=color)
    ranma_set("bat.pct", label=label)


def run():
    setup()
    while True:
        update()
        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    run()
