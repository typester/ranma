#!/usr/bin/env python3

"""Mic & Camera privacy indicator plugin for ranma status bar.

Shows a colored indicator when microphone or camera is actively in use
by any application. Uses AVFoundation via a compiled Swift helper.
Polls every 2 seconds. Hides when nothing is active.
"""

import os
import subprocess
import tempfile
import time

POLL_INTERVAL = 2

SWIFT_SRC = r"""
import Foundation
import AVFoundation

let cameras = AVCaptureDevice.DiscoverySession(
    deviceTypes: [.builtInWideAngleCamera, .external],
    mediaType: .video,
    position: .unspecified
).devices

let mics = AVCaptureDevice.DiscoverySession(
    deviceTypes: [.microphone, .external],
    mediaType: .audio,
    position: .unspecified
).devices

let camActive = cameras.contains { $0.isInUseByAnotherApplication }
let micActive = mics.contains { $0.isInUseByAnotherApplication }

print("camera:\(camActive)")
print("mic:\(micActive)")
"""


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


def compile_helper():
    cache_dir = os.path.join(tempfile.gettempdir(), "ranma-helpers")
    os.makedirs(cache_dir, exist_ok=True)
    binary = os.path.join(cache_dir, "mic_camera")
    src = os.path.join(cache_dir, "mic_camera.swift")
    if not os.path.exists(binary):
        with open(src, "w") as f:
            f.write(SWIFT_SRC)
        subprocess.run(
            ["swiftc", "-O", "-o", binary, src],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    return binary


def check_devices(binary):
    try:
        r = subprocess.run(
            [binary], capture_output=True, text=True, timeout=5
        )
        cam = False
        mic = False
        for line in r.stdout.strip().split("\n"):
            if line.startswith("camera:"):
                cam = line.split(":")[1] == "true"
            elif line.startswith("mic:"):
                mic = line.split(":")[1] == "true"
        return cam, mic
    except Exception:
        return False, False


def ranma_remove(name):
    subprocess.run(
        ["ranma", "remove", name],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def create_container():
    ranma_add(
        "privacy",
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
        gap="3",
        notch_align="right",
    )


def update(binary, prev_state):
    cam, mic = check_devices(binary)
    state = (cam, mic)

    if state == prev_state:
        return prev_state

    prev_active = prev_state[0] or prev_state[1]
    active = cam or mic

    if active and not prev_active:
        create_container()
    elif not active and prev_active:
        ranma_remove("privacy")
        return state

    if cam:
        ranma_add("privacy.cam", parent="privacy", icon="camera.fill", icon_color="#34c759", font_size="10", position="1")
    else:
        ranma_remove("privacy.cam")

    if mic:
        ranma_add("privacy.mic", parent="privacy", icon="mic.fill", icon_color="#ff9500", font_size="10", position="2")
    else:
        ranma_remove("privacy.mic")

    return state


def run():
    binary = compile_helper()
    prev = (False, False)

    while True:
        prev = update(binary, prev)
        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    run()
