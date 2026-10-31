# Product website

The product page at <https://pecofence.jiang.jp> is a static site generated from
`site/` and published with Cloudflare Pages. It needs no Node toolchain: the build is
`scripts/build-site.py` and the standard library.

## Layout

| Path | Purpose |
|---|---|
| `site/template.html` | One HTML template rendered once per language |
| `site/404.html` | The "page not found" page, rendered once per language |
| `site/partials/header.html`, `footer.html` | The header (manual and GitHub links, language picker, download button) and footer, shared by the home page and the manual |
| `site/assets/base.css` | Tokens, header and footer styles, shared by the home page and the manual |
| `site/assets/site.css`, `site.js`, `mark.svg` | The home page's styles; the hero trailer's play control, accessible feature tabs and their animations, AI prompt and install command copy buttons, language picker (`site.js` runs on the manual too) and favicon |
| `site/assets/*.mp4`, `*.jpg`, `panel-*.png`, `wallpaper.jpg` | The 30-second trailer (`promo.mp4`, v3; `--promo-only` re-exports just it and its poster), six feature clips, posters, the three hero fences and the wallpaper, exported by `scripts/make-site-media.py` from the local promo project |
| `site/assets/showcase-wallpaper.jpg` | The hero's backdrop: the original wallpaper from the revision-2 Store scene |
| `site/i18n/<language>.json` | Copy for each language; `en.json` is the source and every other file must have the same keys |
| `site/site.json` | Domain, repository URL and the language list |
| `site/manual/` | The animated user manual (see [The user manual](#the-user-manual)) |

The build writes `dist/site/`: `index.html` for English, one `<language>/index.html`
per translation, a `404.html` beside each of them, the localized README hero images as Open
Graph and Twitter previews, `CNAME`, `robots.txt` and `sitemap.xml`. Cloudflare Pages answers a
missing path with status 404 and the nearest `404.html` above it, so `/ja/old-page` gets the
Japanese one; without a top-level `404.html` it would serve the home page instead. The 404
pages can show at any depth, so their links start at the site root. Pages carry `hreflang` alternates, so search engines
send visitors to their language; the header's language picker and the language links
do the same by hand. The picker preserves the current section. The header links to
the manual and GitHub; at 840 px and narrower only the manual link stays (GitHub and
the docs are in the footer), and at 480 px the download button leaves the header to
the hero's buttons.

The page pairs a warm paper-and-lavender hero with a light reading canvas. The
30-second trailer is the hero's visual and the largest element on the page
(`#watch`): on screens at least 861 px tall the headline and primary Store download
link sit in a row above it, and the film's width follows the viewport height so all
of it stays above the fold; on shorter laptop screens it sits beside the copy; on
tablets and phones it comes straight after the headline. It shows its poster
(`promo.jpg`, preloaded) with a large play control and never starts on its own: a
click plays it inline with sound and hands over to the native controls. Without
JavaScript the native controls are shown from the start. The portable download is a
secondary text link.

The hero also links directly to AI configuration through its CLI badge and a secondary action.
The AI + CLI section follows the feature gallery. It presents settings,
organization rules and configuration backup as everyday uses, alongside an illustrative PowerShell
workflow and a localized prompt readers can copy into their coding agent. The CLI guide supplies
the detailed setup instructions. Both copy buttons have independent feedback and select their own
text if clipboard access fails; the prompt and install command remain readable without JavaScript.

The feature gallery shows one feature at a time, with click and Left/Right/Home/End
keyboard navigation. Each one plays the manual's animation of it in the page's
language (an iframe of `manual/?only=<lesson>&embed`, `FEATURE_LESSONS` in
`build-site.py`), loaded the first time its tab opens; it plays while in view, has
its own play / pause button and links to the lesson's steps. With JavaScript disabled
all six recordings appear with native video controls instead. Five of their covers
come from the revision-2 desktop, Peek, tabs, automatic sorting and folder scenes; the
hide/show cover remains a frame from its existing recording. Installation requirements expand without JavaScript;
clipboard copying is available on HTTPS and localhost. No external fonts, UI libraries
or additional build dependencies are required.

## Building locally

```powershell
uv run python scripts/build-site.py
```

Open `dist/site/index.html` in a browser. `--base http://localhost:8000` rewrites the
canonical URLs for a local server, and `--strict` fails on any language file whose
keys differ from `en.json` (the deployment workflow uses it).

## Publishing

The site is served by **Cloudflare Pages** from the project `pecofence`
(`pecofence.pages.dev`), which was created as a direct-upload project: deployments are
pushed to it with wrangler rather than pulled from Git. The custom domain
`pecofence.jiang.jp` is attached to the project; the zone `jiang.jp` lives in the same
Cloudflare account.

### Deploy from this machine

```powershell
uv run python scripts/build-site.py --strict
npx wrangler pages deploy dist/site --project-name pecofence --branch main
```

`npx wrangler login` once beforehand. The `--branch main` deployment becomes production;
any other branch name creates a preview URL.

### Deploy from GitHub

`.github/workflows/website.yml` runs the same two steps on every push to `main` that
touches the site. It needs two repository secrets: `CLOUDFLARE_API_TOKEN`, a token with
**Account → Cloudflare Pages → Edit**, and `CLOUDFLARE_ACCOUNT_ID`.

### DNS

Pages does not create the record on its own. The zone needs one record, proxied or
DNS-only:

```
CNAME  pecofence  pecofence.pages.dev
```

The domain shows as **Active** in the project's Custom domains tab a few minutes after
the record exists, and Cloudflare issues the certificate itself. The `CNAME` and
`.nojekyll` files in the build output are only meaningful to GitHub Pages and are
harmless here.

### Fallback: GitHub Pages

`.github/workflows/pages.yml` can deploy the same output to GitHub Pages when run
manually from the Actions tab. To use it as the real host instead: in the repository's
**Settings → Pages** set **Source** to **GitHub Actions**, point the DNS record at
`<account>.github.io` (DNS only until the certificate exists), enter
`pecofence.jiang.jp` as the custom domain, and turn on **Enforce HTTPS**.

## Changing copy

Edit `site/i18n/en.json` first, then update every other language file with the same
key. Keep UI terms identical to the language's catalog in `locales/`, and reuse the
wording of the matching README in `docs/readme/`. Strings whose keys are inserted
with `{{raw:...}}` in the template may contain the `<kbd>` and `<code>` markup shown
in `en.json`; everything else is escaped.

## The user manual

`site/manual/` is an interactive manual: 15 lessons whose demos are drawn in HTML and
CSS (a miniature Windows desktop, fences, a scripted pointer with click rings, gesture
captions and keycaps), a step list that highlights in sync, a scrub bar and a half-speed
switch, plus two reference appendices (shortcuts, FAQ). The build writes one copy per
language beside its home page: `/manual/` for English, `/<language>/manual/` for the
others, with the home page's header and footer (`site/partials`, `base.css`) filled into
the page's `<!--manual:header-->` and `<!--manual:footer-->`. Its scripts and stylesheets
get content-hash `?v=` queries at build time, like the site's. The manual is wider than
the home page (a contents column beside each animation), and its header and footer
line up with it.

| Path | Purpose |
|---|---|
| `page.html` | The zh-CN page, the master copy of every lesson's text |
| `manual.js` | Engine and player (`window.PM`). Every demo is a pure function of time, so `?only=<lesson>&bare&seek=<s>` freezes one frame of one lesson; `?only=<lesson>&embed` is the stage alone with a play / pause button, for the home page's gallery |
| `demos/<lesson>.js`, `demos/*.css` | One demo per lesson; `settings-mock.js` draws the settings window for the settings lessons |
| `assets/` | Icons, the wallpaper and the logo used inside the demos |
| `i18n/<language>.json` | Translations: `page` maps each text block of `page.html` (inner HTML) and `stage` maps each Chinese string literal of the demos (plain text) to the language |

Edit lessons in `page.html` and the demo scripts, then refresh the translation files:

```powershell
uv run python scripts/manual_i18n.py --prefill en zh-TW ja ko de fr es pt-BR ru it   # add new keys
uv run python scripts/manual_i18n.py --check                                     # coverage per language
uv run python scripts/manual_i18n.py --build ja                                  # preview into .cache/manual-build/ja (no site header)
```

`--prefill` fills demo strings that are the app's own UI text from `locales/<language>.json`,
so the demos show the words the app really uses; translate the remaining keys by hand.
Text in the page that names app UI must use the same catalog wording. `build-site.py
--strict` fails while any manual string of any language is untranslated. Demo texts are
replaced as whole literals at build time, so lookups such as a menu row found by its
label keep working in every language; menus grow to fit longer translations and long
fence titles ellipsize like the app. The lessons describe the real app: check behaviour
and UI strings against the code (`crates/app/src/app/menus.rs`, `ui/settings.html`)
when changing them.

## Refreshing media

`uv run --with pillow python scripts/make-site-media.py` regenerates the clips, posters, panels and
wallpaper from `extras/pecofence-promo/public/`, which is a local, ignored directory.
Revision-2 scene captures and original wallpaper under `.cache/store-v2/` supply the
current hero backdrop and feature covers. Add `--stills-only` to update only these images
without re-encoding the unchanged videos.
The exported files in `site/assets/` are checked in so the site builds anywhere.

`uv run --with pillow python scripts/make-readme-media.py --stills-only` regenerates
the eleven README/share images from that same desktop and the shared copy under
`docs/store/v2-i18n/`. The generated share URLs contain each image's content hash,
and Open Graph/Twitter metadata declares the corresponding localized image.

## Analytics

The template loads the Cloudflare Web Analytics beacon (site `pecofence.jiang.jp` in the
Cloudflare account, token in `site/template.html`). It counts page views and Core Web
Vitals without cookies or fingerprinting; the dashboard is under **Analytics & Logs →
Web Analytics** in Cloudflare.
