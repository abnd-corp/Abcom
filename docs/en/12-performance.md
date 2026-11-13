# 12 — Measuring performance

**English** · [Français](../12-performances.md) · [Documentation](../README.md)

## Prepare a reproducible measurement

Build with `cargo build --release --locked`. Record commit, Rust version, features, OS, hardware, driver, pixel density and window size. Use two instances and disposable data, identical before and after the change.

Repeat each scenario at least three times; publish raw measurements and their spread. Separate cold startup, warm startup and steady state. A performance target must be agreed, not presented as an achieved result.

## Scenarios

| Scenario | Observations |
|---|---|
| Startup, then 100 messages | RSS and startup time; state whether fonts and media were already cached |
| Idle for 60 seconds | Average CPU, GPU and wake-ups; repeat foreground, background and hidden |
| Three GIF search pages, then 10 minutes idle | RSS, released textures and residual activity; record network and content |
| Scroll 500 messages with Markdown and media | Median/p95 frame times, responsiveness and parsing/cache costs |
| Receive messages and bursts | Visible latency, event cost, storage-thread writes and UI blocking |
| Transfer a 1 GB file | Throughput, both peers' CPU/RSS, disk and network setup |
| Hide and restore | Idle rendering, hidden-window reception and reopening delay |
| Prolonged use | Database/media size, cache growth, threads and resources |

## Tools and interpretation

On macOS, `ps -o rss= -p <pid>` measures RSS and Instruments profiles allocations and CPU. Activity Monitor provides CPU/GPU observations. On other systems use equivalent tools, recording versions and units. Do not compare distinct metrics such as RSS and allocated memory.

Separate evidence from assumptions: a peak is not an average, and one machine is not multi-OS validation. Dated results are in the [maintenance log](08-history-and-audits.md).

Old automatic attachment quotas no longer apply: attachments are purged only on request. Measure growth without destroying real user data. Do not change retention policy simply to improve a number.

## Wayland verification

Wayland runs natively when available; X11 remains supported through winit's automatic selection. The code no longer sets `WINIT_UNIX_BACKEND`, which winit removed. The CPU loop fix (eframe 0.36.2, #8398) and stalled callback fix (0.37.0, #8631) are included in the version in use; no dependency upgrade is needed.

The Linux workflow runs `scripts/test_linux_wayland.py` under headless Weston without `DISPLAY`, using disposable data and software Vulkan rendering. It checks for a Wayland surface receiving a buffer, measures idle CPU, suspends the compositor to block callbacks, then resumes it. Each sample lasts 12 seconds after warm-up; the regression threshold is 20% of one core. This detects a busy loop rather than promising CPU usage on every machine. Suspending the compositor does not replace testing minimized windows on each Linux desktop. X11 tests under Xvfb remain separate.

In a disposable Linux container with Weston, D-Bus and software Vulkan:

```sh
CI=true dbus-run-session -- python3 scripts/test_linux_wayland.py target/release/abcom
```
