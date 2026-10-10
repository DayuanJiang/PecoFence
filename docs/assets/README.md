# README media

These selected assets are part of the public documentation:

- `hero-<language>.png`: 1600×900 localized artwork over the revision-2 native PecoFence demo
  desktop, one per README language (`en` for the root README, the rest for
  `docs/readme/`). The screenshot shows Liquid Glass with Projects, Inspiration,
  and Today groups. The headline and subline use the same copy as the Store
  campaign in `docs/store/v2-i18n/`. These images also serve as website share previews.
- `tabs.gif`: the manual's Tabs lesson (merge two fences, switch tabs, drag one out).
- `peek.gif`: the manual's Peek lesson (fences above a window, back behind it).

The GIFs are the English user-manual animations (`site/manual/`), shared by every
README language, at 800×450 and 12 fps with the step captions.
The screenshots' panels have not been redrawn.

Regenerate still images with
`uv run --with pillow python scripts/make-readme-media.py --stills-only` using the
original local capture `.cache/store-v2/native/overview.png`. Omit `--stills-only`
to also regenerate the GIFs using FFmpeg from the English Store trailer renders
`.cache/store-trailers/out/en/<lesson>.mp4` (made by `.cache/store-trailers/render.py`);
pass `--languages` with no value to skip the hero images.
