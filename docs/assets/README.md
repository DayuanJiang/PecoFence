# README media

These selected assets are part of the public documentation:

- `hero-<language>.png`: localized artwork over the existing native PecoFence demo
  desktop, one per README language (`en` for the root README, the rest for
  `docs/readme/`). The screenshot shows the Fluent theme; only the headline,
  subline, caption and badge text differ.
- `tabs.gif`: native tab switching and detaching.
- `peek.gif`: native Peek activation and returning to the application.

The GIFs are cropped, resized excerpts of the existing 75-second feature film.
They use demonstration files and contain no footage of the user's working desktop.
The screenshots' panels have not been redrawn.

Regenerate with `python scripts/make-readme-media.py` using Pillow, FFmpeg and the
original local media under `extras/pecofence-promo/`. Generated preview pages and
contact sheets stay under `.cache/`; the full recordings remain excluded from
the source export.
