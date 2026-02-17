#!/usr/bin/env python3

"""CPU & Memory monitor plugin for ranma status bar.

Displays CPU usage percentage and memory pressure in a dark pill.
CPU is sampled via ps, memory via vm_stat + sysctl.
Polls every 5 seconds.
"""

import re
import subprocess
import time

POLL_INTERVAL = 5
PAGE_SIZE = 16384


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


def get_cpu():
    try:
        r = subprocess.run(
            ["ps", "-A", "-o", "%cpu"], capture_output=True, text=True
        )
        lines = r.stdout.strip().split("\n")[1:]
        return sum(float(x.strip()) for x in lines if x.strip())
    except Exception:
        return 0.0


def get_total_mem():
    try:
        r = subprocess.run(
            ["sysctl", "-n", "hw.memsize"], capture_output=True, text=True
        )
        return int(r.stdout.strip())
    except Exception:
        return 0


def get_used_mem():
    try:
        r = subprocess.run(["vm_stat"], capture_output=True, text=True)
        stats = {}
        for line in r.stdout.strip().split("\n"):
            m = re.match(r"(.+):\s+(\d+)", line)
            if m:
                stats[m.group(1).strip()] = int(m.group(2))
        active = stats.get("Pages active", 0)
        wired = stats.get("Pages wired down", 0)
        compressed = stats.get("Pages occupied by compressor", 0)
        return (active + wired + compressed) * PAGE_SIZE
    except Exception:
        return 0


def cpu_color(pct):
    if pct >= 80:
        return "#ff3b30"
    if pct >= 50:
        return "#ff9500"
    return "#34c759"


def mem_color(used, total):
    if total == 0:
        return "#ffffff"
    ratio = used / total
    if ratio >= 0.85:
        return "#ff3b30"
    if ratio >= 0.70:
        return "#ff9500"
    return "#34c759"


def format_mem(bytes_val):
    gb = bytes_val / (1024 ** 3)
    return f"{gb:.1f}G"


def setup():
    ranma_add(
        "sysmon",
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
        "sysmon.cpu",
        type="row",
        parent="sysmon",
        align_items="center",
        gap="3",
        position="1",
    )
    ranma_add(
        "sysmon.cpu.icon",
        parent="sysmon.cpu",
        icon="cpu",
        icon_color="#34c759",
        font_size="9",
    )
    ranma_add(
        "sysmon.cpu.val",
        parent="sysmon.cpu",
        label="--%",
        label_color="#ffffff",
        font_size="10",
    )
    ranma_add(
        "sysmon.sep",
        parent="sysmon",
        label="|",
        label_color="#565f8930",
        font_size="11",
        position="2",
    )
    ranma_add(
        "sysmon.mem",
        type="row",
        parent="sysmon",
        align_items="center",
        gap="3",
        position="3",
    )
    ranma_add(
        "sysmon.mem.icon",
        parent="sysmon.mem",
        icon="memorychip",
        icon_color="#34c759",
        font_size="9",
    )
    ranma_add(
        "sysmon.mem.val",
        parent="sysmon.mem",
        label="--G",
        label_color="#ffffff",
        font_size="10",
    )


def update():
    cpu = get_cpu()
    total_mem = get_total_mem()
    used_mem = get_used_mem()

    cc = cpu_color(cpu)
    mc = mem_color(used_mem, total_mem)

    ranma_set("sysmon.cpu.val", label=f"{cpu:.0f}%")
    ranma_set("sysmon.cpu.icon", icon_color=cc)
    ranma_set("sysmon.mem.val", label=format_mem(used_mem))
    ranma_set("sysmon.mem.icon", icon_color=mc)


def run():
    setup()
    while True:
        update()
        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    run()
