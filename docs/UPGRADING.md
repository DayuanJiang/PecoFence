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
prerelease suffixes are not ordered by Setup.

## Updates from Settings

In a marked release package, open **Settings → About → Check for updates**.
The repository shown there comes from `release-info.json`; fork packages check
their own repository without editing source code. Missing or mismatched metadata
disables this feature instead of silently using the upstream repository. Source
builds, legacy unmarked ZIPs and named test instances do not install updates.
The MSIX edition offers a Microsoft Store link and never downloads GitHub packages.

1. Check for a newer public stable release and review its release notes.
2. Choose **Download update**. The matching setup EXE or portable ZIP, its size,
   `.sha256` sidecar and optional GitHub API digest must agree. A missing package,
   checksum or failed request is an error, not an up-to-date result.
3. Wait for verification (and portable extraction) to finish. A download at 100%
   is not yet ready to install. Completed verified downloads can be selected after
   restarting the same copy.
4. Choose **Install and restart**, then confirm. The app saves settings and exits
   normally, allowing desktop icons to be restored before the worker proceeds.
5. Installed copies open the interactive setup wizard. Portable copies replace
   only the twelve files shipped in the ZIP, retaining all other files. Successful
   updates restart PecoFence and open Settings.

There are no automatic update checks or downloads, private-repository credentials,
automatic installation, prerelease selection or downgrades. SHA-256 detects file
corruption; it is not a publisher signature. Only use packages from a repository
you trust. HTTPS certificates remain validated. Windows PowerShell 5.1 is required
for the worker; Rust, Python and Inno Setup are not required on user machines.

## Recovering after a crash or forced shutdown

Update state is stored beside portable data in `data/updates/<attempt-id>/`, or
under `%LOCALAPPDATA%/PecoFence/updates/<attempt-id>/` for installed copies. An
attempt keeps `plan.json`, `update-worker.ps1`, the verified downloaded package,
worker logs, and a durable transaction record. Portable attempts also keep
`backup/`, containing the original managed program files and their recorded hashes.
Configuration and other user data are not part of this program-file transaction.

- **Before file replacement:** a partial download or incomplete backup does not
  modify the running program. Check and download again.
- **During portable replacement or rollback:** the next launch offers **Restore
  previous version**. Recovery first checks every backup hash, then restores all
  original program files. It can resume after another interruption during recovery.
  Corrupt or missing backups stop recovery without deleting the remaining backups.
- **During Setup:** the next launch offers **Run setup again**. It verifies and
  re-runs the retained installer for the registered installation. It does not copy
  portable files over an Inno Setup installation or alter its uninstall records.

If PecoFence cannot launch, close any remaining PecoFence copies and use Windows
PowerShell to run the saved worker. Set `$attempt` to the full existing attempt
directory, not the example below:

```powershell
$attempt = 'D:\Apps\PecoFence\data\updates\<attempt-id>'
& "$attempt\update-worker.ps1" -Action Recover -Plan "$attempt\plan.json"
```

For an installed copy, choose its attempt below `%LOCALAPPDATA%\PecoFence\updates`
instead. The recovery command does not require the old process ID to still exist.
If local script policy blocks it, retain the attempt directory and use the manual
ZIP/Setup procedure above; do not weaken system policy to run the script.

Keep the original directory in place until update or recovery finishes. If a
portable directory was moved while recovery was pending, move the whole directory
back to its original location first. Completed updates have no pending transaction
and the portable folder can be moved normally.

The automated tests forcibly terminate an isolated worker at multiple durable
checkpoints and recover in a fresh process. This models lost process state after a
reboot; it does not certify recovery from hardware failure, filesystem damage or
storage devices that fail to persist flushed writes. Keep independent data backups.

## Automatic cleanup of update files

After the app has initialized and run for 30 seconds, local maintenance checks its
update directory. It also runs after update checks/downloads and at most hourly
while idle. This does not contact GitHub or install anything. It shares the update
lock, so it cannot delete files being used by an update or recovery worker.

- A completed installation is acknowledged only by a running app from the same
  directory, distribution and repository, at that version or newer. Its downloaded
  ZIP/Setup and extracted payloads are then removed. If the app cannot start, these
  files and recovery backups are retained.
- Keep the most recent verified portable backup for up to **30 days after startup
  acknowledgement**. A corrupt newer backup never evicts the last verified backup.
  Retained backups support inspection/manual repair; there is no post-update
  downgrade button.
- Keep at most **10 historical attempt records for 7 days**, plus the current
  maintenance record and the record accompanying a retained backup.
- Keep the selected verified download, or otherwise the newest reusable download,
  until it is installed or superseded. Remove duplicate, corrupt and abandoned
  downloads that never changed the live program files.
- Pending portable recovery and interrupted Setup are never removed automatically,
  regardless of age. Unknown files, links, damaged records and attempts belonging
  to another source or installation are retained. This can exceed the normal limits.
  In particular, records still naming a previous portable-folder location are
  retained when ownership cannot be confirmed after a move.

Deletion first renames an obsolete attempt to `.cleanup-<attempt-id>` under the
same updates directory. If cleanup is interrupted, the next maintenance run can
finish it from its saved ownership record. Only known updater files are removed;
configuration, configuration backups, application logs, WebView2 profiles and
user-added files are not cleanup targets. Inspect the application log for cleanup
results. Close the app before manually removing retained diagnostic attempts, and
never remove an attempt whose update or recovery is still pending.

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
