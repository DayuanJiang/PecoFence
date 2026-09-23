# Store presentation

**Revision 2:** see [the second-version preview and upload bundle](V2.md).
Its entry point is `dist/store-listing-v2/index.html`. The instructions below
describe the preserved first draft.

**Draft status:** the first pack is not the final visual redesign. A comparison with
four published Store listings found that its scenes and feature communication need
reworking. Read [the September 23 review](REVIEW.md) before using these assets.

The September 23, 2026 public listing had two similar Fluent desktop screenshots,
no visible trailer, and a feature-led introduction. This refresh pairs the website's
“Less clutter. More space.” positioning with native Liquid Glass screenshots,
benefit-led copy, and the existing product film.

## Build and review

```powershell
uv run --with pillow --with playwright python scripts/make-store-media.py
```

Requires FFmpeg, Edge or Chrome, and the original local captures under
`extras/pecofence-promo/public/` plus
`extras/pecofence-promo/out/PecoFence-autosort-en-1080p.mp4`.
The site itself builds without these capture files.

Open `dist/store-listing/index.html` to review. Use `--skip-trailer` when iterating on
copy/art; this reuses a previously encoded trailer if present. The script only writes
its named outputs and never deletes a destination directory.

`listings.json` is the copy source for en-US, zh-CN and ja; it is not a Partner Center
import schema. Copy the generated plain-text fields into the matching listing.
No MSIX rebuild is required for these listing-only changes.

## Upload fields

| Partner Center field | Prepared asset |
|---|---|
| Desktop screenshots | `screenshots/01-desktop.png` through `05-live-folder.png`, in order |
| Screenshot captions | `listings/<locale>/captions.txt`, one line per screenshot |
| Short description, description, features, keywords | Matching text files under `listings/<locale>/` |
| 16:9 Super hero art | `art/super-hero-1920x1080.png` |
| 1:1 app tile icon | `art/app-icon-300x300.png`, rendered with the existing package icon renderer |
| Trailer | `trailer/PecoFence-30s-en.mp4` |
| Trailer thumbnail | `trailer/thumbnail-1920x1080.png` |
| Trailer title | `listings/<locale>/trailerTitle.txt` |

Upload screenshot assets separately for each language. The video can be reused across
listings. Its on-screen text remains English, identified in the Chinese and Japanese
trailer titles. Screenshots also use English demo file/group names; the captions and
listing copy are localized. This is not a claim of localized screenshot UI.

`contact-sheet.jpg` and `index.html` are review aids, not upload assets.

## Asset decisions and provenance

- Screenshots use recorded product pixels with only a rectangular crop and proportional
  resize to 1920×1080. No headline, logo, badge, arrows or fabricated UI is composited
  into the screenshot uploads. `provenance.json` records source hashes and video times.
- The first, second and third screenshots use native Liquid Glass launch captures.
  The Peek screenshot shows real product windows above an original staged project
  board in a separate Windows window, identified in its caption.
- The fourth image comes from `autosort/auto-v2`, whose saved routing proof verifies
  new PNG/TXT files were grouped by the normal watcher without manual assignment or
  changes to their paths/content.
- The folder portal image shows the Fluent theme. Both themes are available in the app.
- `hero.svg` is an original vector extension of the bracket-and-four-tile app mark.
  It contains no text, device frame, or app UI. Key artwork is centered above the
  bottom third so Store gradients do not cover it.
- The trailer preserves the existing 30-second film and soundtrack. The Store export
  uses 1920×1080, 30 fps, H.264 High, 4:2:0, two B frames, closed 15-frame GOPs,
  50 Mbps target video rate, AAC-LC stereo at 48 kHz / 384 kbps, fast start and no edit list.
  Do not upload the website's smaller 720p export in its place.

Checked against Microsoft's [MSIX screenshot, image and trailer guidance](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)
on September 23, 2026. Actual placement and visibility vary between Store layouts.
Adding the assets does not guarantee promotional placement.

Changes are prepared locally. Publishing requires uploading them to the existing
product in Partner Center and completing the normal submission/certification flow.
