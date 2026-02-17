# Example Widgets

Example widgets demonstrating ranma's capabilities. Each example can be used as an init script:

```sh
PATH=./target/debug:$PATH ./target/debug/ranma-server start --init ./examples/<name>/init
```

All examples require [reap](https://github.com/typester/reap) (`brew install typester/reap/reap`) for process lifecycle management.

---

## Unified

![unified](../assets/statusbar-unified.png)

A single combined status bar containing clock, CPU/memory, battery, Wi-Fi, now playing, and mic/camera indicators. All data is managed by one widget process with staggered update intervals.

**Prerequisites:** Python 3, Swift (Xcode CLT)

---

## Island

![island](../assets/statusbar-island.png)

Dynamic Island style variant of the unified bar. Attaches to the top edge of the screen with only the bottom corners visible, mimicking Apple's Dynamic Island appearance. Designed for non-notch displays.

**Prerequisites:** Python 3, Swift (Xcode CLT)

---

## All

![all](../assets/statusbar-all.png)

Launches all individual widgets simultaneously. Each widget creates its own separate container, displayed as individual pills on the menu bar.

**Prerequisites:** Python 3, Swift (Xcode CLT)

---

## Individual Widgets

The following widgets can be used standalone or combined via the `all` example.

### Clock

![clock](../assets/example-clock.png)

Displays current time (HH:MM) and day/date. Updates every second.

**Prerequisites:** Python 3

### CPU & Memory

![cpu-mem](../assets/example-cpu-mem.png)

Real-time CPU usage percentage and memory consumption in GB. Color-coded icons: green (normal), orange (moderate), red (high). Updates every 5 seconds.

**Prerequisites:** Python 3

### Battery

![battery](../assets/example-battery.png)

Battery level with SF Symbol icon that changes based on charge level and charging state. Color indicates status: green (charging), orange (low), red (critical). Updates every 30 seconds.

**Prerequisites:** Python 3

### Wi-Fi

![wifi](../assets/example-wifi.png)

Wi-Fi signal strength indicator. Displays signal quality (Strong/Fair/Weak) with color-coded icon. Updates every 10 seconds.

**Prerequisites:** Python 3, Swift (Xcode CLT)

### Now Playing

![now-playing](../assets/example-now-playing.png)

Currently playing media title and artist from any macOS media source. Automatically hides when nothing is playing. Updates every 3 seconds.

**Prerequisites:** Python 3, Swift (Xcode CLT)

### Mic & Camera

Privacy indicator that appears when the microphone or camera is actively in use. Camera shows a green icon, microphone shows an orange icon. Automatically hides when neither is in use. Updates every 2 seconds.

**Prerequisites:** Python 3, Swift (Xcode CLT)

---

## Yashiki Workspace

![yashiki-workspace](../assets/statusbar-yashiki-workspace.png)

Workspace indicator for the [yashiki](https://github.com/typester/yashiki) tiling window manager. Displays workspace tags (1-10) per display with visual states for active, occupied, and vacant workspaces. Updates in real-time via yashiki event subscription.

**Prerequisites:** Python 3, yashiki, Hack Nerd Font (`brew install --cask font-hack-nerd-font`)
