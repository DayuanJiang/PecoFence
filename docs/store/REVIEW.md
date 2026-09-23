# Store visual review — 2026-09-23

## Verdict

The first local listing pack is a draft and should not be published as the final
redesign. It passed technical asset checks, but those checks do not establish
visual quality. The screenshots need stronger scenes and more distinct feature
communication. The earlier “ready” framing was premature.

The user asked for an actual comparison with other Store listings. Four live
Microsoft Store pages and their expanded screenshots were inspected:

| Reference | Observed treatment | Useful direction |
|---|---|---|
| [Files App](https://apps.microsoft.com/detail/9nghp3dx8hdx) | The second screenshot uses a short Columns View headline beside a large cropped native window. Other screenshots focus on different workflows. | One feature per image; readable native details and a concise descriptive heading. |
| [BeWidgets](https://apps.microsoft.com/detail/9nq07fg50h2q) | The first two screenshots show a composed desktop in dark and light treatments, with large real clock widgets and application shortcuts. | Show a desirable finished desktop before explaining configuration. |
| [Lively Wallpaper](https://apps.microsoft.com/detail/9ntm2qc6qws7) | The hero shows a rich wallpaper collage; screenshots show the library, dynamic wallpaper customization and music visualization. | Use meaningful changes in content, color and scenario across the set. |
| [TranslucentTB](https://apps.microsoft.com/detail/9pf4kz2vn4w9) | Several different wallpapers demonstrate the resulting desktop and taskbar. The taskbar itself is small in thumbnails. | Reference the desktop atmosphere, but do not copy its weak feature visibility at small sizes. |

These are observations of published visuals, not evidence of conversion performance
or a guarantee that another submission will receive the same certification result.
BeWidgets also demonstrates that a large icon can work as hero art when related
objects provide useful context; the problem with PecoFence's first hero is repetitive
decoration without additional product meaning.

## Problems in the PecoFence draft

1. All five screenshots reuse the same blue wallpaper and very similar panel framing.
   At Store thumbnail sizes they are difficult to distinguish.
2. The fixtures are sparse folders, text files and simple image samples. They
   demonstrate rendering but do not present a compelling everyday workspace.
3. Static auto-sort and folder screenshots do not provide enough visible evidence
   of their different workflows.
4. Captions below the images carry too much of the explanatory work.
5. The preview shows an asset catalog, not the Store's real placement, crop, fade and
   thumbnail scale. A large standalone hero can therefore appear more effective in
   the catalog than it will in the actual listing.

## Revision brief

- Prepare new native demo scenes instead of repeatedly cropping the same recordings.
  Retain authentic product rendering, controls and supported behavior.
- Lead with a complete, intentionally arranged desktop with credible project material.
  The first image should establish what a customer could make their desktop look like.
- Follow with a clear Peek scenario over a real demonstration work window, a close
  view of project tabs, a readable automatic-sorting scene, and distinct appearance
  examples using actual supported themes.
- Give each image one idea, a distinct composition and enough native UI scale to
  survive thumbnail display. Consider short descriptive feature headings as a design
  direction, while checking the actual submission field's applicable guidance.
- Evaluate both the complete artwork and its use in Store-shaped layouts. Do not
  infer quality from dimensions, file sizes or the count of available assets.
- Preserve the old draft for comparison; do not replace public listings until the
  revision is complete and reviewed.

The local visual comparison is in `dist/store-review/index.html`; full-page captures,
expanded screenshots and retrieval notes are under `.cache/store-benchmark/`.
Reference screenshots are only for this review. They are not part of the upload pack
or the public PecoFence website.

## Revision 2 continuation

The interrupted task's second revision was recovered and completed locally.
See [revision 2](V2.md) and `dist/store-listing-v2/index.html`.

- Fresh native captures use original design-study thumbnails, a project brief,
  project folders and everyday documents.
- Five campaign layouts distinguish the desktop, Peek, tabs, automatic sorting,
  and appearance with different compositions. Headlines are localized in Chinese,
  English and Japanese; native demo UI remains English.
- The Store screenshot bundle uses only native crops and proportional resizing.
  Headlined campaign layouts are kept outside that upload bundle. The preview
  starts with native screenshots and lets reviewers switch asset types.
- The hero's rounded frame now has transparent corners instead of a rectangular
  fill protruding beyond its outline.
- Validation checked all 20 screenshot PNGs, three sets of copy, source/output
  hashes, exact native crop/resize pixels, local HTML references, JavaScript
  syntax and ZIP contents.
- Browser interaction and responsive rendering were not verified in this
  continuation: the browser security policy rejected opening the local file URL.
  The artwork was inspected directly as local images.

The first draft remains available. Neither the Store nor the website was published
by this continuation. Visual review and technical checks do not measure conversion.

## Complete language coverage

The initial three-language scope was inherited from the first Store pack. It did
not cover all languages supported by the product. Revision 2 now includes all ten
languages from `site/site.json`: 50 campaign images, ten complete sets of Store
copy and native screenshot captions, and a translated preview interface.

Traditional Chinese uses Microsoft JhengHei, Korean uses Malgun Gothic, Japanese
uses Yu Gothic, and Simplified Chinese uses Microsoft YaHei. Latin and Cyrillic
text use Segoe UI. Native filenames and user-defined group titles retain their
original sample values.

Language options are generated from the project catalog. Missing locales or
translation keys stop the build instead of silently falling back to English.
