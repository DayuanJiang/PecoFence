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

- **Run `run.sh` itself as the background command** when the round takes longer than a few
  minutes (Claude Code: `run_in_background`). It returns when every job is done or timed out
  (`--timeout`, default 900 s per job), so the completion notice arrives. Never start a separate
  loop that waits for its output: when such a loop stalls, nobody notices the finished results.
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
  check saves `FAIL-NN.png`. An error in a line becomes a failed check and the next line runs.
- `PfStart` starts the build under test as a portable instance (own config, own log; `-Script`
  gives test-script lines, `-Lang`), `PfQuit` ends it like a sign-out, `Pf` / `PfJson` run
  `pecofence-cli` against it, `PfFence`, `PfWaitLog`, `PfLogErrors`.
- Input and windows: `[G]::Click/RClick/MoveTo/Type/Press/KeyDown/KeyUp`, `Chord`, `PeekKey`,
  `DoubleCtrl`, `TitleClick`, `WaitWin`, `Rect`, `Fg`, `[L]::Text/Cls/Tops/MoveWindow`,
  `[M]::BoxText()` (the Peek search box), `[QS]::Address/FileName/SetFileName` (file dialogs),
  `Explorers`, `UiaFind`, `CloseWindows`, `Shot`.
- At the end of every job the app log is checked for ERROR and panic lines (set
  `$PfAllowLogErrors = $true` when a test causes them on purpose).
- Files the steps need (installers, test programs) go in a folder given with `--extra`; on the box
  they are in `$PfExtra` (`C:\pf\extra`). Helper scripts of the kit itself are in `assets/`.

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
