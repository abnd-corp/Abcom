**English** · [Français](0001-dbus-maintained.fr.md)

# Linux D-Bus maintenance

Status: proposed on the branch for issue #59, 2026-11-16.

AccessKit and KSNI remain active projects, but their published versions still use zbus 5. RustSec marks the standalone zvariant, zvariant_derive and zbus_names packages as unmaintained after their integration into zbus 6. A direct dependency update cannot migrate the downstream APIs. Replacing only the tray would leave the accessibility dependency chain unchanged.

Keep the existing integrations and carry six scoped source patches to use maintained zbus 6 APIs. Preserve upstream licenses and registry provenance in [vendor](../../vendor/README.md). The KSNI port follows its upstream PR; AccessKit and AT-SPI are adapted locally. This incurs maintenance responsibility for Abcom; it does not establish upstream support for these patches.

Keep checked deserialization for standard and Qt AT-SPI event formats, preserve the accessibility tree and retain StatusNotifier support on Wayland and X11. Do not disable accessibility, reintroduce GTK3/AppIndicator, or replace the tray with an X11-only implementation to silence an audit.

Remove the three audit exceptions only with the complete retired runtime chain removed. Validate formatting, Clippy, tests, audits, packaging source inclusion, Linux accessibility and tray behavior, and unaffected macOS/Windows operation. VM checks cover virtual displays; physical GPU and monitor transitions remain distinct tests.

Revisit the patches at each upstream dependency update and remove them when compatible releases carry equivalent changes. Keep #59 open until the implementation and verification are accepted into dev. Do not close it merely because a prototype compiles.
