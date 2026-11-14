#!/bin/bash
# The release memory check: two builds on the home PC (real GPU, its 4K monitor, the user's
# own fences with Liquid Glass and the 电脑状态 fence, plus a clock with seconds), each run in
# turn (A B A B) as a portable test instance for --minutes, sampled every 30 s. Prints what
# Task Manager shows for the PecoFence group (with the processes it started), the private
# commit, the GPU memory, threads, handles and CPU time, median and maximum per build.
#   pc-memory.sh <build A dir> <build B dir> [--minutes 30] [--order a,b,a,b] [--out DIR]
# A build dir holds pecofence.exe, pecofence-cli.exe and WebView2Loader.dll (a release zip
# unpacked, or target/wintest-bin). Run it as the background command: about 2 hours for the
# default order. The user's own PecoFence keeps running; the test copy has autostart, hotkeys,
# hooks, the tray icon, update checks and usage uploads off, and hides no desktop icons.
set -uo pipefail
A=${1:?build A dir}; B=${2:?build B dir}; shift 2
MINUTES=30; ORDER=a,b,a,b; OUT=
while [ $# -gt 0 ]; do
  case $1 in
    --minutes) MINUTES=$2; shift 2 ;;
    --order) ORDER=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    *) echo "unknown option $1"; exit 2 ;;
  esac
done
KIT=$(cd "$(dirname "$0")" && pwd)
OUT=${OUT:-$(mktemp -d "${TMPDIR:-/tmp}/pc-memory.XXXX")}
mkdir -p "$OUT"
ROOT='C:\Users\me\pfmem-check'
RROOT=pfmem-check
ssh win "exit" || { echo "the PC does not answer (ssh win)"; exit 2; }

# The user's configuration, made safe to run next to their own copy.
CFG=$(ssh win 'if (Test-Path $env:APPDATA\PecoFence\config.json) { "PecoFence" } else { "OpenFence" }' | tr -d '\r')
scp -q "win:AppData/Roaming/$CFG/config.json" "$OUT/user-config.json" || exit 2
python3 -I - "$OUT/user-config.json" "$OUT/config.json" <<'EOF'
import json, sys
c = json.load(open(sys.argv[1], encoding='utf-8-sig'))
s = c['settings']
s.update(autostart=False, autoCheckUpdates=False, usageStats=False, hideRealIcons=False,
         showTrayIcon=False, featureTips=False, offerNewFence=False, windowsDesktopHotkey=None)
s['quickHide']['enabled'] = False
s['peek'].update(enabled=False, doubleCtrl=False)
s['tools'].update(enabled=True, dialogJump=False, clockSeconds=True)
json.dump(c, open(sys.argv[2], 'w', encoding='utf-8'), ensure_ascii=False)
EOF
rm -f "$OUT/user-config.json"

ssh win "Remove-Item -Recurse -Force $ROOT -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force $ROOT\\a, $ROOT\\b | Out-Null" >/dev/null
scp -q "$A"/pecofence.exe "$A"/pecofence-cli.exe "$A"/WebView2Loader.dll "win:$RROOT/a/" || exit 2
scp -q "$B"/pecofence.exe "$B"/pecofence-cli.exe "$B"/WebView2Loader.dll "win:$RROOT/b/" || exit 2
scp -q "$OUT/config.json" "$KIT/pc-memory.ps1" "win:$RROOT/" || exit 2

# Started by a task in the signed-in session at normal priority (a priority-7 task would give
# the app a low memory priority, and Windows then trims it: numbers that look too good).
TASK=$(cat <<EOF
\$a = New-ScheduledTaskAction -Execute 'conhost.exe' -Argument '--headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File $ROOT\\pc-memory.ps1 -Root $ROOT -Order $ORDER -Minutes $MINUTES'
\$s = New-ScheduledTaskSettingsSet -Priority 4 -ExecutionTimeLimit (New-TimeSpan -Hours 12) -AllowStartIfOnBatteries
\$p = New-ScheduledTaskPrincipal -UserId (whoami) -LogonType Interactive
Unregister-ScheduledTask -TaskName pf-memory-check -Confirm:\$false -ErrorAction SilentlyContinue
Register-ScheduledTask -TaskName pf-memory-check -Action \$a -Settings \$s -Principal \$p | Out-Null
Start-ScheduledTask -TaskName pf-memory-check
EOF
)
ssh win "powershell -NoProfile -EncodedCommand $(printf '%s' "$TASK" | iconv -t UTF-16LE | base64 -w0)" >/dev/null
echo "started on the PC: order $ORDER, $MINUTES minutes each; results in $OUT"
RUNS=$(tr ',' '\n' <<< "$ORDER" | wc -l)
DEADLINE=$(( $(date +%s) + (RUNS * (MINUTES + 2) + 10) * 60 ))
until ssh win "Get-Content $ROOT\\out\\log.txt -ErrorAction SilentlyContinue" 2>/dev/null | tr -d '\r' | grep -q ' done$'; do
  [ "$(date +%s)" -gt "$DEADLINE" ] && { echo "no end after the expected time: the PC slept or the task stopped"; break; }
  sleep 60
done
scp -q "win:$RROOT/out/*" "$OUT/"
ssh win "Unregister-ScheduledTask -TaskName pf-memory-check -Confirm:\$false; Remove-Item -Recurse -Force $ROOT" >/dev/null
rm -f "$OUT/config.json"

python3 -I - "$OUT/samples.csv" "$A" "$B" <<'EOF'
import csv, statistics, sys
rows = list(csv.DictReader(open(sys.argv[1], encoding='utf-8-sig')))
names = {'a': sys.argv[2], 'b': sys.argv[3]}
cols = [('group_ws_mb', 'Task Manager, PecoFence group (MB)'), ('app_ws_mb', 'Task Manager, pecofence.exe (MB)'),
        ('app_commit_mb', 'private commit (MB)'), ('children_ws_mb', 'processes it started (MB)'),
        ('gpu_dedicated_mb', 'GPU dedicated (MB)'), ('gpu_shared_mb', 'GPU shared (MB)'),
        ('threads', 'threads'), ('handles', 'handles')]
print(f"{'':38s}" + ''.join(f'{b + " median":>14s}{b + " max":>11s}' for b in 'ab'))
for key, label in cols:
    line = f'{label:38s}'
    for b in 'ab':
        # The first five minutes (start-up) are left out of the median, not of the maximum.
        vals = [float(r[key]) for r in rows if r['build'] == b and int(r['second']) >= 300]
        allv = [float(r[key]) for r in rows if r['build'] == b]
        line += f'{statistics.median(vals):14.1f}{max(allv):11.1f}' if vals else f'{"-":>14s}{"-":>11s}'
    print(line)
for b in 'ab':
    runs = sorted({r['run'] for r in rows if r['build'] == b})
    cpu = []
    for run in runs:
        rr = [r for r in rows if r['run'] == run]
        late = [r for r in rr if int(r['second']) >= 300]
        if len(late) > 1:
            cpu.append((float(late[-1]['cpu_s']) - float(late[0]['cpu_s'])) * 60 / (int(late[-1]['second']) - int(late[0]['second'])))
    print(f"CPU per idle minute, build {b}: {', '.join(f'{c * 1000:.0f} ms' for c in cpu)}   ({names[b]})")
EOF
