#!/usr/bin/env python3

"""Transparent status bar plugin for ranma.

Transparent background that blends with the macOS menu bar.
Designed for macOS Tahoe's transparent menu bar style.
"""

import os
import re
import subprocess
import tempfile
import time
from datetime import datetime

# ---------------------------------------------------------------------------
# Swift helper sources
# ---------------------------------------------------------------------------

NOWPLAYING_SWIFT = """\
import Foundation

typealias MRMediaRemoteGetNowPlayingInfoFunction =
    @convention(c) (DispatchQueue, @escaping ([String: Any]) -> Void) -> Void

let bundle = CFBundleCreate(
    kCFAllocatorDefault,
    NSURL(fileURLWithPath: "/System/Library/PrivateFrameworks/MediaRemote.framework")
)!
let ptr = CFBundleGetFunctionPointerForName(
    bundle, "MRMediaRemoteGetNowPlayingInfo" as CFString
)!
let fn = unsafeBitCast(ptr, to: MRMediaRemoteGetNowPlayingInfoFunction.self)

let sem = DispatchSemaphore(value: 0)
fn(DispatchQueue.main) { info in
    let title = info["kMRMediaRemoteNowPlayingInfoTitle"] as? String ?? ""
    let artist = info["kMRMediaRemoteNowPlayingInfoArtist"] as? String ?? ""
    print("title:\\(title)")
    print("artist:\\(artist)")
    sem.signal()
}
DispatchQueue.global().async {
    Thread.sleep(forTimeInterval: 2)
    sem.signal()
}
RunLoop.main.run(until: Date(timeIntervalSinceNow: 3))
"""

WIFI_SWIFT = """\
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

MIC_CAMERA_SWIFT = """\
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

