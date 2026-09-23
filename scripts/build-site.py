"""Build the static product site into dist/site.

Renders site/template.html once per language in site/i18n/, copies site/assets and
the localized README hero images, and writes CNAME, robots.txt and sitemap.xml for
GitHub Pages. No dependencies beyond the standard library.
"""
import argparse
import datetime
import hashlib
import html
import json
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
FEATURES = ["groups", "peek", "tabs", "rules", "portal", "hide"]
DETAILS = ["glass", "files", "space", "back", "footprint", "safe"]
FEATURE_ICONS = ["grid", "cursor", "layers", "spark", "folder", "expand"]
DETAIL_ICONS = ["spark", "cursor", "expand", "restore", "feather", "shield"]
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
    for name, icon in zip(FEATURES, FEATURE_ICONS):
        title = html.escape(strings[f"feature.{name}.title"])
        play = html.escape(strings["features.play"], quote=True)
        pause = html.escape(strings["features.pause"], quote=True)
        parts.append(f'''      <article id="feature-{name}" class="feature-card clip feature-{name}">
        <div class="feature-copy">
          <div class="feature-heading">
            <span class="feature-icon"><svg class="icon" aria-hidden="true"><use href="#i-{icon}"/></svg></span>
            <h3>{title}</h3>
          </div>
          <p>{strings[f"feature.{name}.text"]}</p>
        </div>
        <div class="feature-media">
          <video aria-label="{title}" controls muted loop playsinline preload="none" poster="{root}assets/{name}.jpg" width="1290" height="726">
            <source src="{root}assets/{name}.mp4" type="video/mp4">
            <img class="poster" src="{root}assets/{name}.jpg" alt="" width="1290" height="726">
          </video>
          <button class="clip-toggle" type="button" aria-label="{play}: {title}" data-play="{play}" data-pause="{pause}" data-title="{title}" hidden>{PLAY}{PAUSE}</button>
        </div>
      </article>''')
    return "\n".join(parts)


def feature_tabs(strings):
    return "\n".join(
        f'<button id="tab-{name}" type="button" role="tab" aria-controls="feature-{name}" '
        f'aria-selected="{str(index == 0).lower()}" tabindex="{0 if index == 0 else -1}">'
        f'<svg class="icon" aria-hidden="true"><use href="#i-{icon}"/></svg>'
        f'<span>{html.escape(strings[f"feature.{name}.title"])}</span></button>'
        for index, (name, icon) in enumerate(zip(FEATURES, FEATURE_ICONS))
    )


def detail_items(strings):
    return "\n".join(
        f'        <div><dt><span class="detail-icon"><svg class="icon" aria-hidden="true">'
        f'<use href="#i-{icon}"/></svg></span><br>{html.escape(strings[f"detail.{name}.title"])}</dt>'
        f'<dd>{html.escape(strings[f"detail.{name}.text"])}</dd></div>'
        for name, icon in zip(DETAILS, DETAIL_ICONS)
    )


def language_items(languages, current, root):
    parts = []
    for language in languages:
        href = root + (language["dir"] or "./")
        active = ' aria-current="page"' if language is current else ""
        parts.append(
            f'          <li lang="{language["code"]}"><a href="{href}" '
            f'hreflang="{language["code"]}"{active}>{html.escape(language["name"])}</a></li>'
        )
    return "\n".join(parts)


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
    # Content hash appended to the stylesheet and script URLs so browsers pick up new
    # versions immediately despite the CDN's cache lifetime.
    asset_version = hashlib.sha256(
        (SITE / "assets/site.css").read_bytes() + (SITE / "assets/site.js").read_bytes()
        + (SITE / "assets/mark.svg").read_bytes()
    ).hexdigest()[:10]
    languages = config["languages"]
    english = json.loads((SITE / "i18n/en.json").read_text(encoding="utf-8"))

    if out.exists():
        # --out is user supplied; only replace a generated site within the workspace.
        if not out.is_relative_to(ROOT) or out == ROOT or out == SITE or SITE in out.parents:
            raise SystemExit(f"Refusing to replace a directory outside the build workspace: {out}")
        if not (out / "index.html").exists():
            raise SystemExit(f"Refusing to replace a directory without a generated index.html: {out}")
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
        hero = ROOT / "docs/assets" / f"hero-{code}.png"
        if not hero.is_file():
            raise SystemExit(f"Missing localized share image: {hero}")
        hero_version = hashlib.sha256(hero.read_bytes()).hexdigest()[:10]
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
        language_links = language_items(languages, language, root)
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
            "hero_image": f"{origin}/assets/hero-{code}.png?v={hero_version}",
            "alternates": alternates,
            "og_locale": OG_LOCALES.get(code, code.replace("-", "_")),
            "repository": repository,
            "releases": f"{repository}/releases/latest",
            "docs": docs,
            "readme": readme,
            "features": feature_fences(strings, root),
            "feature_tabs": feature_tabs(strings),
            "details": detail_items(strings),
            "language_links": language_links,
            "language_options": language_options,
            "year": str(datetime.date.today().year),
            "v": asset_version,
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
