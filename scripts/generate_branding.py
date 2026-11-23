#!/usr/bin/env python3
"""Generate editable Abcom SVG masters and the PNG export manifest."""

import json
from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACK = ROOT / "assets/branding"
INK = "#302830"
CORAL = "#FF8A60"
CREAM = "#FFF8EF"

# Shared geometry keeps the opposing links aligned across every variant.
LEFT = (
    "M108 14 C48 14 8 56 8 118 C8 164 29 194 68 220 "
    "L100 220 L162 146 H118 C98 146 81 132 81 111 "
    "C81 90 98 75 118 75 H202 L235 36 "
    "C238 32 238 30 234 26 L225 17 C222 14 220 14 215 14 Z"
)
RIGHT = (
    "M256 45 H281 C322 45 350 83 350 132 "
    "C350 195 307 244 247 244 H144 "
    "C141 244 139 243 137 241 L124 227 L173 169 H235 "
    "C258 169 275 155 275 135 C275 115 259 101 239 101 "
    "H195 L247 49 C250 46 253 45 256 45 Z"
)


def symbol(first=INK, second=CORAL):
    return f'<path fill="{first}" d="{LEFT}"/><path fill="{second}" d="{RIGHT}"/>'


def svg(content, title, width=360, height=260):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" role="img" aria-label="{escape(title)}">'
        f'<title>{escape(title)}</title>{content}</svg>\n'
    )


def tile(dark=False):
    background, foreground = (INK, CREAM) if dark else (CREAM, INK)
    border = "#493F47" if dark else "#E4D8CE"
    return (
        f'<rect x="8" y="8" width="496" height="496" rx="112" fill="{background}" stroke="{border}" stroke-width="1.5"/>'
        f'<g transform="translate(36 96) scale(1.23)">{symbol(foreground)}</g>'
    )


def wordmark(dark=False):
    foreground = CREAM if dark else INK
    return (
        f'<g transform="translate(4 39) scale(.7)">{symbol(foreground)}</g>'
        f'<text x="280" y="201" fill="{foreground}" font-family="Inter" '
        'font-size="190" font-weight="700" letter-spacing="-5">ABCOM</text>'
    )


def dock_tile():
    # The legacy ICNS body uses 412 of 512 units; Windows/Linux retain their own tile.
    scale = 412 / 496
    inset = (512 - 512 * scale) / 2
    return f'<g transform="translate({inset} {inset}) scale({scale})">{tile()}</g>'


def social_preview(english=True):
    title = "Encrypted LAN messaging" if english else "Messagerie LAN chiffrée"
    features = "Chats, groups and file sharing" if english else "Discussions, groupes et partage de fichiers"
    promise = "No server. No account." if english else "Sans serveur. Sans compte."
    content = (
        f'<rect width="1280" height="640" fill="{INK}"/>'
        f'<g transform="translate(80 198) scale(1.05)">{symbol(CREAM)}</g>'
        f'<g font-family="Inter" fill="{CREAM}">'
        '<text x="550" y="218" font-size="100" font-weight="700" letter-spacing="-3">Abcom</text>'
        f'<text x="554" y="284" font-size="36" font-weight="600">{title}</text>'
        f'<text x="554" y="338" font-size="25" fill="#D6CDD3">{features}</text>'
        f'<text x="554" y="383" font-size="25" fill="#FF8A60">{promise}</text>'
        '<path d="M554 428 H1200" stroke="#635660"/>'
        '<text x="554" y="476" font-size="23">macOS · Windows · Linux</text>'
        '<text x="554" y="566" font-size="20" fill="#D6CDD3">github.com/abnd-corp/Abcom</text>'
        '<text x="80" y="566" font-size="20" fill="#D6CDD3">Rust · AGPL-3.0</text>'
        '</g>'
    )
    return svg(content, f"Abcom — {title}", 1280, 640)


