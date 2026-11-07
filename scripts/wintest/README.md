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
- The user's own copy (user.ps1): `Ipf` / `IpfJson` / `UserLog` for the installed copy (set
  `$script:CliExe` for a portable one), `StartUser`, `WaitNoPf`; `PfUseUser` makes the item and
  state helpers read that copy; setup and uninstall by their buttons (`Wizard`, `Uninstall`,
  `SilentSetup`, `SilentUninstall`, `ClickButton`, `SetupBoxes`), the tray (`TrayIconRect`,
  `TrayMenu`, `TrayPick`), `CloseNotice`. `tests/holdout-*.ps1` show all of them in use.
- At the end of every job the app logs are checked for ERROR and panic lines (set
  `$PfAllowLogErrors = $true` when a test causes them on purpose); a job without any app log
  says so in `job.log` instead.
- Each job starts with fresh PecoFence data and ends Edge, Notepad and setup programs, but the
  desktop's files and an installed PecoFence stay: steps remove what they create and install.
- Files the steps need (installers, test programs) go in a folder given with `--extra`; on the box
  they are in `$PfExtra` (`C:\pf\extra`). Helper scripts of the kit itself are in `assets/`.
- An RDP session starts with Windows' client-area animations off, so PecoFence snaps where a
  normal desktop animates: switch them on in the steps when a test is about animation.
- pf's Explorer shows icons only, never thumbnails (IconsOnly=1): switch it off for a thumbnail
  test. The watchdog does not restart an instance started with `--test-script`: start the one a
  crash test kills without a script.

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
