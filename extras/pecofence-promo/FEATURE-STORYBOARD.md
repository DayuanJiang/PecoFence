# PecoFence — feature demonstration plan, v2

Status: implemented as the 75-second feature tour. The existing 30-second film
remains the first version. The table below describes the final cut.

Format: 75 seconds, 1920 × 1080, 30 fps. English titles, action labels and
sample names. Retain the first film's visual style and use most of the screen for
the product demonstration.

Final capture adjustments:

- The short second tab is named `Art`. Its chapter starts with the combined window,
  shows Work → Art → Work content switches, and then separates it into two groups.
  The live drag-merge take was excluded after a window-display anomaly in capture.
- Sample names use a `-demo` suffix so capture files and routing rules are isolated.
- The new-group rectangle, tab separation, Peek, quick hide and roll-up are driven
  by the app's existing test script through normal production handlers. Other
  interactions use native mouse/control input. Default gesture and keyboard
  annotations explain the equivalent controls without claiming shortcut testing.
- Peek appears above a staged Windows Forms project brief. The real app groups
  remain visible above it until the normal end-Peek handler restores the prior view.

## What the viewer should understand

PecoFence is a Windows desktop organizer that lets you:

1. Create and arrange groups of desktop files.
2. Automatically route new files with rules.
3. Work with a real folder directly from the desktop.
4. Combine groups into tabs.
5. Bring your groups above another application with a shortcut.
6. Hide or collapse groups when you want more space.

The four primary messages are **organize**, **automate**, **work with folders** and
**access from anywhere**. Tabs, hiding and roll-up support that workflow.

The first film spent substantial time on positioning copy and appearance. Its
sorting shot showed an organized outcome without proving rule-triggered behavior.
Folder portals and Peek were absent. This version must show each feature's trigger,
visible operation and outcome.

## One continuous example

Use a fictional website-launch project throughout the film:

- Virtual groups: `Work`, `Images`, `Documents`.
- Samples: `wireframe-demo.png`, `homepage-demo.png`, `brief-demo.txt`, `meeting-demo.txt`.
- Real folder: `Project assets`, containing `Brand` and `Screenshots`.
- Tab labels: `Work` and `Art`, kept short enough to remain fully readable.

Each scene starts from a recognizable state established earlier. Samples and
desktop arrangement should feel like one person's workflow.

## Main storyboard — 75 seconds

| Time | Feature and viewer benefit | Operation shown on screen | English copy | Required visible result |
|---|---|---|---|---|
| 00–04 | Product identification | Start on an actual organized desktop, with several useful groups visible. Establish the product immediately. | **PecoFence** / **A Windows desktop organizer** | Viewer can identify what the product does before the feature sequence starts. |
| 04–14 | Create and arrange groups | Complete the new-fence action; name the group `Work`; drag two sample desktop files into it; reposition and resize the group. | **Create your own groups** / `Create → Drop files → Arrange` | The same sample files are visibly grouped, and the group has a user-chosen position and size. |
| 14–26 | Automatic sorting | Establish real configured rules for the demo PNG and TXT files. Introduce `homepage-demo.png` on the desktop and show it appear in `Images` without a manual drag. Repeat with `brief-demo.txt` and `Documents`. | **Sort new files automatically** / `PNG → Images` / `TXT → Documents` | Two different file types reach their intended virtual groups. The destination and filename remain visible long enough to recognize. |
| 26–39 | Live folder portals | Identify `Project assets` as the connected real folder. Show its contents in a portal, enter `Brand`, return to the parent folder, then introduce a new sample file on disk and show the portal update. | **Put a live folder on your desktop** / `Browse subfolders` → `Updates with your folder` | The portal's contents change on navigation and reflect the newly added file. It is visibly more than a decorative group of shortcuts. |
| 39–50 | Tabs reduce occupied space | Start with the combined `Work` and `Art` window. Click both tabs so their different contents are visible, then separate Art into an independent group. | **Combine groups into tabs** / `Two groups, one window → Switch → Separate` | Tabs select distinct contents, and a tab becomes a window again. |
| 50–61 | Peek avoids uncovering the desktop | Place a staged project-brief window above the desktop. Display the default `Ctrl + Alt + Space` keys as the normal Peek handler raises groups. Annotate `Esc` as end-Peek restores the previous view. | **Reach your files from any app** / `Ctrl + Alt + Space` / `Esc to return` | The application remains present behind the raised groups; the desktop was not simply shown with Win+D. |
| 61–71 | Hide and roll-up provide quick space | Show the normal hide/restore handlers, then roll up Work and actually hover to expand it. Annotate the equivalent double-click gestures. | **Hide clutter. Keep quick access.** / `Double-click the desktop` → `Double-click a title` → `Hover to expand` | Viewers can distinguish hiding all groups from collapsing one group, and can see how each is brought back. |
| 71–75 | Brand and practical positioning | Return to the complete desktop. Add a restrained PecoFence end card. | **PecoFence** / **Open source · Lightweight · Windows 11** | The brand is associated with the functionality just demonstrated. |

