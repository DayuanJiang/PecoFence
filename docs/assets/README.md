# README media

These selected assets are part of the public documentation:

- `hero-<language>.png`: 1600×900 localized artwork over the revision-2 native PecoFence demo
  desktop, one per README language (`en` for the root README, the rest for
  `docs/readme/`). The screenshot shows Liquid Glass with Projects, Inspiration,
  and Today groups. The headline and subline use the same copy as the Store
  campaign in `docs/store/v2-i18n/`. These images also serve as website share previews.
- `tabs.gif`: native tab switching and detaching.
- `peek.gif`: native Peek activation and returning to the application.

The GIFs remain cropped, resized excerpts of the existing 75-second feature film,
using the Fluent theme.
They use demonstration files and contain no footage of the user's working desktop.
The screenshots' panels have not been redrawn.

Regenerate still images with
`uv run --with pillow python scripts/make-readme-media.py --stills-only` using the
original local capture `.cache/store-v2/native/overview.png`. Omit `--stills-only`
to also regenerate the GIFs using FFmpeg and the recordings under
`extras/pecofence-promo/`. Generated preview pages and
contact sheets stay under `.cache/`; the full recordings remain excluded from
the source export.
