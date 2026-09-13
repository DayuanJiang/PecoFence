# Third-party files

`webview2/WebView2Loader.x64.dll` is the x64 loader from the Microsoft.Web.WebView2
NuGet package. Its license and package metadata are included alongside it.

The loader must be distributed beside `pecofence.exe`. The larger WebView2 Runtime
is installed separately and is not part of this repository or portable ZIP.

Downloaded NuGet archives and extracted build files remain local and are ignored.
The patched Rust Composition wrapper and its licenses are under
`vendor/windows-composition`.
