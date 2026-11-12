#!/usr/bin/env python3
# check_docs.py — Check documentation language pairs, local links and test IDs.
"""Standard-library-only checks for the English and French documentation."""

from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parent.parent
PAGES = (
    ("01-presentation.md", "01-overview.md"),
    ("02-architecture.md", "02-architecture.md"),
    ("03-reseau-et-securite.md", "03-network-and-security.md"),
    ("04-stockage.md", "04-storage.md"),
    ("05-fonctionnalites.md", "05-features.md"),
    ("06-installation.md", "06-installation.md"),
    ("07-developpement.md", "07-development.md"),
    ("08-historique-et-audits.md", "08-history-and-audits.md"),
    ("09-limites-et-pistes.md", "09-limitations.md"),
    ("10-cahier-de-tests.md", "10-testing.md"),
    ("11-depannage.md", "11-troubleshooting.md"),
    ("12-performances.md", "12-performance.md"),
)
LINK = re.compile(r"!?\[[^\]\n]*\]\(([^\s)]+)\)")
CASE = re.compile(r"^\|\s*((?:\d+\.\d+)|(?:[LMW]\d+))\s*\|", re.M)


def prose(text):
    return re.sub(r"^(`{3,}|~{3,}).*?^\1[^\n]*$", "", text, flags=re.M | re.S)


def anchors(text):
    found = set()
    counts = {}
    for title in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", prose(text), re.M):
        title = re.sub(r"\[([^]]+)\]\([^)]+\)", r"\1", title)
        slug = re.sub(r"[^\w\- ]", "", title.lower()).replace(" ", "-")
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        found.add(slug if count == 0 else f"{slug}-{count}")
    return found


def check(root=ROOT):
    errors = []
    pairs = [(root / "README.fr.md", root / "README.md"),
             (root / "docs/README.fr.md", root / "docs/README.md"),
             (root / "packaging/README.fr.md", root / "packaging/README.md")]
    pairs += [(root / "docs" / fr, root / "docs/en" / en) for fr, en in PAGES]
    files = [path for pair in pairs for path in pair]
    guides = set((root / "docs").glob("[0-9][0-9]-*.md"))
    guides.update((root / "docs/en").glob("*.md"))
    for path in sorted(guides - set(files)):
        errors.append(f"Guide missing from language pairs: {path.relative_to(root)}")
    for pair in pairs:
        for path, other in (pair, pair[::-1]):
            if not path.is_file():
                errors.append(f"Missing translation: {path.relative_to(root)}")
                continue
            text = path.read_text(encoding="utf-8")
            destinations = [urlsplit(link) for link in LINK.findall(prose(text))]
            if not any(not link.scheme and not link.netloc and
                       (path.parent / unquote(link.path)).resolve() == other.resolve()
                       for link in destinations):
                errors.append(f"Missing language link: {path.relative_to(root)}")
    for path in files:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        if path.parent == root and len(text.splitlines()) > 60:
            errors.append(f"README exceeds 60 lines: {path.name}")
        for raw in LINK.findall(prose(text)):
            link = urlsplit(raw)
            if link.scheme or link.netloc:
                continue
            target = (path.parent / unquote(link.path)).resolve() if link.path else path
            if not target.exists():
                errors.append(f"Broken link: {path.relative_to(root)} → {raw}")
            elif link.fragment and target.suffix == ".md":
                if unquote(link.fragment) not in anchors(target.read_text(encoding="utf-8")):
                    errors.append(f"Broken anchor: {path.relative_to(root)} → {raw}")
    case_sets = []
    for path in (root / "docs/10-cahier-de-tests.md", root / "docs/en/10-testing.md"):
        if path.is_file():
            ids = CASE.findall(prose(path.read_text(encoding="utf-8")))
            if len(ids) != len(set(ids)):
                errors.append(f"Duplicate test IDs: {path.relative_to(root)}")
            case_sets.append(set(ids))
    if len(case_sets) == 2 and case_sets[0] != case_sets[1]:
        errors.append(f"Manual test IDs differ: {sorted(case_sets[0] ^ case_sets[1])}")
    return errors, len(pairs), len(files)


def main():
    errors, pair_count, file_count = check()
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Documentation OK: {pair_count} EN/FR pairs, {file_count} pages; links, anchors and test IDs checked")
    return 0


if __name__ == "__main__":
    sys.exit(main())
