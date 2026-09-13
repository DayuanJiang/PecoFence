"""Check that every README translation mirrors the English README.

Verifies that each language has a README under docs/readme/, a hero image under
docs/assets/, a ten-entry language bar, the same number of headings, tables,
images and <details> blocks as README.md, and that every relative link and
in-page anchor resolves. Exit status is non-zero on any problem.
"""
import re
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README_DIR = ROOT / "docs/readme"
LANGUAGES = ["zh-CN", "zh-TW", "ja", "ko", "de", "fr", "es", "pt-BR", "ru"]
LINK = re.compile(r'\]\(([^)\s]+)\)|href="([^"]+)"|src="([^"]+)"')
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*$", re.M)


def anchor(text):
    text = re.sub(r"[*_`]", "", text).strip().lower()
    text = "".join(c for c in text if c.isalnum() or c in " -_" or unicodedata.category(c).startswith("M"))
    return text.replace(" ", "-")


def shape(text):
    return {
        "headings": len(HEADING.findall(text)),
        "tables": text.count("| :---"),
        "details": text.count("<details>"),
        "images": text.count("![") + text.count("<img "),
        "code": text.count("```") // 2,
    }


def check(path, expected_shape, hero, problems):
    text = path.read_text(encoding="utf-8")
    if text.startswith("﻿") or "\r" in text:
        problems.append(f"{path.name}: BOM or CRLF line endings")
    if hero and f"hero-{hero}.png" not in text:
        problems.append(f"{path.name}: does not reference hero-{hero}.png")
    if text.count("&nbsp;·&nbsp;") < 11:
        problems.append(f"{path.name}: language bar seems incomplete")
    actual = shape(text)
    if expected_shape and actual != expected_shape:
        problems.append(f"{path.name}: structure {actual} differs from README.md {expected_shape}")
    anchors = {anchor(h) for _, h in HEADING.findall(text)}
    for match in LINK.finditer(text):
        target = next(group for group in match.groups() if group)
        if target.startswith(("http://", "https://", "mailto:")):
            continue
        if target.startswith("#"):
            if target[1:] not in anchors:
                problems.append(f"{path.name}: anchor {target} has no heading")
            continue
        file, _, fragment = target.partition("#")
        resolved = (path.parent / file).resolve()
        if not resolved.exists():
            problems.append(f"{path.name}: missing link target {target}")
        elif fragment and resolved.suffix == ".md":
            other = {anchor(h) for _, h in HEADING.findall(resolved.read_text(encoding="utf-8"))}
            if fragment not in other:
                problems.append(f"{path.name}: {target} anchor not found")
    return actual


def main():
    problems = []
    english = ROOT / "README.md"
    expected = check(english, None, "en", problems)
    for language in LANGUAGES:
        path = README_DIR / f"README.{language}.md"
        if not path.exists():
            problems.append(f"missing docs/readme/README.{language}.md")
            continue
        check(path, expected, language, problems)
        if not (ROOT / "docs/assets" / f"hero-{language}.png").exists():
            problems.append(f"missing docs/assets/hero-{language}.png")
    for problem in problems:
        print(problem)
    print(f"{len(LANGUAGES) + 1} READMEs checked, {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
