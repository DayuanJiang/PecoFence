# Vendored windows-webview

Based on Microsoft's `windows-webview` 0.100.0 from the windows-rs project.
The PecoFence patch adds the WebView2 composition controller (`src/composition.rs`: a web page
hung into a Windows.UI.Composition visual tree, with mouse input and the cursor forwarded by the
host window) and `WebView::try_suspend` / `resume`, which the upstream wrapper does not expose.

Upstream licensing is MIT OR Apache-2.0. Both original license texts are retained
in `license-mit` and `license-apache-2.0`.

The workspace selects this directory through `[patch.crates-io]`. Keep the changes
small and remove the patch when an upstream version provides the composition controller.
