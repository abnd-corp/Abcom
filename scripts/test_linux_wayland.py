#!/usr/bin/env python3
"""Exercise native Wayland rendering and detect an idle CPU spin on disposable CI."""

import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    if os.environ.get("CI") != "true":
        parser.error("run only on disposable Linux CI or a test container with CI=true")
    with tempfile.TemporaryDirectory(prefix="abcom-wayland-") as directory:
        root = Path(directory)
        for name in ["runtime", "home"]:
            (root / name).mkdir(mode=0o700)
        env = os.environ.copy()
        for key in ["DISPLAY", "WAYLAND_SOCKET", "WINIT_UNIX_BACKEND"]:
            env.pop(key, None)
        env.update(HOME=str(root / "home"), XDG_RUNTIME_DIR=str(root / "runtime"),
                   XDG_DATA_HOME=str(root / "home/data"), XDG_CONFIG_HOME=str(root / "home/config"),
                   WAYLAND_DISPLAY="wayland-abcom", WGPU_BACKEND="vulkan")
        with (root / "weston.log").open("w") as weston_log, (root / "app.log").open("w") as app_log:
            weston = subprocess.Popen([
                "weston", "--backend=headless-backend.so", "--use-pixman",
                "--socket=wayland-abcom", "--idle-time=0", "--width=1280", "--height=800",
            ], env=env, cwd=root, stdout=weston_log, stderr=weston_log)
            app = None
            try:
                for _ in range(100):
                    if (root / "runtime/wayland-abcom").exists():
                        break
                    if weston.poll() is not None:
                        raise RuntimeError("Weston exited before opening its socket")
                    time.sleep(0.1)
                else:
                    raise RuntimeError("Weston socket timeout")
                app = subprocess.Popen([str(binary), "wayland-test"],
                                       env={**env, "WAYLAND_DEBUG": "1"}, cwd=root,
                                       stdout=app_log, stderr=app_log)
                time.sleep(10)

                def ticks():
                    if app.poll() is not None:
                        raise RuntimeError(f"Abcom exited: {app.returncode}")
                    fields = Path(f"/proc/{app.pid}/stat").read_text().rsplit(")", 1)[1].split()
                    return int(fields[11]) + int(fields[12])

                def sample(state):
                    initial, start = ticks(), time.monotonic()
                    time.sleep(12)
                    cpu = 100 * (ticks() - initial) / os.sysconf("SC_CLK_TCK") / (time.monotonic() - start)
                    print(json.dumps({"state": state, "cpu_percent_of_one_core": round(cpu, 2)}), flush=True)
                    if cpu >= 20:
                        raise RuntimeError(f"excessive idle CPU in {state}: {cpu:.2f}%")

                sample("idle")
                weston.send_signal(signal.SIGSTOP)
                try:
                    sample("compositor_suspended")
                finally:
                    weston.send_signal(signal.SIGCONT)
                time.sleep(2)
                sample("resumed")
                protocol = (root / "app.log").read_text(errors="replace")
                if not all(marker in protocol for marker in ["xdg_wm_base", "wl_surface", "attach("]):
                    raise RuntimeError("no native Wayland surface with a buffer observed")
            except Exception:
                print((root / "weston.log").read_text(errors="replace")[-4000:])
                print((root / "app.log").read_text(errors="replace")[-6000:])
                raise
            finally:
                if app is not None and app.poll() is None:
                    app.terminate()
                    try:
                        app.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        app.kill()
                        app.wait()
                if weston.poll() is None:
                    weston.send_signal(signal.SIGCONT)
                    weston.terminate()
                    try:
                        weston.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        weston.kill()
                        weston.wait()


if __name__ == "__main__":
    main()
