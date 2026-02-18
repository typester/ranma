#!/usr/bin/env python3

"""Pixel art runner animation plugin for ranma status bar.

Generates 4 frames of a 7x8 pixel-art running character as PNGs,
then cycles through them at ~150ms to create a looping animation.
Pure stdlib — no Pillow required.
"""

import os
import struct
import subprocess
import time
import zlib

TMP_DIR = "/tmp/ranma_pixel_runner"
FRAME_INTERVAL = 0.15

# Palette
_ = (0, 0, 0, 0)            # transparent
H = (0xFF, 0xC8, 0x96, 0xFF) # skin / head
B = (0x42, 0x87, 0xF5, 0xFF) # body (blue shirt)
P = (0x30, 0x50, 0x90, 0xFF) # pants (dark blue)
S = (0xE0, 0x50, 0x50, 0xFF) # shoes (red)

# 7x8 frames — compact running character
FRAMES = [
    # Frame 0: right leg forward, left arm forward
    [
        [_, _, H, H, H, _, _],
        [_, _, H, H, H, _, _],
        [_, B, B, B, B, _, _],
        [_, _, B, B, B, B, _],
        [_, _, P, P, P, _, _],
        [_, P, _, _, _, P, _],
        [P, _, _, _, _, _, P],
        [S, _, _, _, _, _, S],
    ],
    # Frame 1: mid-stride (legs closer)
    [
        [_, _, H, H, H, _, _],
        [_, _, H, H, H, _, _],
        [_, _, B, B, B, _, _],
        [_, B, B, B, _, B, _],
        [_, _, P, P, P, _, _],
        [_, _, P, _, P, _, _],
        [_, P, _, _, _, P, _],
        [_, S, _, _, _, S, _],
    ],
    # Frame 2: left leg forward, right arm forward
    [
        [_, _, H, H, H, _, _],
        [_, _, H, H, H, _, _],
        [_, _, B, B, B, B, _],
        [_, B, B, B, B, _, _],
        [_, _, P, P, P, _, _],
        [_, P, _, _, _, P, _],
        [P, _, _, _, _, _, P],
        [S, _, _, _, _, _, S],
    ],
    # Frame 3: mid-stride (legs closer, mirrored arms)
    [
        [_, _, H, H, H, _, _],
        [_, _, H, H, H, _, _],
        [_, _, B, B, B, _, _],
        [_, _, B, B, B, B, _],
        [_, _, P, P, P, _, _],
        [_, _, P, _, P, _, _],
        [_, P, _, _, _, P, _],
        [_, S, _, _, _, S, _],
    ],
]


def make_png(width, height, pixels):
    """Create a minimal PNG from RGBA pixel data."""
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


def generate_frames():
    os.makedirs(TMP_DIR, exist_ok=True)
    paths = []
    for i, frame in enumerate(FRAMES):
        path = os.path.join(TMP_DIR, f"frame{i}.png")
        with open(path, "wb") as f:
            f.write(make_png(7, 8, frame))
        paths.append(path)
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


def setup(frame_paths):
    ranma_add(
        "runner",
        type="row",
        align_items="center",
        background_color="#0a0a0f",
        corner_radius="8",
        shadow_color="#00000080",
        shadow_radius="8",
        margin_horizontal="4",
        margin_vertical="4",
        padding_horizontal="4",
        padding_vertical="2",
        notch_align="right",
    )
    ranma_add(
        "runner.sprite",
        parent="runner",
        image=frame_paths[0],
        image_scale="3",
    )


def run():
    frame_paths = generate_frames()
    setup(frame_paths)
    frame_index = 0
    while True:
        ranma_set("runner.sprite", image=frame_paths[frame_index])
        frame_index = (frame_index + 1) % len(frame_paths)
        time.sleep(FRAME_INTERVAL)


if __name__ == "__main__":
    run()
