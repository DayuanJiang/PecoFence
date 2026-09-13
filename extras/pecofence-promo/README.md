# PecoFence — English product videos

Review plans and capture-provenance records under `review/` stay local and are
excluded from the public source export, along with the full recordings.

The current promo is a **30-second English spot in a cold-cyan control-console
style**, made with Remotion 4.0.522. A near-black field, rotating wireframe instruments,
holographic scans and geometric typography frame automatic file sorting and tabs
demonstrations. Headlines hold long enough to read; Peek, shortcuts and file
opening are left to the feature clips on the product site.

- 1920 × 1080, 30 fps, H.264 MP4 with stereo AAC audio.
- Real PecoFence captures with English demo folders and files.
- Programmed motion design, English on-screen copy, original instrumental soundtrack.
- No voiceover.

## Preview and export

```powershell
npm install
npm run dev
npm run reviewed:stills
npm run reviewed:render
npm run reviewed:verify
```

Current output: `out/PecoFence-autosort-en-1080p.mp4` and
`out/PecoFence-autosort-en-1080p-poster.png`. Preview composition: `PecoFence-Reviewed`.
Delivery uses 1080p H.264, BT.709, 30 fps and stereo AAC.
The current music is an original 128 BPM dance track with a four-on-the-floor
kick, backbeat claps, offbeat hats, syncopated bass and transition risers.
Kick/impact accents align with 7, 15 and 23 seconds.

| Time | What the viewer sees |
|---|---|
| 0–7 s | Work and Art panels assemble from depth beside the product name and definition |
| 7–15 s | New Coast.png and Notes.txt files automatically appear in Art and Work |
| 15–23 s | Native groups merge and switch; the camera closes in on the tabbed panel |
| 23–30 s | Panels reassemble beside the brand, closing promise and availability |

`review/PECOFENCE-AUTO-PLAN.md` records the current update;
`review/TECH-PLAN.md` records the independently reviewed visual direction. Projected
wireframes, orbital rings, optical particles, angular HUD brackets, holographic
panel scans, electronic scan gates and synchronized grouping signals provide motion.
Bahnschrift supplies geometric display type using the installed Windows font.
Text appears in whole blocks and settles within one second. The opening definition
has about 5.6 unobscured seconds before the outgoing transition; closing copy has
six seconds. Feature headlines remain fixed through their eight-second scenes.
Two-second native-frame holds preserve reading time before the functional actions.

Each scene in `src/motion/` has named, editable text and media layers with frame-based
trims. `src/reviewed/ReviewedFilm.tsx` forwards the existing preview route to this film.
`scripts/verify-reviewed.mjs` checks the actual MP4, fully decodes it, checks audio
and black frames, and creates timestamped frame samples plus a SHA-256 report.
The automatic-routing evidence and final export review are documented in
`review/PECOFENCE-AUTO-REVIEW.md`. The music revision is documented in
`review/ENERGY-AUDIO-REVIEW.md`; the preceding visual review is in
`review/TECH-REVIEW.md`. Earlier verdicts remain in
`review/MOTION-REVIEW.md`, `review/READABILITY-REVIEW.md` and `review/FINAL-REVIEW.md`.

Current footage in `public/autosort/auto-v2/` comes from an isolated copy of the
product source, with only a disclosed debug fixture-directory resolver override.
The product's rendering, grouping and tab handlers are unchanged. Posted native
mouse messages and test-script commands drive the demonstration. The fixture
contains four files, with per-file identity, integrity and membership proofs.
Two files are present initially. New PNG and TXT files are then created in the
isolated Desktop, and the normal filesystem watcher and enabled extension rules
route them to Art and Work. `keepUpdated` is enabled at startup; no manual routing,
rule refresh or item-assignment command triggers this operation. File contents
and disk paths remain unchanged. The same groups then merge and switch tabs.
`public/motion/autosort-*` contains native panel crops, the recording normalized
to 30 fps, and its source hash and exact edit timing.
3D staging, cyan grading, scan lines, wireframes, brackets, packets and scan gates
are editorial presentation effects. The “New files” tray and light trails are
editorial cues, separate from the captured product panels. Native geometry and the recorded functional
actions are preserved. The current film contains no foreground project board,
document viewer, fake metrics or additional system-status claims.

