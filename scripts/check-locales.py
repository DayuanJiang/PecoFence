"""Check embedded translations, source coverage, and interpolation placeholders."""
import ast
import json
import re
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = ("en", "ja", "zh-TW", "ko", "de", "fr", "es", "pt-BR", "ru")
CJK = re.compile(r"[\u3400-\u9fff]")
LITERAL = r'"(?:\\.|[^"\\])*"'


class StaticMessages(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.keys = set()
        self.stack = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        skip = tag in ("script", "style") or "data-language-name" in attrs
        if tag not in ("meta", "link", "input", "br", "img", "hr"):
            self.stack.append((tag, skip))
        for name in ("title", "placeholder", "aria-label"):
            value = attrs.get(name, "")
            if CJK.search(value):
                self.keys.add(value)

    def handle_endtag(self, tag):
        for index in range(len(self.stack) - 1, -1, -1):
            if self.stack[index][0] == tag:
                self.stack = self.stack[:index]
                break

    def handle_data(self, data):
        if not any(skip for _, skip in self.stack):
            value = data.strip()
            if CJK.search(value):
                self.keys.add(value)


def source_keys():
    html = (ROOT / "ui/settings.html").read_text(encoding="utf-8")
    static = StaticMessages()
    static.feed(html)
    keys = static.keys
    for match in re.finditer(r"\b(?:t|tf)\(\s*(" + LITERAL + ")", html):
        keys.add(ast.literal_eval(match[1]))
    for path in (ROOT / "crates").rglob("*.rs"):
        if path.name in ("bindings.rs", "gpu_bindings.rs"):
            continue
        for match in re.finditer(r"i18n::(?:text|format)\(\s*(" + LITERAL + ")", path.read_text(encoding="utf-8")):
            keys.add(ast.literal_eval(match[1]))
    # The native palette stores constant source names and translates at serialization.
    keys.update(("红", "橙", "黄", "绿", "青", "蓝", "紫", "粉", "灰"))
    return keys


def placeholders(value):
    return sorted(re.findall(r"\{\d+\}|%1", value))


def check():
    keys = source_keys()
    failures = []
    for language in LANGUAGES:
        path = ROOT / "locales" / (language + ".json")
        catalog = json.loads(path.read_text(encoding="utf-8"))
        for key in sorted(keys - catalog.keys()):
            failures.append(f"{language}: missing {key!r}")
        for key, value in catalog.items():
            if not isinstance(value, str) or not value.strip():
                failures.append(f"{language}: empty/invalid {key!r}")
            elif placeholders(key) != placeholders(value):
                failures.append(f"{language}: placeholders differ for {key!r}")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"OK: {len(keys)} source messages covered in all {len(LANGUAGES) + 1} languages")


if __name__ == "__main__":
    check()
