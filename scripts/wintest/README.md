# Desktop tests on a Windows test box

These scripts run desktop tests of PecoFence (mouse, keyboard, screenshots, the real Windows
shell) from the Linux dev box on an EC2 Windows Server 2025 box, and print one pass/fail table.

```bash
scripts/wintest/build.sh                       # release build into target/wintest-bin (about 1 min)
scripts/wintest/run.sh scripts/wintest/tests/186-matrix.ps1 scripts/wintest/tests/186-search.ps1
scripts/wintest/unit.sh                        # unit tests in the desktop session (anchor tests too)
scripts/wintest/box.sh down ec2-2              # stop the box when the round is over
```

`run.sh` takes a box (`--box ec2-2`, default the first free one), copies the build and the files
in `--extra DIR` only when they changed, checks every steps file for parse errors and for
functions that a PowerShell alias would shadow, then runs one job per steps file in pf's desktop
session with a fresh `C:\pf\run` and fresh PecoFence data. Output:

```
== 186-matrix: 52 passed, 1 failed (4m10s)
   FAIL opened by double Ctrl, at once | Backspace | box='qza oneq'
   screenshots of the failures: .../186-matrix/shots/FAIL-*.png
== total: 52 passed, 1 failed
box ec2-2; lock wait 0s, box up 3s, files 6s (build same), jobs 250s
```

