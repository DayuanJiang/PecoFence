# PecoFence Windows installer

Run `pecofence-v<version>-x64-setup.exe` on Windows 10 22H2 or Windows 11.
The installer is unsigned. Microsoft Edge WebView2 Runtime is required for Settings;
the installer includes the loader DLL, but does not download the runtime.

Installation is for the current Windows user, without administrator privileges.
The default location is `%LOCALAPPDATA%\Programs\PecoFence`. A Start menu shortcut
is created; a desktop shortcut is optional. The installer does not add the CLI to
PATH and does not write a startup entry itself. **Start with Windows** is on by
default: PecoFence registers it when it runs, and the switch is in Settings. The
entry always points to the copy that ran last.

## Program files and user data

The wizard can choose a different installation folder. Keep the program files in
that folder together:

```text
<installation-directory>/                 // Default: %LOCALAPPDATA%/Programs/PecoFence
├── pecofence.exe                         // Desktop application
├── pecofence-watchdog.exe                // Restores desktop icons after an abnormal exit
├── pecofence-cli.exe                     // Command-line control
├── WebView2Loader.dll                    // Loader for the Settings browser
├── README.md                             // This installer edition guide
├── UPGRADING.md                          // Upgrade and migration instructions
├── SKILL.md                              // CLI instructions for AI agents
├── LICENSE                               // Project license
├── LICENSE-WebView2Loader.txt            // WebView2 Loader license
├── THIRD-PARTY-LICENSES.txt              // Dependency license notices
├── unins000.exe                          // Uninstaller created by Inno Setup
└── unins000.dat                          // Inno Setup uninstall records
```

Settings and working data live outside the program folder, in the same places as
the ZIP edition uses, so moving between the two keeps your fences, rules and
settings. Files appear as needed rather than all being created by Setup:

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
    ├── pecofence.prev.log                // Log of the previous run
    ├── crash-<instance>-<ticks>.dmp      // Created after a captured native crash
    ├── WebView2Profiles/
    │   └── default/                      // Settings browser data and cache
    └── icons-hidden.marker               // Present while desktop icons need restoration
```

Named instances use separate log/marker names and browser profile directories,
while sharing the configuration directory. Desktop items, portal targets and
user-chosen import/export paths remain at their external locations. Existing
pre-rename data is recognized; see [UPGRADING.md](UPGRADING.md).

## Updates and removal

To upgrade, exit PecoFence from its tray menu, then run the newer installer. Setup
reuses the registered installation folder and replaces the program files. To move
the installation to another folder, uninstall first. Numeric version downgrades are
blocked; prerelease suffixes are not ordered.

Setup also accepts a folder you previously extracted the ZIP into, such as
`%LOCALAPPDATA%\Programs\PecoFence`: it replaces the program files and leaves your
other files there alone. It refuses any other folder that already contains files.

Remove PecoFence through Windows Settings > Apps > Installed apps. Uninstall removes
the installed program files and shortcuts, and removes the startup entry only when
it points to this installed executable. Configuration, backups, logs and files you
added to the installation folder are kept.

**Settings > About > Check for updates** asks GitHub for the latest release. To
install it, PecoFence downloads the new setup and runs it; PecoFence closes meanwhile
and opens again when setup is done. You can
also run a newer setup from the Releases page yourself.
