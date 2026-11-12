# 06 — Installation

**English** · [Français](../06-installation.md) · [Documentation](../README.md)

## Requirements

- A Rust toolchain satisfying `rust-version` in [Cargo.toml](../../Cargo.toml), currently 1.95 or newer.
- A graphical desktop and compatible graphics driver.
- Peers on the same LAN, with TCP 9000/9001 and UDP 9001 reachable between them.

macOS, Windows and Linux have build checks in CI. Desktop integration still needs manual testing on each target OS. The [release workflow](../../.github/workflows/release.yml) targets Linux x86-64, macOS Apple silicon and Windows x86-64; other targets require a source build and validation. Consult [releases](https://github.com/abnd-corp/Abcom/releases) for actually published artifacts; a configured workflow does not mean a binary has been published.

## Linux build dependencies

Install the compiler/toolchain prerequisites for your distribution, plus the relevant packages below. They are needed during compilation, not just when running.

| Need | Debian / Ubuntu | Fedora | Arch | Alpine |
|---|---|---|---|---|
| Keyboard/windowing | `libxkbcommon-dev` | `libxkbcommon-devel` | `libxkbcommon` | `libxkbcommon-dev` |
| Sound | `libasound2-dev` | `alsa-lib-devel` | `alsa-lib` | `alsa-lib-dev` |

Package names depend on distribution versions. The tray uses the KSNI/StatusNotifierItem D-Bus backend: GTK and AppIndicator are not required. A session bus and a desktop implementing StatusNotifierItem are needed to display the icon. Native file pickers use the desktop portal. Sound dependencies can be removed by disabling the sound feature.

## Build and run

```sh
git clone https://github.com/abnd-corp/Abcom.git
cd Abcom
cargo run --release -- alice
```

Choose a distinct username on each computer. Without an argument, Abcom uses `USER` or `USERNAME`. The executable is `target/release/abcom` (`abcom.exe` on Windows).

From the project root, `make run` starts a release build. Quit from the tray menu before testing a restart: closing the window normally only hides it.

### Optional build features

`tray` and `sound` are on by default. `avatar-svg` is off by default.

```sh
# No tray or sound; no ALSA requirement
cargo build --release --locked --no-default-features

# Sound, without tray
cargo build --release --locked --no-default-features --features sound
```

Without tray, closing exits. Without sound, the notification beep is removed; system notifications remain. A minimal build is still a GUI application and needs windowing and graphics support.

### Alpine and musl

Dynamic musl builds are the documented Alpine approach. The GUI/driver stack loads libraries dynamically; a completely static musl GUI binary is not supported. Disabling tray and sound reduces dependencies but does not make the application headless or fully static.

## Environment variables

Abcom reads these from the environment or a local `.env` in its working directory:

| Variable | Purpose |
|---|---|
| `ABCOM_PASSPHRASE` | Shared room passphrase, required identically on all participating peers |
| `ABCOM_KLIPY_API_KEY` | Optional Klipy GIF/meme/sticker API key |
| `ABCOM_INSTANCE` | Separate ports and data for local instances; bounded to the valid port range |

Keep credentials out of commits, screenshots and issue reports. Neither a passphrase nor a Klipy key is required for ordinary local messaging. Online fonts are enabled separately in Settings, not by these variables.

## Try two local peers

On macOS or Linux:

```sh
make run2
# Three peers: make rung
# More peers: make run-multi N=4
```

Or run these in separate terminals (POSIX shell):

```sh
ABCOM_INSTANCE=1 cargo run --release -- alice
ABCOM_INSTANCE=2 cargo run --release -- bob
```

In PowerShell, set `$env:ABCOM_INSTANCE = '1'` before starting Alice, and `'2'` in Bob's separate terminal. Instances have separate data directories, so settings from the default instance do not follow them.

## Linux installation

`make install` builds release, copies the executable to `~/.local/bin/abcom`, creates an application-menu launcher with an absolute path and removes a legacy systemd service if present. It does not install a new service.

Login startup uses one XDG entry in `~/.config/autostart/`, controlled in Settings → General. To install an already built binary, run `bash scripts/abcom-install.sh ./target/release/abcom`. Remove it with `make uninstall` or the uninstall script. Uninstalling the executable is not a backup or data-deletion procedure.

If UFW is in use, permit trusted-LAN traffic:

```sh
sudo ufw allow 9000:9001/tcp
sudo ufw allow 9001/udp
```

GNOME may need a StatusNotifierItem-compatible extension; KDE/XFCE typically provide a tray. KSNI registers the icon when a watcher becomes available. If tray creation fails, closing exits; a successfully created tray awaiting a panel may not yet be visible. Verify hide/reopen behavior on the target desktop before distribution.

## macOS

Run from source with `cargo run --release -- alice`. The application manages menu-bar integration and its optional Launch Agent. The Dock uses the high-resolution colour logo; the menu bar uses an alpha template automatically adapted to light/dark backgrounds, including its unread badge.

Create a bundle with its native Finder/Dock icon:

```sh
cargo build --release --locked
python3 scripts/package_macos.py target/release/abcom dist/Abcom.app
open dist/Abcom.app
```

The release workflow archives this `.app` bundle (`corp.abnd.abcom`). It is not Developer-ID signed or notarized; review its origin and published SHA-256 before following security prompts. Notifications from a bare executable may still be attributed to the terminal; verify attribution when launching the bundle on the target system.

## Windows

Use PowerShell in a Windows checkout:

```powershell
cd C:\path\to\Abcom
powershell -ExecutionPolicy Bypass -File .\scripts\install-windows.ps1
```

Review the script before running it. It checks Cargo, builds release when needed, installs in `%LOCALAPPDATA%\Programs\abcom` and adds desktop/Start-menu shortcuts. The multi-resolution ICO is embedded in the executable and used by shortcuts. A full MSI installer is not provided. Native PowerShell is the documented route; WSL builds do not automatically create a native Windows GUI application.

## Docker for multi-peer tests

```sh
cd scripts/docker
docker compose up --build
```

The development image includes GUI dependencies and starts Alice, Bob and Charlie with host networking. It requires access to the host X11 server. It is a Linux desktop testing setup, not a headless messaging server or a portable Docker Desktop installation recipe.

## Make targets

| Target | Purpose |
|---|---|
| `make build` / `make release` | Debug / release build |
| `make run` | Run release |
| `make run2` / `make rung` / `make run-multi N=4` | Multiple local peers |
| `make install` / `make uninstall` | Linux installation / removal |
| `make check` | Formatting, strict Clippy and all-feature tests |
| `make test` / `make test-verbose` | Default-feature tests |
| `make test-module M=app::groups` | One test module |
