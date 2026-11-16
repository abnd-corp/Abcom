**English** · [Français](README.fr.md)

# Linux D-Bus adapters

These are local source patches of active upstream projects, not official upstream releases. Abcom takes responsibility for these patches until equivalent releases are available. They preserve the existing Linux tray and accessibility integration while replacing the unmaintained standalone zvariant, zvariant_derive and zbus_names crates with zbus 6.

| Source | Imported version | Local change |
| --- | --- | --- |
| AccessKit: accesskit_unix, accesskit_atspi_common | 0.24.1, 0.21.1 | zbus 6 types, builders, object registration and connection runtime |
| AT-SPI: atspi, atspi-common, atspi-proxies | 0.29.0, 0.13.0, 0.13.0 | zbus 6 codecs and proxies; explicit checked standard/Qt event decoding |
| KSNI | 0.3.7 | Upstream zbus 6 port, adapted to the published package manifest |

Original licenses are preserved beside each package. `upstream.json` records the registry archive checksums and upstream references. Files modified locally carry a modification notice. No upstream version number is presented as a new official release.

Review the changes against each registry archive in `patches/<package>.patch`.

The runtime graph uses zbus 6. AT-SPI protocol tests also use maintained zbus 6 tools (`zbus-lockstep` 0.8 and `zbus_xml` 6). No RustSec exception is required by Abcom. The adapters require Rust 1.89; Abcom's existing minimum remains 1.95.

## Maintenance

Follow [AccessKit](https://github.com/AccessKit/accesskit), [AT-SPI](https://github.com/odilia-app/atspi) and the [KSNI migration](https://github.com/iovxw/ksni/pull/48). Review upstream changes whenever these packages are updated. Do not replace a local adapter with an upstream release until its complete runtime graph is free of the retired crates.

Run Abcom's `make check`, `make check-dbus`, and the Linux D-Bus integration test. `check-dbus` tests the three accessibility adapters, including both AccessKit runtimes separately, using disposable manifests with the application's source patches. Validate the tray/menu/badge and AT-SPI tree with the Linux test desktops. Root Cargo patches also apply on macOS/Windows dependency resolution; they change only the Linux runtime adapters.

On Linux, `dbus-run-session -- python3 scripts/test_dbus_adapters.py --ksni` also tests KSNI's library. The disposable KSNI test manifest omits GTK/portal dependencies used only by upstream examples.

When an upstream release includes the migration, remove the corresponding `[patch.crates-io]` entry and source directory together, update the lockfile and rerun audits and native tests. Remove this directory once all six patches are unnecessary. See the [decision](../docs/adr/0001-dbus-maintained.md).
