#!/bin/bash
# Runs steps files in a Hyper-V guest of the Windows PC (Windows 11 or 10) and prints the same
# table as run.sh does for an EC2 box.
#   run-hyperv.sh --vm NAME [--bin DIR] [--extra DIR] [--out DIR] [--timeout SEC] [--keep-notepad] steps.ps1 ...
# The PC is `ssh $WINTEST_HV_SSH` (default win), an administrator with Hyper-V. The guest must be
# Off with no checkpoint (another session's checkpoint means it is in use): guest-run.ps1 takes a
# checkpoint, sets 1600x900 (the steps expect it), runs one PFJob job per steps file, and
# restores the checkpoint and the video settings at the end. The Store Notepad is removed in the
# guest first (it restores earlier jobs' tabs over the Open dialog) unless --keep-notepad.
# Run this as the background command itself: it returns when the guest is restored.
set -uo pipefail
HV=$(cd "$(dirname "$0")" && pwd)
KIT=$(cd "$HV/.." && pwd)
REPO=$(cd "$KIT/../.." && pwd)
SSHH=${WINTEST_HV_SSH:-win}
VM=; BIN=$REPO/target/wintest-bin; EXTRA=; TIMEOUT=1500; NOTEPAD=-ClassicNotepad
TAG=$(date +%m%d-%H%M%S)
OUT=${CLAUDE_JOB_DIR:-/tmp/pecofence-wintest}/tmp/wintest-hv-$TAG
while [ $# -gt 0 ]; do
  case "$1" in
    --vm) VM=$2; shift 2 ;;
    --bin) BIN=$2; shift 2 ;;
    --extra) EXTRA=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --timeout) TIMEOUT=$2; shift 2 ;;
    --keep-notepad) NOTEPAD=; shift ;;
    -*) echo "unknown option $1"; exit 2 ;;
    *) break ;;
  esac
