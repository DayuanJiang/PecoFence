#!/bin/bash
# Parts B and C of free-placement.ps1: the screen size (B) or the scale (C) changes while
# PecoFence runs. The job asks for one by writing C:\pf\resize-want.txt (WIDTHxHEIGHT or
# WIDTHxHEIGHT@SCALE); this script reconnects the box's RDP session so and removes the file.
# Ends with the session back at WINTEST_SIZE and 100 %.
# A job that writes C:\pf\clip-want.txt gets a file on the RDP client's clipboard (xclip on the
# session's X display): Windows' remote clipboard (rdpclip) then offers it in pf's session as a
# file copied on another machine (tests/issues-vm-paste.ps1, part V). By hand, before a plain
# run.sh: the same xclip line with DISPLAY and XAUTHORITY of the box's xfreerdp3 process.
#   tests/free-placement.sh [--box ec2-2] [--part B|C] [--steps FILE] [run.sh options ...]
# --steps runs another steps file that asks the same way (tests/fb-free-place-import.ps1); a
# name that is no file here is looked up in tests/.
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
CONF=${WINTEST_CONF:-$HOME/.config/pecofence-wintest}
STATE=${WINTEST_STATE:-/tmp/pecofence-wintest}
BOX=ec2-2
PART=B
[ "${1:-}" = --box ] && { BOX=$2; shift 2; }
[ "${1:-}" = --part ] && { PART=$2; shift 2; }
STEPS=$KIT/tests/free-placement.ps1
[ "${1:-}" = --steps ] && { s=$2; [ -f "$s" ] || s=$KIT/tests/$s; STEPS=$(realpath "$s"); shift 2; }
IP=$(awk -v b="$BOX" '$1 == b {print $3}' "$CONF/boxes")
[ -n "$IP" ] || { echo "no box $BOX"; exit 2; }

rdp_at() {
  # The box's RDP loop ends when its stop file is there and the client exits; start one at $1
  # (WIDTHxHEIGHT, then @SCALE for a desktop scale).
  touch "$STATE/rdp-$BOX.stop"
  pkill -f "xfreerdp3 /v:$IP " 2>/dev/null
  sleep 3
  local size=${1%@*} scale=
  [[ "$1" == *@* ]] && scale=${1#*@}
  WINTEST_SIZE=$size WINTEST_SCALE=$scale setsid nohup "$KIT/box.sh" _rdp "$BOX" >/dev/null 2>&1 < /dev/null &
  echo "rdp session for $BOX at $1"
}

clip_files() {
  # The client's clipboard: one small file, as text/uri-list on the X display of the box's RDP
  # client (its DISPLAY and XAUTHORITY come from xvfb-run).
  local pid env dir file
  # The client itself, not the xvfb-run shell around it (its command line names it too).
  pid=$(pgrep -f "^xfreerdp3 /v:$IP " | head -1)
  [ -n "$pid" ] || { echo "clip: no RDP client for $BOX"; return; }
  env=$(tr '\0' '\n' < "/proc/$pid/environ" | grep -E '^(DISPLAY|XAUTHORITY)=')
  dir="$STATE/clip-$BOX"; mkdir -p "$dir"
  file="$dir/ip vm 远程文件.txt"
  printf 'from the other machine\r\n' > "$file"
  [ -n "${CLIP_PID:-}" ] && kill "$CLIP_PID" 2>/dev/null
  # shellcheck disable=SC2086
  python3 -c 'import sys, urllib.parse; print("file://" + urllib.parse.quote(sys.argv[1]))' "$file" |
    env $env xclip -quiet -selection clipboard -t text/uri-list -i >/dev/null 2>&1 &
  CLIP_PID=$!
  echo "clip: '$file' on the client's clipboard"
}

"$KIT/run.sh" --box "$BOX" --var Part="$PART" "$@" "$STEPS" &
RUN=$!
while kill -0 "$RUN" 2>/dev/null; do
  want=$("$KIT/box.sh" ssh "$BOX" 'if (Test-Path C:\pf\resize-want.txt) { Get-Content C:\pf\resize-want.txt }; if (Test-Path C:\pf\clip-want.txt) { "clip" }' 2>/dev/null | tr -d '\r ')
  if [[ "$want" == *clip ]]; then
    want=${want%clip}
    { [ -n "${CLIP_PID:-}" ] && kill -0 "$CLIP_PID" 2>/dev/null; } || clip_files
  fi
  if [[ "$want" =~ ^[0-9]+x[0-9]+(@[0-9]+)?$ ]]; then
    rdp_at "$want"
    sleep 8
    "$KIT/box.sh" ssh "$BOX" 'Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue' >/dev/null 2>&1
  fi
  sleep 3
done
wait "$RUN"; rc=$?
[ -n "${CLIP_PID:-}" ] && kill "$CLIP_PID" 2>/dev/null
pgrep -f "xfreerdp3 /v:$IP /u:pf .*/size:${WINTEST_SIZE:-1600x900} /cert" >/dev/null || rdp_at "${WINTEST_SIZE:-1600x900}"
exit $rc
