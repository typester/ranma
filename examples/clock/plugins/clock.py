#!/usr/bin/env python3

"""Clock plugin for ranma status bar.

Displays time (HH:MM) and date (e.g. "Sun 16") in a dark pill.
Updates every second.
"""

import subprocess
import time
from datetime import datetime

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


def setup():
    ranma_add(
        "clock",
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
        gap="6",
        notch_align="right",
    )
    ranma_add(
        "clock.time",
        parent="clock",
        label="--:--",
        label_color="#ffffff",
        font_size="11",
        font_weight="medium",
        position="1",
    )
    ranma_add(
        "clock.date",
        parent="clock",
        label="---",
        label_color="#ffffff80",
        font_size="9",
        position="2",
    )


def update():
    now = datetime.now()
    ranma_set("clock.time", label=now.strftime("%H:%M"))
    ranma_set("clock.date", label=now.strftime("%a %d"))


def run():
    setup()
    while True:
        update()
        time.sleep(1)


if __name__ == "__main__":
    run()
