"""Build the static product site into dist/site.

Renders site/template.html and site/404.html once per language in site/i18n/, and beside it that
language's animated manual (site/manual, scripts/manual_i18n.py) with the same header
and footer (site/partials). Copies site/assets and the localized README hero images,
and writes CNAME, robots.txt and sitemap.xml for GitHub Pages. No dependencies beyond
the standard library.
"""
import argparse
import datetime
import hashlib
import html
import json
import re
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manual_i18n  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
FEATURES = ["groups", "peek", "tabs", "rules", "portal", "hide"]
DETAILS = ["glass", "files", "space", "back", "footprint", "safe"]
FEATURE_ICONS = ["grid", "cursor", "layers", "spark", "folder", "expand"]
DETAIL_ICONS = ["spark", "cursor", "expand", "restore", "feather", "shield"]
# The manual lesson each feature's animation and step-by-step link come from.
FEATURE_LESSONS = {"groups": "create", "peek": "peek", "tabs": "tabs", "rules": "rules", "portal": "portal",
                   "hide": "hide"}
STORE = "https://apps.microsoft.com/detail/9MV6WG3XNWSX"
PLACEHOLDER = re.compile(r"\{\{(t|raw):([\w.]+)\}\}|\{\{(\w+)\}\}")
OG_LOCALES = {
    "en": "en_US", "zh-CN": "zh_CN", "zh-TW": "zh_TW", "ja": "ja_JP", "ko": "ko_KR",
    "de": "de_DE", "fr": "fr_FR", "es": "es_ES", "pt-BR": "pt_BR", "ru": "ru_RU",
}


def render(template, values, strings):
    def replace(match):
        kind, key, simple = match.groups()
        if simple:
            return values[simple]
        text = strings[key]
        return text if kind == "raw" else html.escape(text, quote=True)
    return PLACEHOLDER.sub(replace, template)


