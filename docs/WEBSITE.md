# Product website

The product page at <https://pecofence.jiang.jp> is a static site generated from
`site/` and published with Cloudflare Pages. It needs no Node toolchain: the build is
`scripts/build-site.py` and the standard library.

## Layout

| Path | Purpose |
|---|---|
| `site/template.html` | One HTML template rendered once per language |
| `site/assets/site.css`, `site.js`, `mark.svg` | Responsive styles, desktop preview toggle, accessible feature tabs, clip playback, AI prompt and install command copy buttons, language picker and favicon |
| `site/assets/*.mp4`, `*.jpg`, `panel-*.png`, `wallpaper.jpg` | The 30-second spot (`promo.mp4`), six feature clips, posters, the three hero fences and the wallpaper, exported by `scripts/make-site-media.py` from the local promo project |
| `site/assets/showcase-*.webp`, `showcase-wallpaper.jpg` | Native panel crops and the original wallpaper from the revision-2 Store scene |
| `site/i18n/<language>.json` | Copy for each language; `en.json` is the source and every other file must have the same keys |
| `site/site.json` | Domain, repository URL and the language list |

The build writes `dist/site/`: `index.html` for English, one `<language>/index.html`
per translation, the localized README hero images as Open Graph and Twitter previews, `CNAME`,
`robots.txt` and `sitemap.xml`. Pages carry `hreflang` alternates, so search engines
send visitors to their language; the header's language picker and the language links
do the same by hand. The picker preserves the current section.

The page pairs a warm paper-and-lavender hero with a light reading canvas. Native
Liquid Glass panel crops (`showcase-*.webp`) sit beside the headline and primary
Store download link. They show Projects, Inspiration and Today from the revision-2
capture; their composition on the website is editorial.
The portable download is a secondary text link and the film has a separate play action.

The hero also links directly to AI configuration through its CLI badge and a secondary action.
The AI + CLI section follows the hero, before the film and feature gallery. It presents settings,
organization rules and configuration backup as everyday uses, alongside an illustrative PowerShell
workflow and a localized prompt readers can copy into their coding agent. The CLI guide supplies
the detailed setup instructions. Both copy buttons have independent feedback and select their own
text if clipboard access fails; the prompt and install command remain readable without JavaScript.

The feature gallery shows one large native scene cover at a time, with click and Left/Right/Home/End
keyboard navigation. With JavaScript disabled all six clips appear with native video
controls. Five covers come from the revision-2 desktop, Peek, tabs, automatic sorting
and folder scenes; the hide/show cover remains a frame from its existing recording.
Videos play on request and pause when hidden or offscreen. The preview's hide/show button
demonstrates clearing the desktop. Installation requirements expand without JavaScript;
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

## Refreshing media

`uv run --with pillow python scripts/make-site-media.py` regenerates the clips, posters, panels and
wallpaper from `extras/pecofence-promo/public/`, which is a local, ignored directory.
Revision-2 scene captures and original wallpaper under `.cache/store-v2/` supply the
current hero and feature covers. Add `--stills-only` to update only these images
without re-encoding the unchanged videos.
The exported files in `site/assets/` are checked in so the site builds anywhere.

`uv run --with pillow python scripts/make-readme-media.py --stills-only` regenerates
the ten README/share images from that same desktop and the shared copy under
`docs/store/v2-i18n/`. The generated share URLs contain each image's content hash,
and Open Graph/Twitter metadata declares the corresponding localized image.

## Analytics

The template loads the Cloudflare Web Analytics beacon (site `pecofence.jiang.jp` in the
Cloudflare account, token in `site/template.html`). It counts page views and Core Web
Vitals without cookies or fingerprinting; the dashboard is under **Analytics & Logs →
Web Analytics** in Cloudflare.
