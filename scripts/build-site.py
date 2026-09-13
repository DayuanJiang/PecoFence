"""Build the static product site into dist/site.

Renders site/template.html once per language in site/i18n/, copies site/assets and
the localized README hero images, and writes CNAME, robots.txt and sitemap.xml for
GitHub Pages. No dependencies beyond the standard library.
"""
import argparse
import datetime
import html
import json
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
FEATURES = ["groups", "portal", "tabs", "peek", "hide", "rules"]
DETAILS = ["glass", "files", "space", "back", "footprint", "safe"]
PLACEHOLDER = re.compile(r"\{\{(t|raw):([\w.]+)\}\}|\{\{(\w+)\}\}")
OG_LOCALES = {
    "en": "en_US", "zh-CN": "zh_CN", "zh-TW": "zh_TW", "ja": "ja_JP", "ko": "ko_KR",
    "de": "de_DE", "fr": "fr_FR", "es": "es_ES", "pt-BR": "pt_BR", "ru": "ru_RU",
}

PLAY = ('<svg aria-hidden="true" class="icon-play" width="12" height="12" viewBox="0 0 12 12">'
        '<path d="M3 1.5v9l7-4.5z" fill="currentColor"/></svg>')
PAUSE = ('<svg aria-hidden="true" class="icon-pause" width="12" height="12" viewBox="0 0 12 12">'
         '<path d="M2.5 1.5h2.5v9H2.5zM7 1.5h2.5v9H7z" fill="currentColor"/></svg>')


def render(template, values, strings):
    def replace(match):
        kind, key, simple = match.groups()
        if simple:
            return values[simple]
        text = strings[key]
        return text if kind == "raw" else html.escape(text, quote=True)
    return PLACEHOLDER.sub(replace, template)


def feature_fences(strings, root):
    parts = []
    for name in FEATURES:
        title = html.escape(strings[f"feature.{name}.title"])
        parts.append(f'''      <div class="fence clip">
        <div class="fence-title"><span>{title}</span>
          <button class="clip-toggle" type="button" data-play="{html.escape(strings["features.play"])}" data-pause="{html.escape(strings["features.pause"])}">{PLAY}{PAUSE}</button>
        </div>
        <video muted loop playsinline preload="none" poster="{root}assets/{name}.jpg" width="1290" height="495">
          <source src="{root}assets/{name}.mp4" type="video/mp4">
          <img class="poster" src="{root}assets/{name}.jpg" alt="" width="1290" height="495">
        </video>
        <div class="fence-body"><p>{strings[f"feature.{name}.text"]}</p></div>
      </div>''')
    return "\n".join(parts)


def detail_items(strings):
    return "\n".join(
        f'        <div><dt>{html.escape(strings[f"detail.{name}.title"])}</dt>'
        f'<dd>{html.escape(strings[f"detail.{name}.text"])}</dd></div>'
        for name in DETAILS
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default=str(ROOT / "dist/site"))
    parser.add_argument("--base", default=None,
                        help="absolute origin for canonical URLs (default: https://<domain>)")
    parser.add_argument("--strict", action="store_true",
                        help="fail when a language is missing or its keys differ from en.json")
    args = parser.parse_args()
    problems = []
    out = Path(args.out).resolve()
    config = json.loads((SITE / "site.json").read_text(encoding="utf-8"))
    origin = (args.base or f"https://{config['domain']}").rstrip("/")
    repository = config["repository"].rstrip("/")
    docs = f"{repository}/blob/main/docs"
    template = (SITE / "template.html").read_text(encoding="utf-8")
    languages = config["languages"]
    english = json.loads((SITE / "i18n/en.json").read_text(encoding="utf-8"))

    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(SITE / "assets", out / "assets")
    for language in languages:
        hero = ROOT / "docs/assets" / f"hero-{language['code']}.png"
        if hero.exists():
            shutil.copy2(hero, out / "assets" / hero.name)

    alternates = "\n".join(
        f'<link rel="alternate" hreflang="{lang["code"]}" href="{origin}/{lang["dir"]}">' for lang in languages
    ) + f'\n<link rel="alternate" hreflang="x-default" href="{origin}/">'

    urls = []
    for language in languages:
        code, directory = language["code"], language["dir"]
        path = SITE / "i18n" / f"{code}.json"
        strings = dict(english)
        if path.exists():
            translated = json.loads(path.read_text(encoding="utf-8"))
            missing = sorted(set(english) - set(translated))
            extra = sorted(set(translated) - set(english))
            if missing or extra:
                problems.append(f"{path.name}: missing {missing or 'none'}, unexpected {extra or 'none'}")
            strings.update({key: value for key, value in translated.items() if key in english})
        else:
            problems.append(f"no strings for {code}, using English")
        root = "../" if directory else ""
        readme = f"{repository}/blob/main/README.md" if code == "en" else \
            f"{repository}/blob/main/docs/readme/README.{code}.md"
        language_links = "\n".join(
            f'          <li><strong lang="{lang["code"]}">{html.escape(lang["name"])}</strong></li>' if lang is language else
            f'          <li><a lang="{lang["code"]}" href="{root}{lang["dir"]}">{html.escape(lang["name"])}</a></li>'
            for lang in languages
        )
        language_options = "\n".join(
            f'        <option value="{lang["code"]}" data-href="{root}{lang["dir"]}"{" selected" if lang is language else ""}>'
            f'{html.escape(lang["name"])}</option>'
            for lang in languages
        )
        values = {
            "lang": code,
            "root": root,
            "origin": origin,
            "canonical": f"{origin}/{directory}",
            "alternates": alternates,
            "og_locale": OG_LOCALES.get(code, code.replace("-", "_")),
            "repository": repository,
            "releases": f"{repository}/releases/latest",
            "docs": docs,
            "readme": readme,
            "features": feature_fences(strings, root),
            "details": detail_items(strings),
            "language_links": language_links,
            "language_options": language_options,
            "year": str(datetime.date.today().year),
        }
        page = render(template, values, strings)
        target = out / directory / "index.html"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(page, encoding="utf-8", newline="\n")
        urls.append(f"{origin}/{directory}")

    (out / "CNAME").write_text(config["domain"] + "\n", encoding="utf-8")
    (out / ".nojekyll").write_text("", encoding="utf-8")
    (out / "robots.txt").write_text(f"User-agent: *\nAllow: /\nSitemap: {origin}/sitemap.xml\n", encoding="utf-8")
    (out / "sitemap.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        + "".join(f"  <url><loc>{url}</loc></url>\n" for url in urls) + "</urlset>\n", encoding="utf-8")
    total = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
    print(f"built {len(urls)} pages into {out} ({total / 1024 / 1024:.1f} MiB)")
    if "YOUR-ACCOUNT" in repository:
        problems.append("site/site.json still has the placeholder repository URL")
    for problem in problems:
        print(f"warning: {problem}")
    if problems and args.strict:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
