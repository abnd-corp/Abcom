**English** · [Français](README.fr.md)

# Install without public package catalogues

No Homebrew/core, microsoft/winget-pkgs or nixpkgs submission is required. Recipes stay with Abcom. Binary recipes require actual versioned release files with matching SHA-256 hashes; they never advertise a missing artifact. There is no published release yet.

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
