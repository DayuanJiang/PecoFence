# PecoFence portable edition

Upgrading from the previous name? Exit the old application first and keep your
existing `config` directory. See the included [upgrade guide](UPGRADING.md).

Extract the complete ZIP and run `pecofence.exe`. Keep these files together:

- `pecofence.exe`
- `pecofence-watchdog.exe`
- `WebView2Loader.dll`
- `pecofence-cli.exe` (optional command-line control, see below)

Windows 11 x64 and Microsoft Edge WebView2 Runtime are required.

Right-click the tray icon to open Settings or exit. Under General, choose your
display language: English, Simplified/Traditional Chinese, Japanese, Korean,
German, French, Spanish, Portuguese (Brazil), Russian or Follow system.

By default, settings are saved in `%APPDATA%\PecoFence\config.json`. Launch with
`--portable` to use a `config` folder beside the executable.
Portable startup does not synchronize Windows' autostart entry; changing the
autostart toggle in Settings remains an explicit opt-in/out.

Double-click empty desktop space to hide/show fences. **Ctrl+Alt+Space** brings
them above other windows. Drag a title to move a fence; double-click it to roll up.

Automatic organizing rules only change group membership. File operations you
initiate—moving, renaming, copying and deleting—operate on real files.

Exit restores Windows desktop icons. If needed, use **Restore Windows desktop
icons** from the tray menu or Settings → About.

`pecofence-cli.exe` lets a terminal or an AI coding agent configure the running app:
`pecofence-cli fence list`, `pecofence-cli describe`. Run it from this folder or add the
folder to PATH. `SKILL.md` describes the tool for agents such as Claude Code or Codex.

The ZIP is an unsigned portable build. It does not contain your configuration.
Languages work offline; the glass background uses static desktop wallpaper.