To rebuild the edited footage from the saved raw take:

```powershell
npm run autosort:prepare
npm run music:energy
```

The exact capture build command and source identity are recorded in
`public/autosort/auto-v2/capture-manifest.json`. The capture scripts and source
snapshot remain local under `.capture/`. Rendering needs only the saved assets.
Earlier exports keep their original filenames, including
`out/openFence-tech-energy-en-1080p.mp4`.

## Earlier 28-second launch film

```powershell
npm run launch:stills
npm run launch:render
```

Earlier output: `out/PecoFence-launch-en-1080p.mp4` and
`out/PecoFence-launch-poster.png`. Preview composition: `PecoFence-Launch`.

| Time | What the viewer sees |
|---|---|
| 0–3 s | A close reveal of the current glass desktop |
| 3–8 s | Complete groups with documents, folders and image thumbnails |
| 8–15 s | Work / Art tab switching and native tab separation |
| 15–23 s | Peek brings files above a staged project workspace |
| 23–28 s | PecoFence: Make room for focus |

See `LAUNCH-STORYBOARD.md` for the short-film plan. It favors benefits and native
motion over instructions. It contains no voiceover or numbered tutorial steps.

### Refreshing the launch footage

```powershell
python scripts/make-launch-assets.py
./scripts/capture-features.ps1 -Scenes hero,tabs,peek -Launch
npm run launch:prepare
node scripts/make-assets.mjs 28 -launch 120
npm run launch:render
```

The `-Launch` capture mode records the current local debug executable in separate
portable demo instances, explicitly selecting `language: en` and
`themeStyle: liquidGlass`. The three groups contain staged sample folders, documents
and original image thumbnails. All launch product images are fresh captures.
`public/launch/capture-manifest.json` records the build timestamp and capture settings.

The launch tab clicks use native window input messages. Tab separation and Peek
use the existing app test-script commands and their normal production handlers.
This capture does not test physical drag input or global shortcut registration.
The visible shortcut is the documented user control. The Launchpad project board
is original staged artwork in a real Windows
Forms window, used as the foreground application for the Peek demonstration.

The opening, glass beauty shot and closing use actual current-build screenshots
with editorial camera movement and typography. The tab and Peek clips are trimmed
and retimed native recordings. Their windows have not been redrawn. The 28-second
120 BPM track is synthesized by `scripts/make-assets.mjs`.

### Earlier feature walkthrough

```powershell
npm run features:stills
npm run features:render
```

Earlier output: `out/PecoFence-features-en-1080p.mp4` and
`out/PecoFence-features-poster.png`.

The earlier timeline is `src/features/FeatureFilm.tsx`. Its English headings,
step-by-step captions and operation callouts accompany six native recordings:

| Time | Demonstration |
|---|---|
| 0–4 s | What PecoFence does |
| 4–14 s | Create and name a group, drag in two files, move and resize |
| 14–26 s | New PNG and TXT files automatically reach the configured groups |
| 26–39 s | Browse a folder portal, return to the parent, see a new file appear |
| 39–50 s | Switch between Work and Art tabs, then separate the groups |
| 50–61 s | Raise groups over a sample application with Peek, then return |
| 61–71 s | Hide and restore groups, roll up one group, hover to expand |
| 71–75 s | Closing brand card |

See `FEATURE-STORYBOARD.md` for the feature plan and editorial decisions.
The feature film is understandable without sound; its music contains no voiceover.

To regenerate its trimmed clips and timing data after recording:

```powershell
npm run features:prepare
node scripts/make-assets.mjs 75
```

To render the original brand film:

```powershell
npm run stills
npm run render
```

The renderer uses installed Google Chrome on Windows. To use a different compatible
browser, set `REMOTION_BROWSER_EXECUTABLE` to its executable path.

