**English** · [Français](README.fr.md)

# Abcom visual assets

Vector reconstruction of the supplied reference: two opposing links, coral `#FF8A60`, ink `#302830`, cream `#FFF8EF`.

![Asset overview](preview-en.png)

| SVG master in `svg/` | Use |
|---|---|
| `abcom-symbol` / `abcom-symbol-on-dark` | Two-colour symbol on light / dark backgrounds |
| `abcom-symbol-ink` / `-cream` / `-coral` | Single-colour symbol |
| `abcom-symbol-outline` | Outlined version |
| `abcom-wordmark` / `abcom-wordmark-on-dark` | Symbol and outlined ABCOM lettering |
| `abcom-app-light` / `abcom-app-dark` | Rounded application icons |
| `abcom-dock` | Light application icon for the macOS Dock |
| `abcom-tray-light` / `abcom-tray-dark` | Ink / cream tray symbol |
| `abcom-tray-template` | Black alpha mask for a macOS template image |

PNG exports are in `png/`, with transparent backgrounds. Application icons: 16–1024 px. Tray: 16, 24, 32, 48 and 64 px; use 32, 48 and 64 px for the 2× versions of 16, 24 and 32 logical pixels. Wordmarks: 540, 1080 and 2160 px wide. SVG lettering is converted into paths and requires no installed font.

Preserve the proportions and the space between the links. Choose the background-appropriate variant. The template image must be marked as a template by the native macOS integration; the filename alone does not enable that behaviour. The current application integration is separate from this asset pack.

To regenerate from the repository root, using the existing optional SVG dependency:

```sh
python3 scripts/generate_branding.py
cargo run --locked --example export_branding --features avatar-svg
```

`exports.json` lists the exact filenames and dimensions; `palette.json` holds the palette. The renderer checks that exports are visible, that icon backgrounds remain transparent and that the template stays monochrome. Inspect `preview.png` and the native-size tray PNGs after changing the geometry.

Covered by the repository [license](../../LICENSE). Lettering uses the bundled [Inter font](../fonts/Inter-OFL.txt).
