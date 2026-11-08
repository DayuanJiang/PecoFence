#!/bin/bash
# Runs the workspace's unit tests on the Windows test box, in pf's desktop session (the
# desktop anchor tests fail in an ssh session).
#   unit.sh [run.sh options]
# Tests that read source files find them through CARGO_MANIFEST_DIR, a dev box path that
# Windows reads as C:\home\...: the job unpacks crates/, ui/, locales/ and the PC status
# test script there first.
# WINTEST_REPO=<checkout> tests another checkout.
set -uo pipefail
KIT=$(cd "$(dirname "$0")" && pwd)
REPO=${WINTEST_REPO:-$(cd "$KIT/../.." && pwd)}
cd "$REPO" || exit 2
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/gcc CC_aarch64_unknown_linux_gnu=/usr/bin/gcc
export XWIN_ACCEPT_LICENSE=1
EX=$REPO/target/wintest-unit
rm -rf "$EX"; mkdir -p "$EX/exe"
cargo xwin test --no-run --target x86_64-pc-windows-msvc --workspace --message-format=json 2>/dev/null \
  | python3 -I -c "
import json, shutil, sys
for line in sys.stdin:
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get('reason') == 'compiler-artifact' and m.get('executable') and m.get('profile', {}).get('test'):
        shutil.copy(m['executable'], sys.argv[1] + '/' + m['target']['name'] + '-' + m['target']['kind'][0] + '.exe')
" "$EX/exe"
n=$(ls "$EX/exe" | wc -l)
[ "$n" -gt 0 ] || { echo "no test executables built"; exit 2; }
cp third_party/webview2/WebView2Loader.x64.dll "$EX/exe/WebView2Loader.dll"
tar -cf "$EX/src.tar" crates ui locales scripts/test-pc-status.ps1 2>/dev/null || tar -cf "$EX/src.tar" crates locales
echo "$REPO" > "$EX/repo-path.txt"
echo "$n test executables"
exec "$KIT/run.sh" --extra "$EX" "$@" "$KIT/tests/unit.ps1"
