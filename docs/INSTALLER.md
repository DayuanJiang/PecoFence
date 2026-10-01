# PecoFence Windows installer

Run `pecofence-v<version>-x64-setup.exe` on Windows 11 22H2 or later.
The installer is unsigned. Microsoft Edge WebView2 Runtime is required for Settings;
the installer includes the loader DLL, but does not download the runtime.

Installation is for the current Windows user, without administrator privileges.
The default location is `%LOCALAPPDATA%\Programs\PecoFence`. A Start menu shortcut
is created; the desktop shortcut and launching the app at the end are optional.
The installer does not add the CLI to PATH or enable Windows startup. The app's
existing Settings control manages startup when you run the installed release.

## Program files and user data

The wizard can choose a different installation folder. Keep the program files in
that folder together:

```text
<installation-directory>/                 // Default: %LOCALAPPDATA%/Programs/PecoFence
├── pecofence.exe                         // Desktop application
├── pecofence-watchdog.exe                // Restores desktop icons after an abnormal exit
├── pecofence-cli.exe                     // Command-line control
├── WebView2Loader.dll                    // Loader for the Settings browser
├── deployment.json                       // Selects installed mode; do not edit or remove
├── release-info.json                     // Package version and release repository
├── README.md                             // This installer edition guide
├── UPGRADING.md                          // Upgrade and migration instructions
├── SKILL.md                              // CLI instructions for AI agents
├── LICENSE                               // Project license
├── LICENSE-WebView2Loader.txt            // WebView2 Loader license
├── THIRD-PARTY-LICENSES.txt              // Dependency license notices
├── unins000.exe                          // Uninstaller created by Inno Setup
└── unins000.dat                          // Inno Setup uninstall records
```

The application creates its settings and working data separately from the program
files. For a new installation, the default instance uses these locations; files
appear as needed rather than all being created by Setup:

```text
%APPDATA%/
└── PecoFence/
    ├── config.json                       // Settings, layouts, rules and snapshots
    ├── config.bak                        // Previous saved configuration
    └── backups/
        └── YYYY-MM-DD.json               // Daily configuration backups

%LOCALAPPDATA%/
└── PecoFence/                            // Separate from Programs/PecoFence
    ├── pecofence.log                     // Application log
    ├── crash-<instance>-<ticks>.dmp      // Created after a captured native crash
    ├── WebView2Profiles/
    │   └── default/                      // Settings browser data and cache
    ├── updates/                          // Verified setup downloads and update state
    └── icons-hidden.marker               // Present while desktop icons need restoration
```

Named instances use separate log/marker names and browser profile directories,
while sharing the configuration directory. Configuration writes can also create
temporary or recovery files beside `config.json`. Desktop items, portal targets
and user-chosen import/export paths remain at their external locations.

Keep `deployment.json` beside the executables and do not edit it. It selects the
installed data layout: `%APPDATA%\PecoFence` for configuration and backups,
`%LOCALAPPDATA%\PecoFence` for logs, dumps, WebView2 data and desktop recovery state.
Existing pre-rename data is recognized; see [UPGRADING.md](UPGRADING.md).

## Updates and removal

To upgrade, exit PecoFence from its tray menu, wait for its helper processes to
finish, then run the newer installer. Setup reuses the registered installation
directory and updates its files. It refuses to overwrite a portable or unrelated
nonempty folder. To change the installation directory, uninstall first.
Numeric version downgrades are blocked; prerelease suffixes are not ordered.
Upgrades validate and retain the existing `deployment.json`; they do not rewrite
this fixed distribution identity while replacing the versioned program files.

Remove PecoFence through Windows Settings > Apps > Installed apps. Uninstall removes
its installed files and shortcuts, and removes a startup entry only when it points
exactly to this installed executable. Configuration, backups, logs and other user
data are retained. Files you added to the installation folder are also retained.

**Settings → About → Check for updates** checks the repository recorded in
`release-info.json`. Downloading and installing are separate user actions.
**Install and restart** asks for confirmation, saves settings, closes the app,
then opens the verified setup EXE at the registered installation location.
The wizard stays interactive. No silent or scheduled installation is performed.

The updater requires Windows PowerShell 5.1, included with Windows. If an update
is interrupted, the verified installer is retained under
`%LOCALAPPDATA%\PecoFence\updates\<attempt-id>`. The next launch offers to run
setup again. Installed copies are repaired through Setup, never by restoring
portable program files. See [UPGRADING.md](UPGRADING.md) for recovery when the app
cannot start. The Microsoft Store edition continues using Store updates.
