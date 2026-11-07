#!/bin/bash
# Parts B and C of fx-free-placement.ps1: the screen size (B) or the scale (C) changes while
# PecoFence runs. The job asks for one by writing C:\pf\resize-want.txt (WIDTHxHEIGHT or
# WIDTHxHEIGHT@SCALE); this script reconnects the box's RDP session so and removes the file.
# Ends with the session back at WINTEST_SIZE and 100 %.
#   tests/fx-free-placement.sh [--box ec2-2] [--part B|C] [run.sh options ...]
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
CONF=${WINTEST_CONF:-$HOME/.config/pecofence-wintest}
STATE=${WINTEST_STATE:-/tmp/pecofence-wintest}
BOX=ec2-2
PART=B
[ "${1:-}" = --box ] && { BOX=$2; shift 2; }
[ "${1:-}" = --part ] && { PART=$2; shift 2; }
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

"$KIT/run.sh" --box "$BOX" --var Part="$PART" "$@" "$KIT/tests/fx-free-placement.ps1" &
RUN=$!
while kill -0 "$RUN" 2>/dev/null; do
  want=$("$KIT/box.sh" ssh "$BOX" 'if (Test-Path C:\pf\resize-want.txt) { Get-Content C:\pf\resize-want.txt }' 2>/dev/null | tr -d '\r ')
  if [[ "$want" =~ ^[0-9]+x[0-9]+(@[0-9]+)?$ ]]; then
    rdp_at "$want"
    sleep 8
    "$KIT/box.sh" ssh "$BOX" 'Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue' >/dev/null 2>&1
  fi
  sleep 3
done
wait "$RUN"; rc=$?
pgrep -f "xfreerdp3 /v:$IP /u:pf .*/size:${WINTEST_SIZE:-1600x900} /cert" >/dev/null || rdp_at "${WINTEST_SIZE:-1600x900}"
exit $rc