def feature_fences(strings, root):
    """One card per feature: the recording, replaced by site.js with the manual's animation of
    the feature (manual/?only=<lesson>&embed), and a link to the lesson's steps."""
    parts = []
    guide = html.escape(strings["features.guide"])
    for name, icon in zip(FEATURES, FEATURE_ICONS):
        title = html.escape(strings[f"feature.{name}.title"])
        lesson = FEATURE_LESSONS[name]
        parts.append(f'''      <article id="feature-{name}" class="feature-card clip feature-{name}">
        <div class="feature-copy">
          <div class="feature-heading">
            <span class="feature-icon"><svg class="icon" aria-hidden="true"><use href="#i-{icon}"/></svg></span>
            <h3>{title}</h3>
          </div>
          <div class="feature-text">
            <p>{strings[f"feature.{name}.text"]}</p>
            <a class="text-link" href="manual/#{lesson}">{guide}<svg class="icon" aria-hidden="true"><use href="#i-arrow"/></svg></a>
          </div>
        </div>
        <div class="feature-media">
          <video aria-label="{title}" controls muted loop playsinline preload="none" poster="{root}assets/{name}.jpg" width="1290" height="726">
            <source src="{root}assets/{name}.mp4" type="video/mp4">
            <img class="poster" src="{root}assets/{name}.jpg" alt="" width="1290" height="726">
          </video>
          <iframe class="feature-demo" title="{title}" data-src="manual/?only={lesson}&amp;embed" loading="lazy" hidden></iframe>
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
    header = (SITE / "partials/header.html").read_text(encoding="utf-8")
    footer = (SITE / "partials/footer.html").read_text(encoding="utf-8")
    not_found = (SITE / "404.html").read_text(encoding="utf-8")
    # Content hash appended to the stylesheet and script URLs so browsers pick up new
    # versions immediately despite the CDN's cache lifetime.
    asset_version = hashlib.sha256(
        (SITE / "assets/base.css").read_bytes() + (SITE / "assets/site.css").read_bytes()
        + (SITE / "assets/site.js").read_bytes() + (SITE / "assets/mark.svg").read_bytes()
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
    # JSON Schemas referenced from config files (`$schema`); see docs/RELEASING.md.
    if (SITE / "schema").is_dir():
        shutil.copytree(SITE / "schema", out / "schema")
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

        def language_options(href):
            return "\n".join(
                f'        <option value="{lang["code"]}" data-href="{href(lang)}"{" selected" if lang is language else ""}>'
                f'{html.escape(lang["name"])}</option>'
                for lang in languages
            )
        canonical = f"{origin}/{directory}"
        hero_image = f"{origin}/assets/hero-{code}.png?v={hero_version}"
        # schema.org data for search engines; "</" is escaped so the JSON cannot close the script tag.
        structured_data = json.dumps({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "PecoFence",
            "description": strings["meta.description"],
            "url": canonical,
            "image": hero_image,
            "inLanguage": code,
            "applicationCategory": "UtilitiesApplication",
            "operatingSystem": "Windows 11",
            "isAccessibleForFree": True,
            "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
            "license": "https://www.apache.org/licenses/LICENSE-2.0",
            "downloadUrl": STORE,
            "sameAs": [repository, STORE],
        }, ensure_ascii=False).replace("</", "<\\/")
        values = {
            "lang": code,
            "root": root,
            "origin": origin,
            "canonical": canonical,
            "hero_image": hero_image,
            "structured_data": structured_data,
            "alternates": alternates,
            "og_locale": OG_LOCALES.get(code, code.replace("-", "_")),
            "repository": repository,
            "releases": f"{repository}/releases/latest",
            "docs": docs,
            "features": feature_fences(strings, root),
            "feature_tabs": feature_tabs(strings),
            "details": detail_items(strings),
            "year": str(datetime.date.today().year),
            "v": asset_version,
        }
        # Header and footer as on the home page (links relative to it) ...
        chrome = {"home": "", "manual": "manual/", "manual_current": "",
                  "language_options": language_options(lambda lang: f"{root}{lang['dir']}")}
        values["header"] = render(header, {**values, **chrome}, strings)
        values["footer"] = render(footer, {**values, **chrome}, strings)
        page = render(template, values, strings)
        target = out / directory / "index.html"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(page, encoding="utf-8", newline="\n")
        urls.append(f"{origin}/{directory}")

        # The 404 page Cloudflare Pages shows for a missing path under <dir>. It can be
        # served at any depth, so its links start at the site root.
        chrome = {"root": "/", "home": f"/{directory}", "manual": f"/{directory}manual/", "manual_current": "",
                  "language_options": language_options(lambda lang: f"/{lang['dir']}")}
        parts = {"header": render(header, {**values, **chrome}, strings),
                 "footer": render(footer, {**values, **chrome}, strings)}
        (out / directory / "404.html").write_text(
            render(not_found, {**values, **chrome, **parts}, strings), encoding="utf-8", newline="\n")

        # ... and on the manual at <dir>manual/, one level down.
        up = "../" + root
        chrome = {"root": up, "home": "../", "manual": "./", "manual_current": ' aria-current="page"',
                  "language_options": language_options(lambda lang: f"{up}{lang['dir']}manual/")}
        parts = {
            "head": "\n".join(
                [f'<link rel="canonical" href="{origin}/{directory}manual/">']
                + [f'<link rel="alternate" hreflang="{lang["code"]}" href="{origin}/{lang["dir"]}manual/">' for lang in languages]
                + [f'<link rel="alternate" hreflang="x-default" href="{origin}/manual/">',
                   f'<link rel="stylesheet" href="{up}assets/base.css?v={asset_version}">',
                   f'<script src="{up}assets/site.js?v={asset_version}" defer></script>']),
            "header": render(header, {**values, **chrome}, strings),
            "footer": render(footer, {**values, **chrome}, strings),
        }
        if code == "zh-CN":
            data = {"page": {}, "stage": {}}
        else:
            data = manual_i18n.load(code)
            page_keys, stage = manual_i18n.source_strings()
            missing = (sum(not data["page"].get(k) for k in page_keys) + sum(not data["stage"].get(k) for k in stage)
                       + sum(not data.get("keys", {}).get(k) for k in manual_i18n.key_names()))
            if missing:
                problems.append(f"manual/i18n/{code}.json: {missing} untranslated strings (shown in Chinese)")
        manual_i18n.build(out / directory / "manual", code, data, manual_i18n.fill(parts))
        urls.append(f"{origin}/{directory}manual/")

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