def preview(english=False):
    heading = "VISUAL IDENTITY" if english else "IDENTITÉ VISUELLE"
    pieces = [
        f'<rect width="1600" height="1060" fill="{CREAM}"/>',
        f'<g font-family="Inter" fill="{INK}">',
        '<rect x="48" y="42" width="14" height="56" rx="7" fill="#FF8A60"/>',
        '<text x="84" y="84" font-size="43" font-weight="700" letter-spacing="-1">'
        f'ABCOM · {heading}</text>',
    ]
    for x, name, color in [(1050, "Corail", CORAL), (1234, "Encre", INK), (1410, "Crème", CREAM)]:
        if english:
            name = {"Corail": "Coral", "Encre": "Ink", "Crème": "Cream"}[name]
        pieces.append(f'<rect x="{x}" y="48" width="36" height="36" rx="10" fill="{color}" stroke="#D4C9C2"/>')
        pieces.append(f'<text x="{x+46}" y="62" font-size="16" font-weight="600">{name}</text>')
        pieces.append(f'<text x="{x+46}" y="84" font-size="14">{color}</text>')
    labels = ("Application · light", "Application · dark") if english else ("Application · clair", "Application · sombre")
    for x, label, dark in [(58, labels[0], False), (584, labels[1], True), (1110, "macOS · Dock", False)]:
        pieces.append(f'<text x="{x}" y="171" font-size="28" font-weight="700">{label}</text>')
        artwork = dock_tile() if label == "macOS · Dock" else tile(dark)
        pieces.append(f'<g transform="translate({x} 208) scale(.84)">{artwork}</g>')
    for x in (550, 1076):
        pieces.append(f'<path d="M{x} 143 V635" stroke="#E9DED5" stroke-width="2"/>')
    for x, dark in [(58, False), (584, True)]:
        bg, fg = (INK, CREAM) if dark else (CREAM, INK)
        label = ("Menu · dark" if dark else "Menu · light") if english else ("Menu · sombre" if dark else "Menu · clair")
        pieces.append(f'<text x="{x}" y="715" font-size="28" font-weight="700">{label}</text>')
        pieces.append(f'<rect x="{x}" y="748" width="442" height="78" rx="16" fill="{bg}" stroke="#D8CCC3"/>')
        pieces.append(f'<g transform="translate({x+28} 775)" fill="none" stroke="{fg}" stroke-width="3" stroke-linecap="round">'
                      '<path d="M0 6 Q15 -7 30 6 M6 12 Q15 4 24 12 M12 19 Q15 16 18 19"/></g>')
        pieces.append(f'<path d="M{x+92} 781 h8 l10 -8 v28 l-10 -8 h-8 Z" fill="{fg}"/>')
        pieces.append(f'<path d="M{x+118} 779 q9 8 0 16" fill="none" stroke="{fg}" stroke-width="3" stroke-linecap="round"/>')
        pieces.append(f'<g transform="translate({x+171} 769) scale(.14)">{symbol(fg, fg)}</g>')
        pieces.append(f'<rect x="{x+253}" y="779" width="30" height="18" rx="3" fill="none" stroke="{fg}" stroke-width="2"/>')
        pieces.append(f'<rect x="{x+257}" y="783" width="21" height="10" rx="1" fill="{fg}"/>')
        day = "Mon" if english else "Lun."
        pieces.append(f'<text x="{x+310}" y="796" font-size="21" fill="{fg}">{day} 10:24</text>')
    actual = "Actual sizes · 1×" if english else "Tailles réelles · 1×"
    pieces.append(f'<text x="1110" y="715" font-size="28" font-weight="700">{actual}</text>')
    for x, size in [(1150, 16), (1288, 24), (1430, 32)]:
        scale = size / 360
        pieces.append(f'<g transform="translate({x} {782-size*260/720}) scale({scale})">{symbol()}</g>')
        pieces.append(f'<text x="{x-10}" y="829" font-size="18">{size} px</text>')
    pieces.append('<path d="M58 881 H1542" stroke="#E9DED5" stroke-width="2"/>')
    pieces.append(f'<g transform="translate(58 905) scale(.43)">{wordmark()}</g>')
    footer = "Vector SVG · transparent PNG · light / dark" if english else "SVG vectoriels · PNG transparents · clair / sombre"
    details = "Symbol, wordmark, application icons and tray 16 / 24 / 32 px + Retina" if english else "Symbole, logotype, icônes d’application et tray 16 / 24 / 32 px + Retina"
    pieces.append(f'<text x="690" y="947" font-size="19" font-weight="600">{footer}</text>')
    pieces.append(f'<text x="690" y="979" font-size="17">{details}</text>')
    pieces.append('</g>')
    title = "Abcom — visual asset overview" if english else "Abcom — aperçu des déclinaisons graphiques"
    return svg(''.join(pieces), title, 1600, 1060)


def main():
    directory = PACK / "svg"
    directory.mkdir(parents=True, exist_ok=True)
    variants = {
        "abcom-symbol": (symbol(), 360, 260),
        "abcom-symbol-on-dark": (symbol(CREAM), 360, 260),
        "abcom-symbol-ink": (symbol(INK, INK), 360, 260),
        "abcom-symbol-cream": (symbol(CREAM, CREAM), 360, 260),
        "abcom-symbol-coral": (symbol(CORAL, CORAL), 360, 260),
        "abcom-symbol-outline": (f'<g fill="none" stroke-width="5" stroke-linejoin="round"><path stroke="{INK}" d="{LEFT}"/><path stroke="{CORAL}" d="{RIGHT}"/></g>', 360, 260),
        "abcom-app-light": (tile(), 512, 512),
        "abcom-app-dark": (tile(True), 512, 512),
        "abcom-dock": (dock_tile(), 512, 512),
        "abcom-tray-light": (symbol(INK, INK), 360, 260),
        "abcom-tray-dark": (symbol(CREAM, CREAM), 360, 260),
        "abcom-tray-template": (symbol("#000000", "#000000"), 360, 260),
        "abcom-wordmark": (wordmark(), 1080, 260),
        "abcom-wordmark-on-dark": (wordmark(True), 1080, 260),
    }
    exports = []
    for name, (content, width, height) in variants.items():
        (directory / f"{name}.svg").write_text(svg(content, name, width, height), encoding="utf-8")
        if "wordmark" in name:
            sizes = [540, 1080, 2160]
        elif "tray" in name:
            sizes = [16, 18, 20, 24, 32, 36, 40, 48, 64]
        elif "app" in name or "dock" in name:
            sizes = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 128, 256, 512, 1024]
        else:
            sizes = [32, 64, 128, 256, 512, 1024]
        for size in sizes:
            png_height = round(size * height / width) if "wordmark" in name else size
            exports.append({"source": f"svg/{name}.svg", "output": f"png/{name}-{size}.png", "width": size, "height": png_height, "template": name.endswith("template")})
    for name, english in [("preview", False), ("preview-en", True)]:
        (PACK / f"{name}.svg").write_text(preview(english), encoding="utf-8")
        exports.append({"source": f"{name}.svg", "output": f"{name}.png", "width": 1600, "height": 1060})
    for name, english in [("social-preview", True), ("social-preview-fr", False)]:
        (PACK / f"{name}.svg").write_text(social_preview(english), encoding="utf-8")
        exports.append({"source": f"{name}.svg", "output": f"{name}.png", "width": 1280, "height": 640})
    (PACK / "exports.json").write_text(json.dumps(exports, indent=2) + "\n", encoding="utf-8")
    (PACK / "palette.json").write_text(json.dumps({"coral": CORAL, "ink": INK, "cream": CREAM}, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