Each job folder has `results.tsv`, `job.log`, `logs/` (the app's log) and `shots/`. Look at
the screenshots of failed checks first; the others are for reports and visual checks.

## The whole suite

```bash
scripts/wintest/build.sh && scripts/wintest/build.sh --debug
scripts/wintest/installers.sh                  # old-setup.exe, new-setup.exe, portable.zip of a CI run
scripts/wintest/suite.sh --installers <the folder it printed>          # about 25 min on three boxes
scripts/wintest/suite.sh --only 'tabs|rolled' --bin <older release>    # a few lines, another build
```

`suite.txt` lists every steps file once per part (a line: name, minutes, where, build, file,
run.sh options). `suite.sh` spreads the lines over the boxes (`--boxes ec2-2,ec2-3,ec2-4`, each
takes the longest job still waiting), runs the `vm` lines in a Hyper-V guest with `--vm`
(`--in-vm` sends the ec2 lines there too), skips the lines that need `--installers` without it
the `manual` ones and, without `--signed`, the `signed` ones (they install the setup edition, which
turns autostart on: wait for signed builds), and prints one table: passed and failed per job, the failures, what it
skipped. Run it as the background command. A new steps file gets a line in `suite.txt`.
`installers.sh` downloads the build under test from a CI run (default: the latest green run on
main) and the release a user upgrades from (`--old v0.2.5`), and unpacks the CI build into
`bin/` for `--bin`.

## Rules for test rounds

- **Make sure the end of `run.sh` reaches whoever waits for it.** It returns when every job is
  done or timed out (`--timeout`, default 900 s per job). In the main Claude Code session, run it
  as the background command itself (`run_in_background`): the completion notice arrives. In a
  subagent, run it in the foreground (Bash timeout 600000 ms) and keep one call under about
  9 minutes (split the steps files): a subagent that went idle is not woken when its background
  command ends (2026-11-06: two finished runs sat unread for 6 and 17 minutes). The main session
  should check its agents' last activity every 15 minutes. Never start a separate loop that
  waits for the output: when such a loop stalls, nobody notices the finished results.
- One `run.sh` per box at a time: a lock in `/tmp/pecofence-wintest/locks/<box>` keeps other runs
  (other sessions too) off the box until the run ends; a lock whose process is gone is taken over.
  For parallel rounds, make another box (`box.sh create ec2-3`).
- Put the checks that combine features first (every way of opening a window x every key), and
  read outcomes (window text, which file opened, `fence get`) rather than judging screenshots.
- **Name what you mean, not where it was.** The first-run fences, the scene tips, new buttons and
  the settings page move between builds; a point that was empty desktop on one build is a fence
  or a card on the next (10 of the 14 failures that were not PecoFence's fault in the 2026-11-08
  round). Ask the kit instead: `FreeSpot` / `DeskClick` / `ParkMouse` / `FreeRect` for empty
  desktop, `ItemPt` for an icon, `OcrFind` / `OcrClick` for a label on a release build's page,
  `ClearFences` before steps that build their own layout at fixed places. Keep literal points
  only for geometry the check is about (a drop 20 px above the taskbar, a fence the steps made).

## Writing steps

A steps file is plain PowerShell that `job.ps1` dot-sources after `lib.ps1`:

```powershell
Case 'note editor'
PfStart -Script @('sleep 9000', 'new-note 760 120', 'sleep 3600000')
$ed = WaitWin 'PecoFence.NoteEdit' 25
Check 'new-note opens the editor' ($ed -ne [IntPtr]::Zero) "editor=$ed"
```

- `Case 'name'` starts a group; `Check 'what' <bool> 'detail'` is one row of the table. A failed
  check saves `FAIL-NN.png`. An error in a top-level line becomes a failed check and the next
  line runs; an error inside a block (`if`, a function) skips the rest of that block, so keep
  steps flat, or give a function of steps its own `trap { PfTrap $_; continue }`.
- A long file can run in parts across calls: `run.sh --var Part=AB ...` sets `$Part = 'AB'`
  for the steps (`tests/holdout2-fixes.ps1` reads its part another way, from `--extra`).
- `PfStart` starts the build under test as a portable instance (own config, own log; `-Script`
  gives test-script lines, `-Lang`; `$env:` variables set before it reach the app), `PfQuit`
  ends it like a sign-out, `Pf` / `PfJson` run `pecofence-cli` against it (`fence list`,
  `item list` and `rule list` print plain arrays), `PfFence`, `PfWaitLog`, `PfLogAll`,
  `PfLogErrors` (all logs of the job, also across restarts and of an installed copy).
- Input and windows: `[G]::Click/RClick/MoveTo/Type/Press/KeyDown/KeyUp/mouse_event`, `Chord`,
  `PeekKey`, `DoubleCtrl`, `TitleClick`, `WaitWin`, `Rect`, `Fg`, `[L]::Text/Cls/Tops/MoveWindow`,
  `[M]::BoxText()` (the Peek search box), `[QS]::Address/FileName/SetFileName` (file dialogs),
  `Explorers`, `UiaFind` (native windows only), `CloseWindows`, `CloseMenus`, `Shot`.
- More input and state (lib.ps1): `MDown`, `MUp`, `Glide`, `DragTo`, `Wheel`, `WheelAt`,
  `DoubleClick`, `ClientOrigin`, `NcHit`; popup menus `MenuWin`, `MenuRows`, `MenuTexts`,
  `MenuDupes`, `MenuPick`, `MenuClick`, `[Menu]::Items()`; windows `[W]::Tops/Buttons/Statics/
  IconsVisible/ToggleIcons/RootClassAt`, `WaitGoneWin`, `VisFences`; `[Anim]::Get/Set` (Windows
  animations); fence state at any moment `DumpScript` + `DumpOf` / `PfDump` (the test-script
  `dump` lines: rects, tab order, `tab_rects`, active tab, rolled), `FenceIds`, `FenceGet`;
  items `DeskFile`, `DeskLink`, `ItemFence`, `WaitItemFence`, `ItemPt` (by name, a fence sorted
  by name; This PC and the Recycle Bin first only when it is arranged like the desktop), `CellPt`
  and `IconCrop` (by index), `RClickItem`; rules `Rules`, `RuleById`, `WaitRule`; `StateText`
  (fences, rules, items, theme as lines to compare), `BinNames`, `RecycleDir`, `Snaps`;
  `PfFresh` (a first run again), `DumpVal` (any field of a dump line), `ShareLog`, `FeedbackInfo`
  (what the feedback form sends, from a `feedback-info` test-script line); `ClipFiles`,
  `ClipEffect`; `ShellMenuPoint` (Windows 11's own desktop menu too); `ProcTops`; `[Th]`: suspend
  a UI thread (a hang), end a process with an exit code (a crash's 0xC0000005), a layered
  window's alpha, how long a window takes to answer, Explorer's click-to-open, many Recycle Bin
  items at once.
- PowerShell 5.1 trap: a property of a plain array (`$list.fences` when the CLI printed an
  array) gives one `$null` per item, not nothing; filter for real items.
- The settings page (page.ps1) is a WebView2 page: UI Automation sees none of its content on the
  box. Use a debug build (`build.sh --debug`, then `run.sh --bin target/wintest-bin-debug`)
  started with `$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'`: `PfSettings`
  opens it and connects, `Js` / `JsWait` evaluate in the page, `PageRect`, `PageClick`,
  `PageType`, `PagePick` (a `<select>` by keyboard), `PageToast`; `$PfDebugBuild` says which build
  runs. On a release build, click by coordinates measured on the debug build.
- Pixels (pixels.ps1): `[Px]::Grab/Ink/FirstInk/MaxInk/Marker/Diff/Burst/Track/Strip`, `Crop`,
  `GreyWallpaper`, `MeanRgb`, `ExplorerIconRgb` (Explorer's own icon of a file, a reference for
  thumbnails), `OcrText` / `ToastOcr` (Windows' OCR: toasts and a release build's settings page,
  which UI Automation cannot see).
- Virtual desktops (vdesk.ps1): `VdNew`, `VdClose`, `VdLeft`, `VdRight` (Win+Ctrl+D/F4/←/→),
  `VdOnlyOne` (every job starts with one desktop), `VdTwo a b` (two desktops named in Task View),
  `VdRename`, `TaskView`, `TaskViewPick n`, `VdIds`, `VdCurrent` (Windows 10 too), `VdName`,
  `VdState`; fences by title shown or hidden: `[VD]::Fence`, `FVis`, `VisText`; `FenceMenu`,
  `VdSubmenu`, `CloseAllMenus` (an open fence menu holds PecoFence: switches are followed when it
  closes), `CloseToast`. Explorer caches desktop names: rename in Task View, never in the registry.
  `tests/vd-*.ps1` use them; `vd-win10.ps1` is for the PF-Win10 guest (no Task View clicks).
- Z-order (zorder.ps1, dot-source `C:\pf\kit\zorder.ps1`): `[Zs]::Run` presses Win+<key> and
  records which of the fences, the desktop windows and the test's Notepad / Explorer windows are
  above which (as fast as EnumWindows allows, frames optional), `ZsStats` sums up how long the
  fences were covered, topmost or over a window; `tests/desktop-clicks-and-focus.ps1` uses it for Win+D.
- Windows' own desktop icons (deskicons.ps1, dot-source `C:\pf\kit\deskicons.ps1`): `[DeskIcons]::List()`
  (each icon's parsing name and place), `Place` (IFolderView::SelectAndPositionItems), `Spacing`,
  `Flags` / `SetFlags` (1 自动排列图标). Explorer snaps a placed icon to its grid a second or two later.
  A place set only with `Place` does not last: hiding and showing the icons (0x7402) or an Explorer
  restart can put the icon back in the columns. Make a user's own layout with real mouse drags
  (`DragTo` from the icon's centre), which last like a user's. After a scale change Explorer keeps
  the old scale's grid until it restarts. `tests/fb-free-place-import.ps1` shows all of it.
- The user's own copy (user.ps1): `Ipf` / `IpfJson` / `UserLog` for the installed copy (set
  `$script:CliExe` for a portable one), `StartUser`, `WaitNoPf`; `PfUseUser` makes the item and
  state helpers read that copy; setup and uninstall by their buttons (`Wizard`, `Uninstall`,
  `SilentSetup`, `SilentUninstall`, `ClickButton`, `SetupBoxes`), the tray (`TrayIconRect`,
  `TrayMenu`, `TrayPick`), `CloseNotice`. `tests/holdout-*.ps1` show all of them in use.
- At the end of every job the app logs are checked for ERROR and panic lines (set
  `$PfAllowLogErrors = $true` when a test causes them on purpose); a job without any app log
  says so in `job.log` instead.
- Layout-independent points (lib.ps1): `FreeSpot [x] [y]` (bare desktop near a point, no fence,
  card, button or menu there or within 40 px), `DeskClick`, `ParkMouse`, `FreeRect W H [x] [y]`
  (an empty rectangle for a marquee), `ClearFences` (every fence but the inbox deleted);
  pixels.ps1: `OcrFind '^Label$' [x y w h]` (the centre of the words, anchored regex) and
  `OcrClick`.
- When a check fails, `job.log` gets a `FAIL NN context` line: the foreground window and every
  window above the fences (a popup, a terminal, a card), before anyone opens the screenshot.
- Windows Defender: a job whose build is gone at the start gets one failed row saying so, and
  every job ends with a `Windows Defender left the test files alone` row that names any detection
  of `C:\pf` files during the job. Its machine-learning verdicts (`Trojan:Win32/Bearfoos.A!ml`,
  `Behavior:Win32/Persistence.A!ml`) take unsigned builds away now and then, mostly in the
  Hyper-V guests. In a guest, run the CI build (`installers.sh` unpacks it into `<folder>/bin`
  for `--bin`): on 2026-11-08 Defender removed the dev-box build of main there mid-run and left
  the CI build of the same commit alone.
- Each job starts with fresh PecoFence data and ends Edge, Notepad and setup programs, but the
  desktop's files and an installed PecoFence stay: steps remove what they create and install.
- Files the steps need (installers, test programs) go in a folder given with `--extra`; on the box
  they are in `$PfExtra` (`C:\pf\extra`). Helper scripts of the kit itself are in `assets/`.
- An RDP session starts with Windows' client-area animations off, so PecoFence snaps where a
  normal desktop animates: switch them on in the steps when a test is about animation.
- pf's Explorer shows icons only, never thumbnails (IconsOnly=1): switch it off for a thumbnail
  test. The watchdog does not restart an instance started with `--test-script`: start the one a
  crash test kills without a script.

## Before/after animations

For a change a reviewer should see rather than read, record the same steps on the old build and on the
new one and compose them side by side:

- In the steps file: `. C:\pf\kit\rec.ps1`, then `RecStart 'U3-before' -X 200 -Y 100 -W 960 -H 540`
  (`-Fps 12` by default, up to about 30 on the EC2 boxes), `RecMark 'step text'` before every user step
  (the same marks in the same order on both builds), `RecMetric <number>` for values to chart (wait
  seconds, frame gap ms, MB), `RecStop`. Frames, pointer positions and button presses land in
  `shots/rec-<name>/`, which run.sh brings back.
- On the dev box: `uv run -q --with pillow python -I scripts/wintest/mkanim.py <spec.json>` writes
  `<out>.webm` and `<out>-sheet.jpg` (the spec format is in the script's header): the old build left
  with a red label, the new one right with a green one, a step caption, the pointer drawn in, a timer
  per side, highlight boxes, an optional metric chart, steps over 5 s fast-forwarded on both sides, and
  a result banner per side at the end. It uses Playwright's ffmpeg build (VP9) and the WenQuanYi font.
- Keep one idea per animation, 2 to 4 steps, about 20 s, plain words of what the user does and sees.
  Screen grabs load the box: timing effects (frame gaps, drag lag) look smaller while recording, so
  show them with a metric and slow motion, and check the sheet before using the animation.

## Hyper-V guests on the home PC (Windows 11 and Windows 10)

```bash
scripts/wintest/hyperv/run-hyperv.sh --vm PF-Win11-2 scripts/wintest/tests/186-search.ps1
scripts/wintest/hyperv/run-hyperv.sh --vm PF-Win10 --extra <dir> scripts/wintest/tests/186-install.ps1
```

`run-hyperv.sh` runs the same steps files in a Hyper-V guest of the Windows PC (`ssh win`, or
`WINTEST_HV_SSH`) for what a Server 2025 box cannot show: Windows 11's own shell and Windows 10.
It copies the kit, the build and `--extra` to a folder of its own under the PC's `%TEMP%`, starts
`hyperv/guest-run.ps1` there outside the ssh session (so a dropped connection does not end the
run), waits for it and prints the same table as `run.sh`. Run it as the background command itself.
The guest runner:

- needs the guest Off with no checkpoint (another session's checkpoint means it is in use);
- takes a checkpoint, sets the guest's video to 1600x900 (`Set-VMVideo`; the steps expect the EC2
  box's size), boots it and waits for pf's desktop;
- points the guest's PFJob task at a hidden launcher with normal priority, turns Spotlight off and
  removes the Store Notepad, which restores earlier jobs' tabs and shows "Cannot find" boxes over
  the Open dialog (`--keep-notepad` keeps it);
- runs one PFJob job per steps file, with the kit's `ctl.ps1 reset` before each, and packs
  `C:\pf\run` as `run.sh` does;
- turns the guest off, restores the checkpoint and the video settings and deletes the checkpoint.

All steps files take about 30 minutes in one guest; a Win11 and a Win10 guest can run at the same
time (keep at most four guests running across sessions). Where the guests differ from the EC2 box:
they are no Remote Desktop session (the feedback form's "remote desktop" fact), pf is an
administrator there (no UAC prompt for the Public Desktop), and Windows 10 keeps the tray icon in
the overflow window, which `TrayPick` does not open yet. Windows 10 also reports the desktop
coming to the front late or not at all now and then: a step that clicks the desktop and acts at
once can see that.

## The boxes

`box.sh create <name>` launches a box from `~/.config/pecofence-wintest/userdata.ps1` (OpenSSH
for the dev box key, a standard user `pf` that signs in by itself, the `PFJob` task, WebView2,
Chinese fonts, Windows Update off); it is ready about 20 minutes later. `box.sh up` starts a
stopped box and keeps an RDP session for pf open from a virtual X server, so pf's desktop is
1600x900 (`WINTEST_SIZE`); `box.sh down` stops both.

Private settings live outside the repo in `~/.config/pecofence-wintest/`: `config.sh` (pf's
password, AMI, subnet, security group), `boxes` (`name instance-id private-ip` per line), the ssh
key `pfwin` and `known_hosts`.

What a Server 2025 box does not cover: Windows 10, Windows 11 only shell features, a GPU (Fluent
corners), several monitors. Those still need the Hyper-V guests on the home PC.
