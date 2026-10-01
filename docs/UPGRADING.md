# Upgrading to PecoFence

PecoFence is the new name of this project. The executable is now `pecofence.exe`,
with `pecofence-watchdog.exe` beside it.

## Choosing a distribution

New releases offer `pecofence-v<version>-x64-portable.zip` and
`pecofence-v<version>-x64-setup.exe`, each with a `.sha256` file. Both contain the
same release executables. Their `deployment.json` selects different data paths.
The Microsoft Store/MSIX edition continues to use package identity.

- **Installer:** exit PecoFence, then run the newer setup EXE. It keeps the existing
  installation directory and AppData. Uninstall retains user data and only removes
  a startup value that points to its own executable.
- **Portable:** exit PecoFence and its helpers, back up the whole folder, then
  extract the complete newer ZIP into it. Keep `config/` and `data/`; neither is
  shipped in the ZIP. Do not extract a portable ZIP over an installer-managed copy.
- **Older ZIP that used AppData:** use the installer to retain that behavior, or
  export the configuration from the old app and explicitly import it into the
  new portable copy. The portable copy never imports AppData automatically.
  Keep the original AppData as a backup; exports do not include its entire backup
  history, logs or browser cache.
- **Older ZIP used with `--portable`:** keep its local `config/` directory. New
  logs, crash dumps, browser profiles and recovery markers now stay under `data/`.
  Exit the old app normally first so it can restore desktop icons and clear its
  older recovery marker.

Setup cannot convert a portable/nonempty unrelated folder in place. Its generated
installed marker must remain unchanged. Numeric version downgrades are blocked;
prerelease suffixes are not ordered. There is no in-app GitHub update downloader.

## Existing installed and pre-rename data

Exit the older openFence application before starting PecoFence. Both names share
a compatibility instance lock so two versions cannot manage the same desktop at once.

New installations save configuration in `%APPDATA%\PecoFence`. If that location
has no configuration or backups and `%APPDATA%\OpenFence` contains an existing
installation, PecoFence continues using the old directory **in place**. Nothing
is copied or rewritten simply to change the product name. Existing PecoFence data
always takes priority. This compatibility applies to installed, unmarked and MSIX
copies, not to the new portable distribution.

This preserves fence layouts, rules, language preferences, snapshots and backups.
Filenames and custom fence/rule names are not renamed. Portable mode continues
using the `config` directory beside the executable.

New logs and WebView2 profiles use `%LOCALAPPDATA%\PecoFence`. An outstanding
desktop-icon recovery marker from the older name is recognized.

## Windows startup

A normal release launch registers the `PecoFence` startup entry when required.
The legacy `openFence` entry is removed only after a replacement was registered
successfully, or when startup is disabled. When startup is enabled, the running
non-portable release updates the entry to point to its own executable.

Portable launches do not change startup entries and cannot enable startup through
Settings, IPC or configuration import. Development launches skip automatic startup
reconciliation. The Store edition continues to use its MSIX startup task.

## Environment overrides

Use the `PECOFENCE_` prefix for application overrides, for example `PECOFENCE_INSTANCE` and
`PECOFENCE_ACRYLIC`. Existing `OPENFENCE_` overrides are still accepted; when both
are defined, the `PECOFENCE_` value wins.
