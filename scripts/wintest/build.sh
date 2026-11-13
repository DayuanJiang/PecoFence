#!/bin/bash
# Builds the Windows binaries of this checkout on the dev box (cargo-xwin) into
# target/wintest-bin (release) or target/wintest-bin-debug (--debug); run.sh copies
# target/wintest-bin by default, give the other with --bin.
#   build.sh [--debug]   a debug build also takes the test-script `input` lines
#                        (PECOFENCE_UI_TEST_WINDOWS=1) and opens the settings page to CDP
#                        when started with PECOFENCE_WEBVIEW_ARGS=--remote-debugging-port=9352
# WINTEST_REPO=<checkout> builds another checkout (into its own target folder).
set -uo pipefail
REPO=${WINTEST_REPO:-$(cd "$(dirname "$0")/../.." && pwd)}
cd "$REPO" || exit 2
PROFILE=release; OUT=$REPO/target/wintest-bin
[ "${1:-}" = --debug ] && { PROFILE=debug; OUT=$REPO/target/wintest-bin-debug; }
# ~/.local/bin/cc on the dev box is not a compiler: host build scripts need gcc.
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/gcc CC_aarch64_unknown_linux_gnu=/usr/bin/gcc
export XWIN_ACCEPT_LICENSE=1
# The exe's icon and version resource, as on CI (tests/owner-notices.ps1 reads the file
# description): llvm-rc stands in for the Windows SDK's rc.exe.
[ -z "${PECOFENCE_RC:-}" ] && [ -x /usr/bin/llvm-rc ] && export PECOFENCE_RC=/usr/bin/llvm-rc
args=(); [ "$PROFILE" = release ] && args=(--release)
cargo xwin build "${args[@]}" --target x86_64-pc-windows-msvc -p pecofence -p pecofence-cli -p pecofence-watchdog 2>&1 \
  | grep -E '^error|Finished' | tail -5
rm -rf "$OUT"; mkdir -p "$OUT"
for e in pecofence pecofence-cli pecofence-watchdog; do
  cp "target/x86_64-pc-windows-msvc/$PROFILE/$e.exe" "$OUT/" || exit 2
done
cp third_party/webview2/WebView2Loader.x64.dll "$OUT/WebView2Loader.dll"
# The test shell extension of tests/shellext-fault.ps1 (run.sh --extra target/wintest-shellext).
cargo xwin build --release --target x86_64-pc-windows-msvc --manifest-path tools/shellext-fault/Cargo.toml \
  --target-dir target/shellext-fault 2>&1 | grep -E '^error' | tail -5
mkdir -p target/wintest-shellext
cp target/shellext-fault/x86_64-pc-windows-msvc/release/pf_shellext_fault.dll target/wintest-shellext/ || exit 2
echo "$PROFILE build in $OUT, pecofence.exe $(sha256sum "$OUT/pecofence.exe" | cut -c1-12)"
