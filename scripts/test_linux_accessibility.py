#!/usr/bin/env python3
"""Check the real AT-SPI tree and action dispatch in a disposable desktop session."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--tray", action="store_true", help="also require a real StatusNotifier panel")
    args = parser.parse_args()
    if os.environ.get("CI") != "true":
        parser.error("run only in a disposable desktop with CI=true")
    import dbus
    import pyatspi

    bus = dbus.SessionBus()
    status = bus.get_object("org.a11y.Bus", "/org/a11y/bus")
    properties = dbus.Interface(status, "org.freedesktop.DBus.Properties")
    previous = {name: properties.Get("org.a11y.Status", name)
                for name in ("IsEnabled", "ScreenReaderEnabled")}
    process = None
    try:
        for name in previous:
            properties.Set("org.a11y.Status", name, dbus.Boolean(True))
        process = subprocess.Popen([str(args.binary.resolve(strict=True))])

        def nodes():
            desktop = pyatspi.Registry.getDesktop(0)
            desktop.clearCache()
            for app in desktop:
                if app.get_process_id() != process.pid:
                    continue
                pending = [(app, 0)]
                count = 0
                while pending and count < 500:
                    node, depth = pending.pop()
                    node.clearCache()
                    count += 1
                    yield node
                    if depth < 12:
                        pending.extend((child, depth + 1) for child in node)

        def find(name):
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"probe exited early: {process.returncode}")
                for node in nodes():
                    if node.name == name:
                        return node
                time.sleep(0.2)
            raise RuntimeError(f"AT-SPI node unavailable: {name}")

        button = find("Accessible action")
        action = button.queryAction()
        if action.nActions < 1 or not action.doAction(0):
            raise RuntimeError("AT-SPI button action failed")
        find("Action count: 1")
        if args.tray:
            daemon = dbus.Interface(bus.get_object("org.freedesktop.DBus", "/org/freedesktop/DBus"),
                                    "org.freedesktop.DBus")
            watcher = bus.get_object("org.kde.StatusNotifierWatcher", "/StatusNotifierWatcher")
            items = dbus.Interface(watcher, "org.freedesktop.DBus.Properties").Get(
                "org.kde.StatusNotifierWatcher", "RegisteredStatusNotifierItems")
            matched = False
            for item in items:
                service, _, path = str(item).partition("/")
                if int(daemon.GetConnectionUnixProcessID(service)) != process.pid:
                    continue
                icon = bus.get_object(service, "/" + path if path else "/StatusNotifierItem")
                values = dbus.Interface(icon, "org.freedesktop.DBus.Properties").GetAll(
                    "org.kde.StatusNotifierItem")
                pixmaps = values["IconPixmap"]
                if not pixmaps or not all(w > 0 and h > 0 and len(data) == 4 * w * h
                                          for w, h, data in pixmaps):
                    raise RuntimeError("invalid native tray pixmap")
                menu = dbus.Interface(bus.get_object(service, values["Menu"]),
                                      "com.canonical.dbusmenu")
                _, layout = menu.GetLayout(0, -1, dbus.Array([], signature="s"))
                pending, labels = [layout], set()
                while pending:
                    _, attributes, children = pending.pop()
                    labels.add(str(attributes.get("label", "")))
                    pending.extend(children)
                if not {"Open", "Quit"}.issubset(labels):
                    raise RuntimeError("native tray menu missing actions")
                matched = True
                break
            if not matched:
                raise RuntimeError("probe missing from the StatusNotifier panel")
        print(json.dumps({"session": os.environ.get("XDG_SESSION_TYPE"),
                          "tree": "ok", "button_action": "ok",
                          "tray": "ok" if args.tray else "not requested"}), flush=True)
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            process.wait(timeout=10)
        for name, value in previous.items():
            properties.Set("org.a11y.Status", name, value)


if __name__ == "__main__":
    main()