done
[ -n "$VM" ] && [ $# -ge 1 ] || { sed -n '2,11p' "$0"; exit 2; }
for s in "$@"; do [ -f "$s" ] || { echo "no steps file $s"; exit 2; }; done
[ -f "$BIN/pecofence.exe" ] || { echo "no build in $BIN"; exit 2; }
mkdir -p "$OUT"
STAGE=$OUT/.stage
rm -rf "$STAGE"; mkdir -p "$STAGE/stage/kit/steps" "$STAGE/stage/bin" "$STAGE/stage/extra"
ps() {  # PowerShell on the PC, UTF-8 out
  local enc
  enc=$( (printf '[Console]::OutputEncoding = [Text.Encoding]::UTF8\n$ProgressPreference = "SilentlyContinue"\n'; cat) | iconv -t UTF-16LE | base64 -w0)
  ssh -o ServerAliveInterval=30 "$SSHH" "powershell -NoProfile -EncodedCommand $enc" 2>&1 | grep -v -e '^#< CLIXML' -e '^<Objs'
}

# ---- files: the kit's scripts and the steps as run.sh writes them (UTF-8 with BOM, CRLF, a trap)
python3 -I - "$STAGE/stage/kit" "$KIT" -- "$@" <<'EOF'
import os, sys
dst, kit, rest = sys.argv[1], sys.argv[2], sys.argv[4:]
def put(src, to, trap=False):
    text = open(src, encoding='utf-8-sig').read()
    if trap:
        text = 'trap { PfTrap $_; continue }; ' + text
    open(to, 'w', encoding='utf-8-sig', newline='\r\n').write(text)
for n in sorted(os.listdir(kit)):
    if n.endswith('.ps1'):
        put(os.path.join(kit, n), os.path.join(dst, n))
for s in rest:
    put(s, os.path.join(dst, 'steps', os.path.basename(s)), trap=True)
EOF
[ -d "$KIT/assets" ] && cp -r "$KIT/assets" "$STAGE/stage/kit/assets"
cp -r "$BIN"/. "$STAGE/stage/bin/"
[ -n "$EXTRA" ] && cp -r "$EXTRA"/. "$STAGE/stage/extra/"
cp "$HV/guest-run.ps1" "$STAGE/"
STEPS=$(for s in "$@"; do basename "$s" .ps1; done | paste -sd+)
tar -cf "$OUT/stage.tar" -C "$STAGE" stage guest-run.ps1

# ---- the PC: a folder of this run, the guest runner started outside the ssh session
RB=$(printf '$b = Join-Path $env:TEMP "pfwintest-%s"; New-Item -ItemType Directory -Force $b | Out-Null; $b\n' "$TAG" | ps | tr -d '\r' | tail -1)
[ -n "$RB" ] || { echo "the PC did not answer (ssh $SSHH)"; exit 2; }
RBS=$(printf '%s' "$RB" | tr '\\' '/')
scp -q "$OUT/stage.tar" "$SSHH:$RBS/stage.tar" || { echo "copy to the PC failed"; exit 2; }
T0=$(date +%s)
START=$(ps <<EOF
\$b = '$RB'
Set-Location \$b
& tar.exe -xf stage.tar 2>&1 | Out-Null
\$v = Get-VM -Name '$VM' -ErrorAction SilentlyContinue
if (-not \$v) { 'no VM $VM'; exit }
\$cps = @(Get-VMSnapshot -VMName '$VM' -ErrorAction SilentlyContinue).Count
if (\$v.State -ne 'Off' -or \$cps) { "VM $VM is in use: \$(\$v.State), \$cps checkpoints"; exit }
\$cmd = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \`"\$b\\guest-run.ps1\`" -VM $VM -Steps $STEPS -Timeout $TIMEOUT -Base \`"\$b\`" -Stage stage $NOTEPAD"
\$r = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = \$cmd; CurrentDirectory = \$b }
"started \$(\$r.ReturnValue)"
EOF
)
START=$(printf '%s' "$START" | tr -d '\r' | tail -1)
[ "$START" = "started 0" ] || { echo "not started: $START"; exit 2; }
DONE="$RB\\done-$VM.txt"
while :; do
  sleep 30
  r=$(printf "if (Test-Path '%s') { 'yes' } else { 'no' }\n" "$DONE" | ps | tr -d '\r' | tail -1)
  [ "$r" = yes ] && break
done
T1=$(date +%s)

# ---- results: one tar per job (C:\pf\run), the runner's log
mkdir -p "$OUT/res"
scp -q -r "$SSHH:$RBS/res-$VM/." "$OUT/res/"
scp -q "$SSHH:$RBS/run-$VM.log" "$OUT/run.log"
for t in "$OUT"/res/*.tar; do
  [ -f "$t" ] || continue
  n=$(basename "$t" .tar); mkdir -p "$OUT/$n"
  tar -xf "$t" -C "$OUT/$n" --strip-components=1 2>/dev/null
done
printf "Remove-Item -Recurse -Force '%s' -ErrorAction SilentlyContinue\n" "$RB" | ps >/dev/null
grep -E 'ERROR|restored|video|notepad' "$OUT/run.log" | tr -d '\r'
python3 -I - "$OUT" "$OUT/res/jobs.txt" <<'EOF'
import os, sys
out, jobs = sys.argv[1], sys.argv[2]
tp = tf = 0
for line in (open(jobs, encoding='utf-8-sig') if os.path.exists(jobs) else []):
    if not line.strip():
        continue
    name, secs, status = line.split(' ', 2)
    status = status.strip()
    p = os.path.join(out, name, 'results.tsv')
    rows = [r.rstrip('\n').split('\t') for r in open(p, encoding='utf-8-sig') if r.strip()] if os.path.exists(p) else []
    passed = sum(1 for r in rows if len(r) > 2 and r[2] == 'PASS')
    failed = [r for r in rows if len(r) > 2 and r[2] != 'PASS']
    tp += passed; tf += len(failed)
    m, s = divmod(int(secs), 60)
    note = '' if status == 'done' else f', {status.upper()}'
    print(f'== {name}: {passed} passed, {len(failed)} failed ({m}m{s:02d}s{note})')
    for r in failed:
        r += [''] * 4
        print(f'   FAIL {r[0]} | {r[1]} | {r[3][:220]}')
    if failed:
        print(f'   screenshots of the failures: {out}/{name}/shots/FAIL-*.png')
print(f'== total: {tp} passed, {tf} failed')
sys.exit(1 if tf else 0)
EOF
FAILED=$?
echo "vm $VM; guest run $((T1 - T0))s"
echo "results: $OUT (per job: results.tsv, job.log, logs/, shots/)"
rm -rf "$STAGE" "$OUT/stage.tar"
exit "$FAILED"
