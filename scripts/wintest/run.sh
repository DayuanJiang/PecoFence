#!/bin/bash
# Runs desktop test steps on a Windows test box and prints one pass/fail table.
#   run.sh [--box NAME|any] [--bin DIR] [--extra DIR] [--out DIR] [--timeout SEC]
#          [--var NAME=VALUE ...] steps.ps1 [more.ps1 ...]
# Each steps file is one job: a fresh C:\pf\run and fresh PecoFence data, the build in --bin
# (default target/wintest-bin, made by build.sh). --var sets $NAME = 'VALUE' for the steps (for
# example which part of a long file to run). The command returns when every job is done or
# hit its timeout (default 900 s each), so it can be the background command itself: never
# watch its output with a separate wait loop. Exit code: 0 all passed, 1 a check failed,
# 2 the harness could not run a job.
set -uo pipefail
KIT=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$KIT/../.." && pwd)
CONF=${WINTEST_CONF:-$HOME/.config/pecofence-wintest}
STATE=${WINTEST_STATE:-/tmp/pecofence-wintest}
BOX=any; BIN=$REPO/target/wintest-bin; EXTRA=; TIMEOUT=900; VARS=
OUT=${CLAUDE_JOB_DIR:-$STATE}/tmp/wintest-$(date +%m%d-%H%M%S)
while [ $# -gt 0 ]; do
  case "$1" in
    --box) BOX=$2; shift 2 ;;
    --bin) BIN=$2; shift 2 ;;
    --extra) EXTRA=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --timeout) TIMEOUT=$2; shift 2 ;;
    --var)
      vn=${2%%=*}; vv=${2#*=}
      [[ "$2" == *=* && "$vn" =~ ^[A-Za-z_][A-Za-z0-9_]*$ && "$vv" != *\'* ]] || { echo "--var wants NAME=VALUE (no quotes)"; exit 2; }
      VARS+="\$$vn = '$vv'"$'\r\n'; shift 2 ;;
    -*) echo "unknown option $1"; exit 2 ;;
    *) break ;;
  esac
done
[ $# -ge 1 ] || { sed -n '2,8p' "$0"; exit 2; }
for s in "$@"; do [ -f "$s" ] || { echo "no steps file $s"; exit 2; }; done
[ -f "$BIN/pecofence.exe" ] || { echo "no build in $BIN (run build.sh)"; exit 2; }
mkdir -p "$STATE/locks" "$OUT"
T0=$(date +%s)
now() { date +%s; }

# ---- lock: one run per box; a lock whose owner process is gone is taken over
# Looking at a lock, taking it over and writing its owner line happen under one file lock: without
# it a run that had just made the lock's folder and not yet written its owner line lost it to the
# next run that looked (2026-11-13: two jobs of two runs ran in one C:\pf\run at once).
take_lock() {
  local d=$STATE/locks/$1 pid got=
  exec 9> "$STATE/take-$1.lock"
  flock 9
  if mkdir "$d" 2>/dev/null; then
    got=1
  else
    pid=$(awk '{print $1}' "$d/owner" 2>/dev/null)
    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
      rm -rf "$d"; mkdir "$d" 2>/dev/null && got=1
    fi
  fi
  [ -n "$got" ] && echo "$$ $(date +%H:%M:%S) ${CLAUDE_JOB_DIR:-no-job} $*" > "$d/owner"
  flock -u 9; exec 9>&-
  [ -n "$got" ] || return 1
  LOCKED=$d
}
LOCKED=
trap '[ -n "$LOCKED" ] && rm -rf "$LOCKED"' EXIT
while :; do
  if [ "$BOX" = any ]; then
    for b in $(awk '{print $1}' "$CONF/boxes"); do take_lock "$b" "$@" && { BOX=$b; break; }; done
  else
    take_lock "$BOX" "$@"
  fi
  [ -n "$LOCKED" ] && break
  [ $(( $(now) - T0 )) -gt 3600 ] && { echo "no free box after 60 minutes"; exit 2; }
  [ -z "${WAITED:-}" ] && { echo "waiting for a free box ($(cat "$STATE"/locks/*/owner 2>/dev/null | tr '\n' ';'))"; WAITED=1; }
  sleep 15
done
T_LOCK=$(now)

# ---- box up, one ssh connection for everything
"$KIT/box.sh" up "$BOX" >/dev/null || { echo "box $BOX did not come up"; exit 2; }
IP=$(awk -v b="$BOX" '$1 == b {print $3}' "$CONF/boxes")
SSHO=(-i "$CONF/pfwin" -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile="$CONF/known_hosts"
  -o ConnectTimeout=10 -o ServerAliveInterval=30 -o ControlMaster=auto -o ControlPath="$STATE/ssh-$BOX" -o ControlPersist=15m)
sshx() { ssh "${SSHO[@]}" "Administrator@$IP" "$@"; }
scpx() { scp -q "${SSHO[@]}" "$@"; }
ctl() { sshx "powershell -NoProfile -ExecutionPolicy Bypass -File C:\\pf\\kit\\ctl.ps1 $1" 2>&1 | tr -d '\r'; }
T_UP=$(now)

# ---- files: kit and steps every time (UTF-8 with BOM for PowerShell 5.1), build and extras
# only when their sha256 list differs from the box's
STAGE=$OUT/.stage; rm -rf "$STAGE"; mkdir -p "$STAGE/kit/steps"
python3 -I - "$STAGE/kit" "$KIT"/*.ps1 -- "$@" <<'EOF'
import os, sys
dst, rest = sys.argv[1], sys.argv[2:]
cut = rest.index('--')
for i, p in enumerate(rest):
    if p == '--':
        continue
    to = os.path.join(dst, os.path.basename(p)) if i < cut else os.path.join(dst, 'steps', os.path.basename(p))
    text = open(p, encoding='utf-8-sig').read()
    if i > cut:
        # A trap in the steps file itself: an error in a line is recorded and the next line of
        # the steps runs (a trap only in job.ps1 would skip the rest of the file). Same line, so
        # line numbers in the table stay right.
        text = 'trap { PfTrap $_; continue }; ' + text
    open(to, 'w', encoding='utf-8-sig', newline='\r\n').write(text)
EOF
# Helper programs the steps start (C:\pf\kit\assets on the box).
[ -d "$KIT/assets" ] && cp -r "$KIT/assets" "$STAGE/kit/assets"
sshx 'Remove-Item -Recurse -Force C:\pf\kit -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force C:\pf\kit | Out-Null' >/dev/null
scpx -r "$STAGE/kit/"* "Administrator@$IP:C:/pf/kit/"
sync_dir() {  # local dir, box dir name
  local src=$1 name=$2 here there
  here=$(cd "$src" && find . -type f ! -name manifest.txt -print0 | sort -z | xargs -0 sha256sum)
  there=$(sshx "Get-Content C:\\pf\\$name\\manifest.txt -ErrorAction SilentlyContinue" 2>/dev/null | tr -d '\r')
  if [ "$here" = "$there" ]; then echo "same"; return; fi
  printf '%s\n' "$here" > "$STAGE/manifest-$name.txt"
  sshx "Remove-Item -Recurse -Force C:\\pf\\$name -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force C:\\pf\\$name | Out-Null" >/dev/null
  scpx -r "$src"/* "Administrator@$IP:C:/pf/$name/"
  scpx "$STAGE/manifest-$name.txt" "Administrator@$IP:C:/pf/$name/manifest.txt"
  echo "copied"
}
SYNC_BIN=$(sync_dir "$BIN" bin)
SYNC_EXTRA=; [ -n "$EXTRA" ] && SYNC_EXTRA=$(sync_dir "$EXTRA" extra)
CHECK=$(ctl check)
if ! grep -q ': 0 problems' <<< "$CHECK"; then
  echo "steps not started, the scripts have problems:"; grep -v '^checked' <<< "$CHECK"; exit 2
fi
T_SYNC=$(now)

# ---- one job per steps file
HARNESS=0
for s in "$@"; do
  name=$(basename "$s" .ps1)
  { printf '%s' "$VARS"
    printf '$PfSteps = '"'"'C:\\pf\\kit\\steps\\%s.ps1'"'"'\r\n. C:\\pf\\kit\\job.ps1\r\n' "$name"; } > "$STAGE/job.ps1"
  ctl reset >/dev/null
  scpx "$STAGE/job.ps1" "Administrator@$IP:C:/pf/job.ps1"
  t=$(now)
  st=$(ctl start | tail -1)
  status=$st
  if [ "$st" = started ]; then
    while :; do
      sleep 3
      status=$(ctl status | tail -1)
      [ "$status" = running ] || break
      if [ $(( $(now) - t )) -gt "$TIMEOUT" ]; then ctl abort >/dev/null; status="timeout after ${TIMEOUT}s"; break; fi
    done
  fi
  echo "$name $(( $(now) - t )) $status" >> "$STAGE/jobs.txt"
  col=$(ctl collect | tail -1)
  mkdir -p "$OUT/$name"
  scpx "Administrator@$IP:C:/pf/run.tar" "$STAGE/run.tar" &&
    terr=$(tar -xf "$STAGE/run.tar" -C "$OUT/$name" --strip-components=1 2>&1) ||
    echo "$name: the results archive is incomplete ($col; $terr)"
  [ "$status" = done ] || HARNESS=1
done
T_END=$(now)

# ---- the table
python3 -I - "$OUT" "$STAGE/jobs.txt" <<'EOF'
import os, sys
out, jobs = sys.argv[1], sys.argv[2]
total_p = total_f = 0
for line in open(jobs):
    name, secs, status = line.split(' ', 2)
    status = status.strip()
    rows = []
    p = os.path.join(out, name, 'results.tsv')
    if os.path.exists(p):
        rows = [r.rstrip('\n').split('\t') for r in open(p, encoding='utf-8-sig') if r.strip()]
    passed = sum(1 for r in rows if len(r) > 2 and r[2] == 'PASS')
    failed = [r for r in rows if len(r) > 2 and r[2] != 'PASS']
    total_p += passed; total_f += len(failed)
    m, s = divmod(int(secs), 60)
    note = '' if status == 'done' else f', {status.upper()}'
    print(f'== {name}: {passed} passed, {len(failed)} failed ({m}m{s:02d}s{note})')
    for i, r in enumerate(failed, 1):
        r += [''] * 4
        print(f'   FAIL {r[0]} | {r[1]} | {r[3][:220]}')
    if failed:
        print(f'   screenshots of the failures: {out}/{name}/shots/FAIL-*.png')
print(f'== total: {total_p} passed, {total_f} failed')
sys.exit(1 if total_f else 0)
EOF
FAILED=$?
echo "box $BOX; lock wait $((T_LOCK - T0))s, box up $((T_UP - T_LOCK))s, files $((T_SYNC - T_UP))s (build ${SYNC_BIN}${SYNC_EXTRA:+, extra $SYNC_EXTRA}), jobs $((T_END - T_SYNC))s"
echo "results: $OUT (per job: results.tsv, job.log, logs/, shots/)"
rm -rf "$STAGE/kit" "$STAGE/run.tar"
[ "$HARNESS" = 1 ] && exit 2
exit "$FAILED"