print("camera:\\(camActive)")
print("mic:\\(micActive)")
"""

# ---------------------------------------------------------------------------
# Config — Transparent theme
# ---------------------------------------------------------------------------

PAGE_SIZE = 16384
MAX_TITLE_LEN = 30
MAX_ARTIST_LEN = 20

# Status colors — only used for dynamic indicators (CPU load, battery, etc.)
GREEN = "#34c759"
ORANGE = "#ff9500"
RED = "#ff3b30"

# ---------------------------------------------------------------------------
# ranma helpers
# ---------------------------------------------------------------------------


def ranma_add(name, **kwargs):
    args = ["ranma", "add", name]
    for key, value in kwargs.items():
        flag = "--" + key.replace("_", "-")
        args.extend([flag, str(value)])
    subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def ranma_set(name, **kwargs):
    args = ["ranma", "set", name]
    for key, value in kwargs.items():
        if value is None:
            continue
        flag = "--" + key.replace("_", "-")
        args.extend([flag, str(value)])
    subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def ranma_remove(name):
    subprocess.run(
        ["ranma", "remove", name],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


# ---------------------------------------------------------------------------
# Swift helper management
# ---------------------------------------------------------------------------


def ensure_swift_scripts():
    cache_dir = os.path.join(tempfile.gettempdir(), "ranma-helpers")
    os.makedirs(cache_dir, exist_ok=True)
    scripts = {}
    for name, src in [
        ("nowplaying.swift", NOWPLAYING_SWIFT),
        ("wifi.swift", WIFI_SWIFT),
    ]:
        path = os.path.join(cache_dir, name)
        if not os.path.exists(path):
            with open(path, "w") as f:
                f.write(src)
        scripts[name.split(".")[0]] = path

    mic_bin = os.path.join(cache_dir, "mic_camera")
    mic_src = os.path.join(cache_dir, "mic_camera.swift")
    if not os.path.exists(mic_bin):
        with open(mic_src, "w") as f:
            f.write(MIC_CAMERA_SWIFT)
        subprocess.run(
            ["swiftc", "-O", "-o", mic_bin, mic_src],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    scripts["mic_camera"] = mic_bin
    return scripts


# ---------------------------------------------------------------------------
# Data collection
# ---------------------------------------------------------------------------


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


def get_battery():
    try:
        r = subprocess.run(
            ["pmset", "-g", "batt"], capture_output=True, text=True
        )
        output = r.stdout
        m = re.search(
            r"(\d+)%;\s*(charging|charged|discharging|finishing charge)", output
        )
        if m:
            pct = int(m.group(1))
            charging = m.group(2) in ("charging", "finishing charge")
            return pct, charging
    except Exception:
        pass
    return None, False


def get_wifi(script):
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


def get_now_playing(script):
    try:
        r = subprocess.run(
            ["swift", script], capture_output=True, text=True, timeout=10
        )
        title = ""
        artist = ""
        for line in r.stdout.strip().split("\n"):
            if line.startswith("title:"):
                title = line[6:]
            elif line.startswith("artist:"):
                artist = line[7:]
        return title, artist
    except Exception:
        return "", ""


def get_mic_camera(binary):
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


# ---------------------------------------------------------------------------
# Display helpers
# ---------------------------------------------------------------------------


def truncate(s, maxlen):
    if len(s) <= maxlen:
        return s
    return s[: maxlen - 1] + "\u2026"


def cpu_color(pct):
    if pct >= 80:
        return RED
    if pct >= 50:
        return ORANGE
    return GREEN


def mem_color(used, total):
    if total == 0:
        return GREEN
    ratio = used / total
    if ratio >= 0.85:
        return RED
    if ratio >= 0.70:
        return ORANGE
    return GREEN


def format_mem(bytes_val):
    gb = bytes_val / (1024 ** 3)
    return f"{gb:.1f}G"


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


def battery_color(pct, charging):
    if charging:
        return GREEN
    if pct is not None and pct < 10:
        return RED
    if pct is not None and pct < 25:
        return ORANGE
    return None


def wifi_icon(rssi):
    if rssi >= -70:
        return "wifi"
    return "wifi.exclamationmark"


def wifi_color(rssi):
    if rssi >= -50:
        return GREEN
    if rssi >= -70:
        return None
    return ORANGE


def wifi_label(info):
    if not info["power"]:
        return "Off"
    if info["rssi"] == 0:
        return "No Signal"
    if info["ssid"]:
        return info["ssid"]
    rssi = info["rssi"]
    if rssi >= -50:
        return "Strong"
    if rssi >= -70:
        return "Fair"
    return "Weak"


# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

POS = 1


def next_pos():
    global POS
    p = POS
    POS += 1
    return str(p)


def setup():
    ranma_add(
        "bar",
        type="row",
        align_items="center",
        corner_radius="0",
        margin_horizontal="0",
        margin_vertical="0",
        padding_horizontal="4",
        padding_vertical="0",
        gap="14",
        notch_align="right",
    )

    # Clock
    ranma_add(
        "bar.clock",
        type="row",
        parent="bar",
        align_items="center",
        gap="5",
        position=next_pos(),
    )
    ranma_add(
        "bar.clock.time",
        parent="bar.clock",
        label="--:--",
    )
    ranma_add(
        "bar.clock.date",
        parent="bar.clock",
        label="---",
    )

    # CPU/Mem
    ranma_add(
        "bar.cpu",
        type="row",
        parent="bar",
        align_items="center",
        gap="4",
        position=next_pos(),
    )
    ranma_add(
        "bar.cpu.icon",
        parent="bar.cpu",
        icon="cpu",
    )
    ranma_add(
        "bar.cpu.val",
        parent="bar.cpu",
        label="--%",
    )
    ranma_add(
        "bar.mem",
        type="row",
        parent="bar",
        align_items="center",
        gap="4",
        position=next_pos(),
    )
    ranma_add(
        "bar.mem.icon",
        parent="bar.mem",
        icon="memorychip",
    )
    ranma_add(
        "bar.mem.val",
        parent="bar.mem",
        label="--G",
    )

    # Battery
    ranma_add(
        "bar.bat",
        type="row",
        parent="bar",
        align_items="center",
        gap="4",
        position=next_pos(),
    )
    ranma_add(
        "bar.bat.icon",
        parent="bar.bat",
        icon="battery.100percent",
    )
    ranma_add(
        "bar.bat.pct",
        parent="bar.bat",
        label="--%",
    )

    # Wi-Fi
    ranma_add(
        "bar.wifi",
        type="row",
        parent="bar",
        align_items="center",
        gap="4",
        position=next_pos(),
    )
    ranma_add(
        "bar.wifi.icon",
        parent="bar.wifi",
        icon="wifi",
    )
    ranma_add(
        "bar.wifi.label",
        parent="bar.wifi",
        label="--",
    )


# ---------------------------------------------------------------------------
# Dynamic sections (now playing / mic-camera)
# ---------------------------------------------------------------------------

NP_POS = "20"
PRIV_POS = "21"


def add_np_section(title, artist):
    ranma_add(
        "bar.np",
        type="row",
        parent="bar",
        align_items="center",
        gap="5",
        position=NP_POS,
    )
    ranma_add(
        "bar.np.icon",
        parent="bar.np",
        icon="music.note",
    )
    ranma_add(
        "bar.np.title",
        parent="bar.np",
        label=truncate(title, MAX_TITLE_LEN),
    )
    if artist:
        ranma_add(
            "bar.np.artist",
            parent="bar.np",
            label=truncate(artist, MAX_ARTIST_LEN),
        )


def remove_np_section():
    ranma_remove("bar.np")


def add_priv_section(cam, mic):
    ranma_add(
        "bar.priv",
        type="row",
        parent="bar",
        align_items="center",
        gap="5",
        position=PRIV_POS,
    )
    if cam:
        ranma_add(
            "bar.priv.cam",
            parent="bar.priv",
            icon="camera.fill",
            icon_color=GREEN,
        )
    if mic:
        ranma_add(
            "bar.priv.mic",
            parent="bar.priv",
            icon="mic.fill",
            icon_color=ORANGE,
        )


def remove_priv_section():
    ranma_remove("bar.priv")


# ---------------------------------------------------------------------------
# Update functions
# ---------------------------------------------------------------------------


def update_clock():
    now = datetime.now()
    ranma_set("bar.clock.time", label=now.strftime("%H:%M"))
    ranma_set("bar.clock.date", label=now.strftime("%a %d"))


def update_cpu_mem(total_mem):
    cpu = get_cpu()
    used_mem = get_used_mem()
    ranma_set("bar.cpu.val", label=f"{cpu:.0f}%")
    ranma_set("bar.cpu.icon", icon_color=cpu_color(cpu))
    ranma_set("bar.mem.val", label=format_mem(used_mem))
    ranma_set("bar.mem.icon", icon_color=mem_color(used_mem, total_mem))


def update_battery():
    pct, charging = get_battery()
    ranma_set(
        "bar.bat.icon",
        icon=battery_icon(pct, charging),
        icon_color=battery_color(pct, charging),
    )
    ranma_set("bar.bat.pct", label=f"{pct}%" if pct is not None else "--%")


def update_wifi(script):
    info = get_wifi(script)
    if not info["power"]:
        ranma_set("bar.wifi.icon", icon="wifi.slash", icon_color=RED)
        ranma_set("bar.wifi.label", label="Off")
    elif info["rssi"] == 0:
        ranma_set("bar.wifi.icon", icon="wifi.slash")
        ranma_set("bar.wifi.label", label="--")
    else:
        ranma_set(
            "bar.wifi.icon",
            icon=wifi_icon(info["rssi"]),
            icon_color=wifi_color(info["rssi"]),
        )
        ranma_set("bar.wifi.label", label=wifi_label(info))


def update_np(script, np_visible):
    title, artist = get_now_playing(script)
    if title:
        if not np_visible:
            add_np_section(title, artist)
        else:
            ranma_set("bar.np.title", label=truncate(title, MAX_TITLE_LEN))
            if artist:
                ranma_set(
                    "bar.np.artist",
                    label=truncate(artist, MAX_ARTIST_LEN),
                )
        return True
    else:
        if np_visible:
            remove_np_section()
        return False


def update_priv(binary, prev_priv):
    cam, mic = get_mic_camera(binary)
    state = (cam, mic)
    if state == prev_priv:
        return prev_priv
    prev_active = prev_priv[0] or prev_priv[1]
    active = cam or mic
    if prev_active:
        remove_priv_section()
    if active:
        add_priv_section(cam, mic)
    return state


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------


def run():
    scripts = ensure_swift_scripts()
    total_mem = get_total_mem()

    setup()

    np_visible = False
    priv_state = (False, False)
    tick = 0

    while True:
        update_clock()

        if tick % 5 == 0:
            update_cpu_mem(total_mem)

        if tick % 30 == 0:
            update_battery()

        if tick % 10 == 0:
            update_wifi(scripts["wifi"])

        if tick % 3 == 0:
            np_visible = update_np(scripts["nowplaying"], np_visible)

        if tick % 2 == 0:
            priv_state = update_priv(scripts["mic_camera"], priv_state)

        tick += 1
        time.sleep(1)


if __name__ == "__main__":
    run()
