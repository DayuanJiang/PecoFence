# Product website

The product page at <https://pecofence.jiang.jp> is a static site generated from
`site/` and published with Cloudflare Pages. It needs no Node toolchain: the build is
`scripts/build-site.py` and the standard library.

## Layout

| Path | Purpose |
|---|---|
| `site/template.html` | One HTML template rendered once per language |
| `site/assets/site.css`, `site.js`, `mark.svg` | Responsive styles, desktop preview toggle, clip playback, command copy button, language picker and favicon |
| `site/assets/*.mp4`, `*.jpg`, `panel-*.png`, `wallpaper.jpg` | The 30-second spot (`promo.mp4`), six feature clips, posters, the three hero fences and the wallpaper, exported by `scripts/make-site-media.py` from the local promo project |
| `site/i18n/<language>.json` | Copy for each language; `en.json` is the source and every other file must have the same keys |
| `site/site.json` | Domain, repository URL and the language list |

The build writes `dist/site/`: `index.html` for English, one `<language>/index.html`
per translation, the localized README hero images as Open Graph previews, `CNAME`,
`robots.txt` and `sitemap.xml`. Pages carry `hreflang` alternates, so search engines
send visitors to their language; the header's language picker and the language links
do the same by hand. The picker preserves the current section.

The page uses a light canvas with the original desktop wallpaper and product panels
inside the hero preview. Feature videos play only while visible, with individual
pause controls, and never autoplay when reduced motion is preferred. The preview's
hide/show button demonstrates clearing the desktop. Installation requirements expand
without JavaScript; clipboard copying is available on HTTPS and localhost. No external
fonts, UI libraries or additional build dependencies are required.

## Building locally

```powershell
python scripts/build-site.py
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
python scripts/build-site.py --strict
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

## Refreshing media

`python scripts/make-site-media.py` regenerates the clips, posters, panels and
wallpaper from `extras/pecofence-promo/public/`, which is a local, ignored directory.
The exported files in `site/assets/` are checked in so the site builds anywhere.