Durations are editorial targets. Slow actions should be trimmed between steps
rather than making the complete interaction unreadably fast.

## Presentation rules

- Every feature follows **name → action → outcome**.
- The application occupies roughly 75–85% of the feature shot. Titles should not
  force an important interaction into a small corner.
- Use concrete feature names. Generic phrases such as “More clarity” should not
  replace explanations of what happened.
- Retain visible cursor movement during drag-and-drop. Show where the drag starts,
  the target highlight and the release.
- Display the relevant keyboard keys at the instant the shortcut is used.
- Keep the result of an action visible for at least one second.
- Use a close-up when the viewer must read a tab, filename or rule. Keep enough
  surrounding context to understand where the action occurs.
- Reveal no more than one primary instruction at a time. Increase information
  through demonstrations, not dense paragraphs of text.
- Keep aesthetics present throughout the film. Give no standalone long beauty
  shot to an appearance feature.
- The film must remain understandable when muted. Music supports the pacing.

## Accuracy rules

- **Virtual grouping and automatic sorting:** original files remain in place.
  If this benefit appears, label it specifically: `Virtual groups keep originals in place.`
- **Folder portals:** they show actual folder contents. Dragging a file into a
  portal can move it into that folder. Do not apply the virtual-group statement
  to portal file operations.
- **Rules:** demonstrate configured rules responding to a genuinely new file.
  Do not label manual movement or a purely decorative animation as proof of
  automatic sorting.
- **Peek:** demonstrate groups above an application. A desktop-only shot does
  not prove global access.
- **Tabs:** show a content switch, not only an extra title appearing.
- **Quick hide and roll-up:** use separate gesture labels and show both recovery
  actions. Confirm hover-open is enabled for the recording.
- Use actual product controls and footage for the principal interactions.
  Editorial labels can translate the explanation; they must not masquerade as
  product UI or imply English localization of native menus that is not present.
- The current feature list is the source for claims. Exact memory/CPU figures
  require a fresh measurement of the build being shown.

## Capture list before editing

| New recording | What it must contain | What to check before adding it to Remotion |
|---|---|---|
| `01-create-groups` | Creation, file grouping, repositioning and resizing | Pointer actions and destination files are legible. |
| `02-auto-rules` | Rule context plus two new-file routing examples | Routing is automatic and targets virtual groups. |
| `03-folder-portal` | Parent folder, subfolder, return and live update | The same real folder and sample filenames are used throughout. |
| `04-tabs` | Combined state, both tab selections and separation | Short tab names remain fully visible. |
| `05-peek` | Application foreground, shortcut, raised groups and return | Application context stays visible; no unrelated personal content appears. |
| `06-hide-roll` | Hide/restore all, roll up one, hover-open | Gesture labels match the action and the recovery is visible. |

Capture with an isolated demo configuration and prepared sample files. Review
each clip independently before building the full timeline. Reuse existing assets
only where they demonstrate the planned behavior.

## Secondary features

These are useful but should not displace the primary demonstrations in the
75-second cut:

- Icon, list and details views, including sorting.
- Per-group color, opacity and icon spacing.
- Layout snapshots and restore.
- Multiple monitors and swapping monitor layouts.
- Familiar Windows copy/paste, drag-out and keyboard operations.

If the running time becomes 90 seconds, use the extra time for **views and
customization** and **layout restore**, each with a before/after operation.

If the running time must be 45 seconds, prioritize **grouping**, **automatic
rules**, **folder portals** and **Peek**, and omit the standalone tabs and
hide/roll-up chapters.

## Review before the final render

1. Can a first-time viewer name the six main features without reading a feature list?
2. Does each claim have a visible operation or result in the footage?
3. Can viewers distinguish automatic rules, virtual groups and folder portals?
4. Do the cursor, gesture labels and actual result agree?
5. Is each key filename, tab and rule readable at ordinary playback size?
6. Does the video still make sense with sound disabled?
7. Is there enough time to recognize the outcome before the next cut?

Feature references: the repository's `README.md` and `docs/FEATURES.md`.