Output: `out/PecoFence-promo-en-1080p.mp4` and `out/PecoFence-poster.png`.

Each scene is also registered separately in Remotion Studio. Copy and layout live in
`src/scenes/`; the main timeline is `src/Promo.tsx`.

## Original brand film storyboard

| Time | Scene | Headline |
|---|---|---|
| 0–4.3 s | Scattered desktop files | Less clutter. More clarity. |
| 4.3–10.2 s | Organized real product panels | Everything in its place. |
| 10.2–15.2 s | Acrylic material close-up | Native feel. Clear thinking. |
| 15.2–20.7 s | Recorded tab merge and roll-up | More space. Same desktop. |
| 20.7–25.2 s | Animated quick-hide demonstration | Your desktop. Room to breathe. |
| 25.2–30 s | Brand closing frame | Make room for focus. |

Transitions overlap by 12 frames. Organization and quick-hide shots use motion
graphics built from actual panel captures; the tab/roll-up shot uses a real recording.
The file grouping shot illustrates functionality documented in the main project README.

## Asset provenance

- `public/features/*-raw.mp4`: new native recordings from isolated portable demo
  instances. Each contains only staged sample content inside the capture area.
- `public/features/{groups,rules,portal,tabs,peek,hide}.mp4`: cropped, trimmed and
  retimed versions prepared by `scripts/prepare-feature-media.mjs`.
- `public/features/*-events.json` and `src/features/timings.ts`: action timestamps
  used to align English captions with the edited recordings.
- `public/features/rules-proof.json`: recorded group membership after two genuinely
  new sample files were automatically routed. The demo rules also restrict matching
  to the staged `-demo` filenames.
- `public/soundtrack-features.wav` and `.m4a`: 75 seconds of original synthesized
  instrumental music and edit sounds.
- `public/panel-*.png`: captured from the local PecoFence debug build.
- `public/product-demo.mp4`: real desktop capture of the demo instance.
- `public/product-detail.mp4`: crop of that recording.
- `public/folder-icon.png`, `public/document-icon.png`: local Windows shell icons,
  shown in the context of this Windows product demonstration.
- `public/wallpaper.png`: original procedural graphic created by `scripts/make-assets.mjs`.
- `public/soundtrack.wav` and its AAC copy `soundtrack.m4a`: original synthesized music and edit sounds from the same script;
  no stock music, external samples or cloned voices.
- The simple PecoFence motion mark is authored as SVG in `src/design.tsx`.

`scripts/capture-product.ps1` uses an isolated portable demo configuration and sample
files under the ignored `.capture/` directory. It temporarily shows a capture
backdrop, records the product, and restores the prior foreground window. It depends
on a 2560 × 1440 or larger Windows display, the source repository's debug executable,
and the existing monitor configuration. FFmpeg is required for recording and asset
generation. Rendering the included composition needs only the saved public assets.

`scripts/capture-features.ps1` records the revised film. It temporarily creates
uniquely named sample files, refuses to overwrite existing ones, and removes only
the samples it created. Desktop enumeration is confined to an excluded private
inbox outside the captured area. The app's normal automatic rules and filesystem
watchers handle the routing and live folder update.

Creation menus, renaming, file drags, group positioning/resizing, portal navigation
and tab clicks are recorded through native controls. The existing app test-script
commands drive creation of the initial new-group rectangle, tab separation, Peek,
quick hide and roll-up through their normal production handlers. Shortcut and
gesture callouts explain the corresponding user controls; these particular
recordings do not test global keyboard registration. The other application in the
Peek shot is a staged Windows Forms project-brief window.

The tabs chapter begins with an already combined group and demonstrates both
content switches and separation. A live drag-merge take was excluded because the
capture developed a window-display anomaly. No simulated merge replaces it.
Some native empty-state text and transient menus remain in the build's language;
the film's editorial copy and sample names are English.

Remotion has its own license terms. The product repository's Apache 2.0 license does not
replace Remotion's license.
