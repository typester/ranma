#!/usr/bin/env python3

"""Now Playing plugin for ranma status bar.

Displays currently playing media (title and artist) using macOS
MediaRemote private framework via a Swift script.
Polls every 3 seconds. Hides when nothing is playing.

Note: MediaRemote requires the swift interpreter's TCC entitlements,
so the helper runs via `swift <script>` rather than as a compiled binary.
"""

import os
import subprocess
import tempfile
import time

POLL_INTERVAL = 3
MAX_TITLE_LEN = 30
MAX_ARTIST_LEN = 20

SWIFT_SRC = """\
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


def ranma_remove(name):
    subprocess.run(
        ["ranma", "remove", name],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def ensure_script():
    cache_dir = os.path.join(tempfile.gettempdir(), "ranma-helpers")
    os.makedirs(cache_dir, exist_ok=True)
    src = os.path.join(cache_dir, "nowplaying.swift")
    if not os.path.exists(src):
        with open(src, "w") as f:
            f.write(SWIFT_SRC)
    return src


def truncate(s, maxlen):
    if len(s) <= maxlen:
        return s
    return s[: maxlen - 1] + "\u2026"


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


def create_container():
    ranma_add(
        "np",
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
        gap="5",
        notch_align="right",
    )
    ranma_add(
        "np.icon",
        parent="np",
        icon="music.note",
        icon_color="#ff6b9d",
        font_size="10",
        position="1",
    )
    ranma_add(
        "np.title",
        parent="np",
        label="",
        label_color="#ffffff",
        font_size="10",
        font_weight="medium",
        position="2",
    )
    ranma_add(
        "np.artist",
        parent="np",
        label="",
        label_color="#ffffff60",
        font_size="9",
        position="3",
    )


def run():
    script = ensure_script()
    visible = False

    while True:
        title, artist = get_now_playing(script)

        if title:
            if not visible:
                create_container()
                visible = True
            ranma_set("np.title", label=truncate(title, MAX_TITLE_LEN))
            ranma_set(
                "np.artist",
                label=truncate(artist, MAX_ARTIST_LEN) if artist else "",
            )
        else:
            if visible:
                ranma_remove("np")
                visible = False

        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    run()
