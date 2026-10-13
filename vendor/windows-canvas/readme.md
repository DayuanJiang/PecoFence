Vendored windows-canvas

Based on Microsoft's `windows-canvas` 0.100.0 from the windows-rs project.
The PecoFence patch adds `TextFormat::with_locale`: upstream creates every text format
with the locale `en-us`, and DirectWrite then fills Han characters from the Japanese UI
font, so Chinese file names in fences showed Japanese glyph forms mixed with YaHei.

Upstream licensing is MIT OR Apache-2.0. Both original license texts are retained
in `license-mit` and `license-apache-2.0`.

The workspace selects this directory through `[patch.crates-io]`. Keep the changes
small and remove the patch when an upstream version lets callers choose the locale.
