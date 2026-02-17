#!/usr/bin/env python3

"""Wi-Fi status plugin for ranma status bar.

Displays Wi-Fi connection status and signal strength.
Uses CoreWLAN via a Swift script for RSSI and power state.
Polls every 10 seconds.

Note: SSID requires Location Services permission which is not available
from CLI tools on modern macOS, so only signal strength is shown.
"""

import os
import subprocess
import tempfile
import time

POLL_INTERVAL = 10

SWIFT_SRC = """\
import Foundation
import CoreWLAN

let client = CWWiFiClient.shared()
if let iface = client.interface() {
    let power = iface.powerOn()
    let rssi = iface.rssiValue()
    let ssid = iface.ssid() ?? ""
    print("power:\\(power)")
    print("rssi:\\(rssi)")
    print("ssid:\\(ssid)")
} else {
    print("power:false")
    print("rssi:0")
    print("ssid:")
}
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


def ensure_script():
    cache_dir = os.path.join(tempfile.gettempdir(), "ranma-helpers")
    os.makedirs(cache_dir, exist_ok=True)
    src = os.path.join(cache_dir, "wifi.swift")
    if not os.path.exists(src):
        with open(src, "w") as f:
            f.write(SWIFT_SRC)
    return src


def get_wifi_info(script):
    try:
        r = subprocess.run(
            ["swift", script], capture_output=True, text=True, timeout=10
        )
        info = {"power": False, "rssi": 0, "ssid": ""}
        for line in r.stdout.strip().split("\n"):
            if line.startswith("power:"):
                info["power"] = line.split(":")[1] == "true"
            elif line.startswith("rssi:"):
                info["rssi"] = int(line.split(":")[1])
            elif line.startswith("ssid:"):
                info["ssid"] = line[5:]
        return info
    except Exception:
        return {"power": False, "rssi": 0, "ssid": ""}


def wifi_icon(rssi):
    if rssi >= -50:
        return "wifi"
    if rssi >= -70:
        return "wifi"
    return "wifi.exclamationmark"


def signal_color(rssi):
    if rssi >= -50:
        return "#34c759"
    if rssi >= -70:
        return "#ffffff"
    return "#ff9500"


def signal_label(rssi):
    if rssi >= -50:
        return "Strong"
    if rssi >= -70:
        return "Fair"
    return "Weak"


def setup():
    ranma_add(
        "wifi",
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
        "wifi.icon",
        parent="wifi",
        icon="wifi",
        icon_color="#ffffff",
        font_size="10",
        position="1",
    )
    ranma_add(
        "wifi.label",
        parent="wifi",
        label="--",
        label_color="#ffffff",
        font_size="10",
        position="2",
    )


def update(script):
    info = get_wifi_info(script)

    if not info["power"]:
        ranma_set("wifi.icon", icon="wifi.slash", icon_color="#ff3b30")
        ranma_set("wifi.label", label="Off")
    elif info["rssi"] == 0:
        ranma_set("wifi.icon", icon="wifi.slash", icon_color="#ffffff60")
        ranma_set("wifi.label", label="No Signal")
    else:
        rssi = info["rssi"]
        label = info["ssid"] if info["ssid"] else signal_label(rssi)
        ranma_set("wifi.icon", icon=wifi_icon(rssi), icon_color=signal_color(rssi))
        ranma_set("wifi.label", label=label)


def run():
    script = ensure_script()
    setup()
    while True:
        update(script)
        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    run()
