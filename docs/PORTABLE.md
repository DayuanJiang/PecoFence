# PecoFence portable edition

Before replacing an older ZIP, read the included [upgrade guide](UPGRADING.md).
Older ZIPs could use AppData; this distribution does not automatically adopt it.

Extract the complete ZIP and run `pecofence.exe`. Keep the program files together
as shown below; moving only the executable is not enough.

Windows 11 22H2 or later and Microsoft Edge WebView2 Runtime are required.

Right-click the tray icon to open Settings or exit. Under General, choose your
display language: English, Simplified/Traditional Chinese, Japanese, Korean,
German, French, Spanish, Portuguese (Brazil), Russian or Follow system.

## Program files and user data

The ZIP contains these program files in the extraction folder:

```text
<portable-directory>/                     // Complete ZIP extraction folder
├── pecofence.exe                         // Desktop application
├── pecofence-watchdog.exe                // Restores desktop icons after an abnormal exit
├── pecofence-cli.exe                     // Command-line control
├── WebView2Loader.dll                    // Loader for the Settings browser
├── deployment.json                       // Selects portable mode; do not edit or remove
├── release-info.json                     // Package version and release repository
├── README.md                             // This portable edition guide
├── UPGRADING.md                          // Upgrade and migration instructions
├── SKILL.md                              // CLI instructions for AI agents
├── LICENSE                               // Project license
├── LICENSE-WebView2Loader.txt            // WebView2 Loader license
└── THIRD-PARTY-LICENSES.txt              // Dependency license notices
```

Double-clicking the app automatically keeps its settings and working data in the
same folder. The following directories and files are created as needed at runtime;
they are not included in the ZIP:

```text
<portable-directory>/
├── config/
│   ├── config.json                       // Settings, layouts, rules and snapshots
│   ├── config.bak                        // Previous saved configuration
│   └── backups/
│       └── YYYY-MM-DD.json               // Daily configuration backups
└── data/
    ├── logs/
    │   └── pecofence.log                 // Application log
    ├── crashes/
    │   └── crash-<instance>-<ticks>.dmp  // Created after a captured native crash
    ├── WebView2Profiles/
    │   └── default/                      // Settings browser data and cache
    ├── updates/                          // Update packages, workers and recovery backups
    └── recovery/
        └── icons-hidden.marker           // Present while desktop icons need restoration
```

The tree shows the default instance. Named instances use separate log/marker names
and browser profile directories, while sharing this copy's `config/` directory.
Configuration writes can also create temporary or recovery files inside `config/`.

The folder must be writable. A storage failure displays an error; the app never
falls back to AppData or Temp. It does not read installed or pre-rename OpenFence
data. Windows startup is disabled for this edition, and its Settings/CLI cannot
change the installed edition's startup entries. `--portable` remains accepted
for compatibility but is unnecessary for this ZIP.

Exit the app and wait for its watchdog/browser processes to finish before moving
the whole folder. Its own data paths follow the folder's new location; desktop
items, portal targets and user-chosen import/export paths remain external.

## Using PecoFence

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
`release-info.json` selects the version and repository used by **Settings → About →
Check for updates**. Checks and downloads run only when requested. After verifying
the download, **Install and restart** asks for confirmation, saves settings and
closes the app before replacing the packaged program files. `config/`, `data/`
and unrelated files are retained. Updates never switch this copy to the installer.

Windows PowerShell 5.1 (included with Windows) runs the update worker. If policy
blocks it, use the manual ZIP upgrade procedure in [UPGRADING.md](UPGRADING.md).
That guide also explains recovery after an interrupted update. Do not move the
folder or delete `data/updates/` while an update or recovery is pending.
