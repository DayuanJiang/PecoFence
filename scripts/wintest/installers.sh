#!/bin/bash
# Makes the folder the install tests take with --extra (holdout-day.ps1, 186-install.ps1):
# new-setup.exe and portable.zip from a CI run's pecofence-windows artifact (the build under
# test), old-setup.exe from a release (the version a user upgrades from).
#   installers.sh [--run ID] [--old v0.2.5] [--out DIR]
# Default run: the latest successful CI run on main. Prints the folder on its last line.
set -euo pipefail
RUN=; OLD=v0.2.5; OUT=
while [ $# -gt 0 ]; do
  case "$1" in
    --run) RUN=$2; shift 2 ;;
    --old) OLD=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    *) sed -n '2,6p' "$0"; exit 2 ;;
  esac
done
[ -n "$RUN" ] || RUN=$(gh run list --workflow ci.yml --branch main --status success --limit 1 --json databaseId --jq '.[0].databaseId')
[ -n "$RUN" ] || { echo "no successful CI run found"; exit 2; }
OUT=${OUT:-${CLAUDE_JOB_DIR:-/tmp}/tmp/installers-$RUN}
mkdir -p "$OUT"
DL=$(mktemp -d)
gh run download "$RUN" -n pecofence-windows -D "$DL"
cp "$DL"/pecofence-v*-x64-setup.exe "$OUT/new-setup.exe"
cp "$DL"/pecofence-v*-x64-portable.zip "$OUT/portable.zip"
gh release download "$OLD" -p '*-x64-setup.exe' -D "$DL" --clobber
cp "$DL"/pecofence-"$OLD"-x64-setup.exe "$OUT/old-setup.exe"
rm -r "$DL"
echo "CI run $RUN, old $OLD:"
ls -l "$OUT" | tail -n +2
echo "$OUT"
