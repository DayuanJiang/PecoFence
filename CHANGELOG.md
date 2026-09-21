# Changelog

## Unreleased

- Start with Windows now follows the release you launch: an entry still pointing at an older copy (for example one left in Downloads) is re-pointed to the running executable.

## 0.0.5

- Fence corners are 8 DIP in both materials (Fluent was 4, Liquid Glass 24), matching Windows 11 windows. Title-bar controls follow.
- Settings: the Liquid Glass switch is 44×24 instead of the oversized 54×30.

## 0.0.4

- Fence context menu → Sort → "Group by date": items are shown under Today / Yesterday / This week / This month / Earlier headers in the icon, list and details layouts.
- Settings → Organizing rules → Quick add: one click creates a fence and its rule for Images, Music, Videos, Archives, Installers, or "To clean up" (installers and archives unused for 30 days; gathered, never deleted).
- New rule condition "idle days": the item was neither modified nor opened from a fence for N days. Rules using it are re-run hourly.
- New file type category "Installers": .msi/.msix/.appx packages and setup/install-named .exe files.
- Consistent app icon: Store tiles, exe icon, taskbar icon, tray icon and the settings window share the white-on-indigo mark.

## 0.0.2

- While Windows desktop icons are hidden, the special desktop items the user has enabled in Windows (Recycle Bin, This PC, User's Files, Network, Control Panel) appear in the Desktop fence: double-click opens them, the shell context menu works (Empty Recycle Bin, Properties), files dropped on the Recycle Bin are recycled, and the Recycle Bin icon follows its contents (also when files are recycled from elsewhere).
- Version bump for the Microsoft Store resubmission; package identity requires a new version per upload.
- Executables link the C runtime statically; the Visual C++ Redistributable is no longer required (Store policy 10.2.4.1).
- License changed from MIT to the Apache License 2.0; a NOTICE file accompanies the LICENSE.

## 0.0.1

- Renamed the product, executables and packages to PecoFence; existing configuration
  directories, environment overrides and startup entries are handled compatibly.
- Desktop fences, folder portals, tab groups, Explorer file operations and sorting.
- Quick Hide, Peek, Fluent and Liquid Glass themes.
- Automatic organizing rules, layout snapshots, configuration import/export and backups.
- Ten offline interface languages with live switching and Windows display-language detection.
- README and hero artwork in all ten languages under `docs/readme/`.
- Static product website in `site/`, built by `scripts/build-site.py` and published with GitHub Pages.
- Shared native/settings translation catalogs and translation validation.
- Portable x64 packaging, clean source export and draft GitHub Release workflow.
- Portable startup preserves the installed copy's Windows autostart registration.
- Independent instances use separate WebView2 profiles.

This is an initial release. Full compatibility testing across older Windows 11
versions and different graphics/display configurations is still in progress.
