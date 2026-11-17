**English** · [Français](README.fr.md)

# Bundled font styles

Regular and bold fonts retain their existing license notices. Additional italic files come from the fixed `@fontsource/*` **5.3.0** packages, under OFL-1.1. [italic-sources.json](italic-sources.json) records every URL, SHA-256, byte count and applicable license notice. The 24 additional WOFF2 files total **704,512 bytes**; they are decoded on demand and cached.

| Family | Italic | Bold italic |
| --- | --- | --- |
| Noto Sans, Inter | Yes | Yes |
| Atkinson Hyperlegible, OpenDyslexic, Literata, Nunito, JetBrains Mono | Yes | Yes |
| Lexend | Synthetic slant | Bold with synthetic slant |

Latin and Latin Extended subsets are bundled where available. Symbol and missing-glyph fallbacks remain available. These files are bundled locally; selecting a bundled font sends no network request.

Additional copyright/license notices are retained in [styles/licenses](styles/licenses/). OpenDyslexic uses its existing [OFL notice](suggestions/opendyslexic/OFL.txt). Upstream distributions: [Fontsource](https://fontsource.org/), [Inter](https://github.com/rsms/inter), [Noto Sans](https://github.com/notofonts/latin-greek-cyrillic).

For personal style association, fallback behavior and supported formats, see the [user guide](../../docs/en/05-features.md#settings-and-appearance).
