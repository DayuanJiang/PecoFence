# Upgrading to PecoFence

PecoFence is the new name of this project. The executable is now `pecofence.exe`,
with `pecofence-watchdog.exe` beside it.

## Existing installations

Exit the older openFence application before starting PecoFence. Both names share
a compatibility instance lock so two versions cannot manage the same desktop at once.

New installations save configuration in `%APPDATA%\PecoFence`. If that location
has no configuration or backups and `%APPDATA%\OpenFence` contains an existing
installation, PecoFence continues using the old directory **in place**. Nothing
is copied or rewritten simply to change the product name. Existing PecoFence data
always takes priority.

This preserves fence layouts, rules, language preferences, snapshots and backups.
Filenames and custom fence/rule names are not renamed. Portable mode continues
using the `config` directory beside the executable.

New logs and WebView2 profiles use `%LOCALAPPDATA%\PecoFence`. An outstanding
desktop-icon recovery marker from the older name is recognized.

## Switching between the ZIP and the installer

Releases offer `pecofence-v<version>-x64-portable.zip` and
`pecofence-v<version>-x64-setup.exe`, with the same program inside. Both keep
settings in `%APPDATA%\PecoFence`, so switching keeps your fences, rules and
settings. Exit PecoFence from its tray menu first.

- **ZIP to installer:** run the setup EXE. It can install into the folder you
  extracted the ZIP into and replaces the program files there. If you install
  somewhere else, you can delete the old ZIP folder afterwards.
- **Installer to ZIP:** uninstall through Windows Settings (your data is kept),
  then extract the ZIP and run `pecofence.exe`.
- If you started the ZIP with `--portable`, its configuration is in the `config`
  folder beside the executable, which other copies do not read. Use **Export or
  import configuration** in Settings to carry it over.

## Going back to 0.1.3 or earlier

Versions after 0.1.3 know a monitor by its model and connection. 0.1.3 and
earlier do not, so with a configuration a newer version has saved they show no
fences. Nothing is lost: the fences come back as soon as the newer version runs
again. To stay on the older version, restore a daily backup from before the
upgrade in Settings → 布局与备份 (the files are in the `backups` folder next to
`config.json`).

## Going back to 0.2.3 from 0.2.4

0.2.3 reads a configuration 0.2.4 has saved, but it does not know the options
0.2.4 added, and its first save drops them: 像桌面一样排列, icon spacing between
the presets, 图标对齐 居中, 文件夹优先, 向上展开, 为空时自动隐藏 on other
fences, opening folders inside a fence, the custom theme, the temporary Windows
desktop shortcut and keeping the Windows icons hidden on exit. They are off when
you come back to 0.2.4.

- The 「桌面」 fence arranged like the desktop and an empty fence that hid itself
  show in 0.2.3 as ordinary fences at their saved place, and 0.2.3 moves fences
  that then overlap them. A fence that opened upward shows its title row at the
  top of its rect.
- Icons in fences with hidden labels show squeezed labels; a default icon size
  other than 32, 48, 64 or 96 makes 0.2.3's `pecofence-cli settings set` fail
  until it is set back to one of those.
- With 「退出 PecoFence 时保持 Windows 桌面图标隐藏」 on, the desktop shows no
  icons while 0.2.3 is not running: turn it off before going back, or choose
  View → Show desktop icons on the desktop.

To get everything back when you return to 0.2.4, restore the daily backup from
before the downgrade in Settings → 布局与备份 (the files are in the `backups`
folder next to `config.json`), or save a snapshot before going back.

## Going back to 0.2.5

0.2.5 reads a configuration a later version has saved, but it does not know the
小工具 (tools) added after it. It shows note, clock and script fences as empty
ordinary fences at their place, and its first save drops what belongs to the tools:

- the text of every note, and which script each script fence runs and how often
  (the script files themselves stay where they are);
- the 小工具 switch on the 小工具 page; on the 速览与搜索 page, 对话框跳转,
  速览搜索也找整台电脑（Everything） and 双击 Ctrl as Peek's shortcut; and the search
  history that puts what you opened from the Peek search box higher in the list.

The fences stay as ordinary fences, in the snapshots too, so restoring a snapshot
does not bring the tools back. The daily backups still have them: when you return
to the later version, restore the daily backup from before the downgrade in
Settings → 布局与备份 (the files are in the `backups` folder next to `config.json`).
That folder keeps the last seven backups and 0.2.5 adds one for each day it saves,
so restore within a week.

A daily backup is made at the first save of the day, so it does not have what you
changed later that day: a note written on the day you go back is in no daily
backup. Before going back, export the configuration with 导出 / 导入配置文件 and
import that file when you return (an import keeps 脚本栅栏 off until you turn it on
again); such a file also does not age out. Without an export, the text may still be
in `config.bak` next to `config.json` while 0.2.5 has saved only once: copy that
file somewhere else before you start the later version again (its first save
replaces `config.bak`), then import the copy.

## Going back to 0.3.1

0.3.1 reads a configuration 0.3.2 has saved, but it does not know what 0.3.2 added, and
its first save drops it:

- A 智能栅栏 shows as an empty ordinary fence at its place, and its search is dropped.
  Back in 0.3.2 it stays an ordinary fence.
- Fences set to show on one virtual desktop show on every desktop, and which desktop
  they belong to is dropped, together with the switch 每个虚拟桌面各用一套栅栏.
- 自由摆放 on the 「桌面」 fence is off, and the places of its icons are dropped.

0.3.1 keeps no copy of the newer file (only later versions do, see below). To get these
back when you return to 0.3.2, restore the daily backup from before the downgrade in
Settings → 布局与备份, or export the configuration with 导出 / 导入配置文件 before going
back and import that file when you return.

## Going back to an earlier version from a later one

Versions after 0.3.1 record at every save which version made it (`savedBy` in
`config.json`). When such a version opens a file a later version saved, it first keeps
a copy in the `backups` folder, named for the day and the later version
(`2026-12-01-0.4.0.json`), because its own first save drops what only the later
version knows. When you return to the later version, restore that copy in
Settings → 布局与备份. Like the daily backups, it ages out after seven newer ones.

## Windows startup

A normal release launch registers the `PecoFence` startup entry when required.
The legacy `openFence` entry is removed only after a replacement was registered
successfully, or when startup is disabled. When startup is on, the running release
points the entry at its own executable, so the copy that ran last starts with Windows.

Portable and development launches do not change startup entries. Changing the
autostart switch in Settings remains an explicit opt-in/out.

## Environment overrides

Use the `PECOFENCE_` prefix for application overrides, for example `PECOFENCE_INSTANCE` and
`PECOFENCE_ACRYLIC`. Existing `OPENFENCE_` overrides are still accepted; when both
are defined, the `PECOFENCE_` value wins.
