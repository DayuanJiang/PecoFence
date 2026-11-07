#!/bin/bash
# Part B of fx-free-placement.ps1: the screen size changes while PecoFence runs. The job asks for
# a size by writing C:\pf\resize-want.txt (WIDTHxHEIGHT); this script reconnects the box's RDP
# session at that size and removes the file. Ends with the session back at WINTEST_SIZE.
#   tests/fx-free-placement.sh [--box ec2-2] [run.sh options ...]
set -uo pipefail
KIT=$(cd "$(dirname "$0")/.." && pwd)
CONF=${WINTEST_CONF:-$HOME/.config/pecofence-wintest}
STATE=${WINTEST_STATE:-/tmp/pecofence-wintest}
BOX=ec2-2
[ "${1:-}" = --box ] && { BOX=$2; shift 2; }
IP=$(awk -v b="$BOX" '$1 == b {print $3}' "$CONF/boxes")
[ -n "$IP" ] || { echo "no box $BOX"; exit 2; }

rdp_at() {
  # The box's RDP loop ends when its stop file is there and the client exits; start one at $1.
  touch "$STATE/rdp-$BOX.stop"
  pkill -f "xfreerdp3 /v:$IP " 2>/dev/null
  sleep 3
  WINTEST_SIZE=$1 setsid nohup "$KIT/box.sh" _rdp "$BOX" >/dev/null 2>&1 < /dev/null &
  echo "rdp session for $BOX at $1"
}

"$KIT/run.sh" --box "$BOX" --var Part=B "$@" "$KIT/tests/fx-free-placement.ps1" &
RUN=$!
while kill -0 "$RUN" 2>/dev/null; do
  want=$("$KIT/box.sh" ssh "$BOX" 'if (Test-Path C:\pf\resize-want.txt) { Get-Content C:\pf\resize-want.txt }' 2>/dev/null | tr -d '\r ')
  if [[ "$want" =~ ^[0-9]+x[0-9]+$ ]]; then
    rdp_at "$want"
    sleep 8
    "$KIT/box.sh" ssh "$BOX" 'Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue' >/dev/null 2>&1
  fi
  sleep 3
done
wait "$RUN"; rc=$?
pgrep -f "xfreerdp3 /v:$IP .*/size:${WINTEST_SIZE:-1600x900}" >/dev/null || rdp_at "${WINTEST_SIZE:-1600x900}"
exit $rc
