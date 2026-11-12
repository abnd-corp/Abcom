**English** · [Français](README.fr.md)

# Install without public package catalogues

No Homebrew/core, microsoft/winget-pkgs or nixpkgs submission is required. Recipes stay with Abcom. Binary recipes require actual versioned release files with matching SHA-256 hashes; they never advertise a missing artifact. Check [GitHub releases](https://github.com/abnd-corp/Abcom/releases) for available downloads; preparing a recipe does not publish it.

## Direct GitHub installation versus catalogue names

`winget install --exact --id cjpais.Handy` works because [Handy has a manifest in Microsoft's catalogue](https://github.com/microsoft/winget-pkgs/tree/master/manifests/c/cjpais/Handy). That manifest downloads the installer from Handy's GitHub release. WinGet does not turn an arbitrary GitHub repository name into an application: Abcom still needs a local manifest, a configured package source, or a future catalogue entry. A catalogue entry can point to our GitHub files; it need not host our binaries.

Nix can directly use `github:abnd-corp/Abcom/<revision>#abcom` once that revision contains this flake and is available remotely. Homebrew needs its cask in a configured tap, or the local-tap helper below; it does not discover our GitHub release automatically. Flatpak needs a bundle or an OSTree remote, not a source-repository URL. None of these commands require a new GitHub repository, but a bare `brew install abcom`, `winget install Abnd.Abcom` or `flatpak install corp.abnd.abcom` is not globally available without configuring or registering a source first.

## Linux AppImage and Flatpak

Both formats are produced locally by `scripts/package_appimage.py` and `scripts/package_flatpak.sh`. They submit nothing to AppImageHub or Flathub. The dedicated Linux workflow creates test artifacts; the existing release workflow attaches them only when a release is explicitly triggered. Until then, use locally built files.

After an actual release exists, download the matching architecture (`x86_64` or `aarch64`) from its GitHub release. For example, with `VERSION` set to the released number without `v`:

```sh
BASE="https://github.com/abnd-corp/Abcom/releases/download/v$VERSION"
ARCH=$(uname -m)
curl --fail --location --remote-name "$BASE/Abcom-$VERSION-$ARCH.AppImage"
curl --fail --location --remote-name "$BASE/SHA256SUMS.txt"
sha256sum --check --ignore-missing SHA256SUMS.txt
chmod +x "Abcom-$VERSION-$ARCH.AppImage"
"./Abcom-$VERSION-$ARCH.AppImage"
```

AppImage requires no package catalogue or Rust installation. It is a portable executable, not a `.deb`/`.rpm` and not a Docker image. On hosts without FUSE, use `APPIMAGE_EXTRACT_AND_RUN=1 ./Abcom-…AppImage`. The bundled runtime and packaging tools have pinned SHA-256 hashes in `linux/appimage-tools.json`; an upstream moving asset fails verification instead of being silently trusted. Builds use Ubuntu 22.04 as the compatibility baseline. Windowing libraries loaded through `dlopen` are included explicitly. Sound uses the desktop's PulseAudio/compatible PipeWire service; GPU drivers still come from the host. Keep the AppImage at a durable path when enabling autostart.

With Flatpak already installed on Linux:

```sh
curl --fail --location --remote-name "$BASE/Abcom-$VERSION-$ARCH.flatpak"
# Verify with the release's SHA256SUMS.txt as above.
flatpak install --user "./Abcom-$VERSION-$ARCH.flatpak"
flatpak run corp.abnd.abcom
# Uninstall without deleting personal data:
flatpak uninstall --user corp.abnd.abcom
```

The bundle supplies the application's files and the Freedesktop 25.08 runtime reference. Flatpak obtains missing runtime/graphics extensions from Flathub; Abcom itself is not published there. A single-file bundle is not an offline runtime bundle and does not provide automatic Abcom updates; download and install the next version explicitly. An updateable remote could be added later without using Flathub, but an ordinary GitHub source tree is not an OSTree repository.

Flatpak grants LAN networking, graphics/audio, X11/Wayland and specific tray/notification D-Bus access. It grants neither the whole home directory nor the whole session bus. Attach files through the portal-backed picker; dropping a host path not granted by the portal may fail. Data stays in the application's sandbox, separate from native/AppImage data. Autostart is disabled and labelled unavailable in this Flatpak build until a Background portal integration exists. It does not write a misleading sandbox-only autostart file.

To build on Linux with the dependencies listed in `linux/Dockerfile`:

```sh
cargo build --release --locked
python3 scripts/package_appimage.py target/release/abcom
bash scripts/package_flatpak.sh target/release/abcom
```

The scripts reject architecture mismatches. AppImage tools are checksum-verified and extracted without FUSE. `test_linux_appimage.sh` and `test_linux_flatpak.sh` run with an isolated Xvfb display and disposable data; the latter downloads its runtime into a temporary user installation. Docker needs nested namespace capabilities for Flatpak export/tests, not personal folders or the Docker socket. `stage_linux_source.py` exports only declared public build inputs, excluding `.env`, `.git`, personal data and build directories. Tests in a virtual display do not validate every physical Linux desktop, portal or LAN.

Run the installation tests only on disposable CI runners/containers; both scripts require `CI=true`. The optional `ABCOM_FLATPAK_TEST_INSTALLATION` points to a dedicated test runtime cache, never a personal Flatpak installation. No additional application is installed on the macOS host for these Linux tests.

## Homebrew on macOS

For a distributed release, unpack `abcom-package-recipes.tar.gz` and run:

```sh
bash homebrew/install-homebrew.sh homebrew/Casks/abcom.rb
```

This creates `abnd-corp/abcom-local`, a local tap with no remote repository, and installs the application in Homebrew's app directory. Repeating the command with the recipe from a newer release reinstalls that version; there is no automatic remote tap update. Uninstall with `brew uninstall --cask abcom`. Personal data is retained; the Launch Agent is removed. An enabled autostart preference refreshes its executable path on the next launch.

**Signing:** the bundle has a verified ad-hoc signature, not a Developer ID signature or Apple notarization. A quarantined download can therefore be blocked, including with a “damaged” alert. Homebrew installation success is not proof that Gatekeeper accepts the application. A downloaded release needs an explicit macOS authorization where available, or Developer ID signing and notarization for a normal first launch. Do not disable Gatekeeper globally. See [Apple's explanation](https://support.apple.com/en-us/102445).

For a **build compiled on your own Mac**, prepare and test the cask from the repository:

```sh
cargo build --release --locked
python3 scripts/package_macos.py target/release/abcom dist/Abcom.app
mkdir -p dist/packages/artifacts
tar -czf dist/packages/artifacts/abcom-macos-aarch64.tar.gz -C dist Abcom.app
python3 scripts/prepare_packages.py --artifacts dist/packages/artifacts
bash scripts/install-homebrew.sh dist/packages/homebrew/Casks/abcom.rb --local-build
open /Applications/Abcom.app
```

`--local-build` only accepts a local `file:///` recipe. After Homebrew verifies the archive hash and installs it, the script verifies the bundle identifier and signature, then removes the download quarantine **only from this application**. It changes no global security setting. Use it only for code you built and trust; it is not a workaround for an unknown downloaded application. The recipe currently targets Apple silicon, matching the macOS release artifact.

## WinGet on Windows

The recipe targets a per-user Inno Setup installer, with a Start menu shortcut and an uninstaller. It preserves the application's data. From an unpacked release recipe archive:

```powershell
winget validate --manifest .\winget\Abnd.Abcom.yaml
winget install --manifest .\winget\Abnd.Abcom.yaml --accept-package-agreements
winget uninstall --id Abnd.Abcom
```

Local manifests must be allowed by the Windows package manager policy. If disabled, enabling `LocalManifestFiles` through `winget settings --enable LocalManifestFiles` requires an administrator; the installer itself does not. The manifest points to HTTPS release files. For unpublished tests, serve your own artifact over localhost HTTP and regenerate with `--base-url http://127.0.0.1:<port>`; WinGet cannot install a `file://` recipe generated for a local Mac test. [Manifest documentation](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest).

To build on Windows, install Inno Setup 6 alongside the Rust/Visual Studio build tools, then run:

```powershell
cargo build --release --locked
./scripts/package_windows.ps1 -BinaryPath target/release/abcom.exe
```

`scripts/test_windows_package.ps1` checks install, reinstall, shortcut creation, uninstall and data retention on a disposable Windows runner. It does not certify native visual rendering. CI runs it on Windows; no native WinGet test is claimed from macOS.

## Nix on Linux and macOS

From the repository, with flakes enabled:

```sh
nix build .#abcom
nix profile add .#abcom
# later:
nix profile remove abcom
```

Use a pinned checkout or commit when installing remotely. `flake.lock` pins nixpkgs and `Cargo.lock` pins crates. Nix supplies the toolchain, SQLite and, on Linux, ALSA plus the dynamically loaded graphics libraries. Linux gets its desktop file and hicolor icon; macOS gets `Applications/Abcom.app` inside the package. Add that application to your desktop environment explicitly if needed; a Nix profile does not automatically copy it to `/Applications`.

The source includes only the build inputs listed in `packaging/nix/package.nix`, never `.env`, local data or build directories. The build sandbox does not run network/native-window tests; those remain in `make check` and the desktop checklist. A graphical session, session D-Bus, desktop portal and an appropriate graphics driver are still system requirements. Removing the profile preserves personal data; disable autostart before uninstalling a Nix installation.

## Maintainers

`scripts/prepare_packages.py --artifacts <directory> --base-url <versioned-url>` generates cask and WinGet recipes using hashes of actual files. It refuses credentials in URLs and remote plain HTTP. The release workflow attaches these recipes, the Windows installer and `SHA256SUMS.txt` to the GitHub release; it submits nothing to external catalogues.

Docker's existing three-peer setup remains a Linux/X11 tool. Its build context now includes the build script and examples. It does not test WinGet, Gatekeeper, a macOS menu bar or a real Linux panel. The local installation/build report is in the [audit log](../docs/en/08-history-and-audits.md).
