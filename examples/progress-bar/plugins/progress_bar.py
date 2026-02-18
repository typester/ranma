#!/usr/bin/env python3

"""CPU progress bar plugin for ranma status bar.

Demonstrates image tiling: a 1px-wide solid-color PNG is tiled across a
variable width inside a fixed-width track, creating a progress bar.
Updates every 2 seconds.
"""

import os
import struct
import subprocess
import time
import zlib

POLL_INTERVAL = 2
TMP_DIR = "/tmp/ranma_progress_bar"
TRACK_WIDTH = 80


def make_png(width, height, pixels):
    """Create a minimal PNG from RGBA pixel data.

    pixels: list of rows, each row is a list of (r, g, b, a) tuples.
    """
    def chunk(ctype, data):
        c = ctype + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack(">I", len(data)) + c + crc

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))

    raw = b""
    for row in pixels:
        raw += b"\x00"
        for r, g, b, a in row:
            raw += struct.pack("BBBB", r, g, b, a)

    idat = chunk(b"IDAT", zlib.compress(raw))
    iend = chunk(b"IEND", b"")
    return sig + ihdr + idat + iend


def generate_bar_images():
    os.makedirs(TMP_DIR, exist_ok=True)
    paths = {}
    colors = {
        "green": (0x34, 0xC7, 0x59, 0xFF),
        "orange": (0xFF, 0x95, 0x00, 0xFF),
        "red": (0xFF, 0x3B, 0x30, 0xFF),
    }
    for name, color in colors.items():
        pixels = [[(color)] for _ in range(12)]
        path = os.path.join(TMP_DIR, f"bar_{name}.png")
        with open(path, "wb") as f:
            f.write(make_png(1, 12, pixels))
        paths[name] = path
    return paths


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
        total = sum(float(x.strip()) for x in lines if x.strip())
        ncpu = os.cpu_count() or 1
        return min(total / ncpu, 100.0)
    except Exception:
        return 0.0


def cpu_level(pct):
    if pct >= 80:
        return "red"
    if pct >= 50:
        return "orange"
    return "green"


ICON_COLORS = {
    "green": "#34c759",
    "orange": "#ff9500",
    "red": "#ff3b30",
}


def setup(bar_images):
    ranma_add(
        "cpubar",
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
        "cpubar.icon",
        parent="cpubar",
        icon="cpu",
        icon_color="#34c759",
        font_size="9",
        position="1",
    )
    ranma_add(
        "cpubar.track",
        type="box",
        parent="cpubar",
        width=str(TRACK_WIDTH),
        height="12",
        background_color="#ffffff15",
        corner_radius="3",
        position="2",
    )
    ranma_add(
        "cpubar.fill",
        parent="cpubar.track",
        image=bar_images["green"],
        image_scale="1",
        width="1",
        height="12",
    )
    ranma_add(
        "cpubar.label",
        parent="cpubar",
        label="0%",
        label_color="#ffffff",
        font_size="10",
        font_weight="medium",
        position="3",
    )


def update(bar_images, prev_level):
    cpu = get_cpu()
    bar_width = max(1, int(cpu / 100.0 * TRACK_WIDTH))
    level = cpu_level(cpu)

    props = {"width": str(bar_width)}
    if level != prev_level:
        props["image"] = bar_images[level]
    ranma_set("cpubar.fill", **props)

    if level != prev_level:
        ranma_set("cpubar.icon", icon_color=ICON_COLORS[level])

    ranma_set("cpubar.label", label=f"{cpu:.0f}%")
    return level


def run():
    bar_images = generate_bar_images()
    setup(bar_images)
    level = "green"
    while True:
        level = update(bar_images, level)
        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    run()
