# Microsoft Store (MSIX)

The Store build is the portable build wrapped in an MSIX package. The Store signs the
package after certification, so no code-signing certificate is needed.

## Files

- `packaging/msix/AppxManifest.xml`: manifest template. `runFullTrust` (desktop hooks,
  icon host, tray), Windows 11 minimum (`10.0.22000.0`), x64, a `windows.startupTask`
  for run-at-logon, tile assets under `Assets\`.
- `packaging/msix/identity.json`: the identity Partner Center assigned
  (Product management > Product identity). These values are not secret and must match
  the manifest exactly or the upload is rejected.
- `scripts/make-msix-assets.py`: renders every Store/tile PNG from `site/assets/mark.svg` in
  the app-icon style (white mark on the indigo `#4768DE` rounded plate, the manifest
  `BackgroundColor`); `--preview .cache/icon-sheet.png` writes a contact sheet.
  `scripts/make-app-icon.py` renders the same icon into `crates/app/assets/pecofence.ico`,
  which `crates/app/build.rs` embeds into both executables.
- `scripts/make-msix.ps1`: builds, stages, indexes resources (MakePri) and packs
  (MakeAppx) into `dist/pecofence-<version>-x64.msix`.

## Build

```powershell
./scripts/make-msix.ps1              # build + pack, unsigned (upload this)
./scripts/make-msix.ps1 -SkipBuild   # reuse target/package/release
./scripts/make-msix.ps1 -TestSign    # also writes a self-signed copy for local installs
```

Requires the Windows SDK (`winget install Microsoft.WindowsSDK.10.0.26100`) and Python
with Pillow. The package version is `<Cargo.toml version>.0`; the Store requires the
fourth part to be 0.

Local install test: import `dist/pecofence-test-signing.cer` into
`Cert:\LocalMachine\TrustedPeople` (administrator), then
`Add-AppxPackage dist/pecofence-<version>-x64-testsigned.msix`. Exit the running
PecoFence first; both builds share the single-instance mutex.

## Certification notes

- Submission 1 (2026-09-14) failed policy 10.2.4.1 because the binaries imported
  `VCRUNTIME140.dll` from the Visual C++ Redistributable. `.cargo/config.toml` now links
  the MSVC runtime statically (`+crt-static`); verify with a dependency scan before
  uploading that only `api-ms-win-crt-*` (Universal CRT, part of Windows) and system DLLs
  remain.

## Behavior differences in the packaged build

- Autostart: HKCU writes are virtualized inside the package, so the Run-key code is
  skipped (`process::is_packaged()`). The manifest startup task is enabled by default and
  users manage it under Settings > Apps > Startup; the in-app toggle opens that page.
- Configuration: `%APPDATA%\PecoFence\config.json` is written into the package's
  virtualized AppData (`%LOCALAPPDATA%\Packages\DayuanJiang.PecoFence_0bme5nfnaj27p\`).
  An existing portable config is read on first launch but changes stay in the package.
- Uninstall removes the virtualized config with the package.

## Submission checklist (Partner Center)

1. Pricing and availability: free (base price 0 USD), all markets.
2. Properties: category Productivity, no personal data collected (no privacy policy
   URL), display mode PC, support URL and website.
3. Age ratings: IARC questionnaire, utility with no user-generated or online content.
4. Packages: upload `dist/pecofence-<version>-x64.msix`.
5. Store listings: en-US, zh-CN, ja. Screenshots are the demo-desktop captures under
   `extras/pecofence-promo/public/` (2560×1440 PNG, no overlaid text).
6. Submission options: certification notes explaining that the app hides the real
   desktop icons by design and restores them on exit (Restore Windows desktop icons in
   the tray menu).

Store ID `9MV6WG3XNWSX`; listing URL <https://apps.microsoft.com/detail/9MV6WG3XNWSX>.
