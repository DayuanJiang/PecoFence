#!/bin/bash
# Runs the desktop suite (suite.txt) on several EC2 boxes at once and prints one table.
#   suite.sh [--boxes ec2-2,ec2-3,ec2-4] [--vm PF-Win11-2] [--in-vm] [--bin DIR] [--debug-bin DIR]
#            [--installers DIR] [--only REGEX] [--out DIR]
# --bin / --debug-bin  the release and debug builds (default target/wintest-bin and
#                      target/wintest-bin-debug: build.sh, build.sh --debug); give an older
#                      release's folder to see which checks fail there.
# --installers DIR     the folder of installers.sh; without it the lines that need it are skipped.
# --vm NAME            runs the `vm` lines in that Hyper-V guest; with --in-vm every ec2 line too.
# --only REGEX         only the lines whose name matches.
# Each box takes the longest job still waiting when it gets free. Run this as the background
# command itself: it returns when every job has ended. Exit code: 0 all passed, 1 a check failed
# or a job could not run.
set -uo pipefail
KIT=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$KIT/../.." && pwd)
BOXES=ec2-2,ec2-3,ec2-4; VM=; INVM=0; ONLY=.; INST=
BIN=$REPO/target/wintest-bin; DBIN=$REPO/target/wintest-bin-debug
OUT=${CLAUDE_JOB_DIR:-/tmp}/tmp/suite-$(date +%m%d-%H%M%S)
while [ $# -gt 0 ]; do
  case "$1" in
    --boxes) BOXES=$2; shift 2 ;;
    --vm) VM=$2; shift 2 ;;
    --in-vm) INVM=1; shift ;;
    --bin) BIN=$2; shift 2 ;;
    --debug-bin) DBIN=$2; shift 2 ;;
    --installers) INST=$2; shift 2 ;;
    --only) ONLY=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    *) sed -n '2,14p' "$0"; exit 2 ;;
  esac
done
mkdir -p "$OUT"
T0=$(date +%s)

# The jobs: ec2 and resize lines go to the box queue, vm lines (and with --in-vm the ec2 ones)
# to the guest queue, unit to the first box; longest first.
: > "$OUT/queue-ec2"; : > "$OUT/queue-vm"; : > "$OUT/skipped"
grep -v -E '^\s*(#|$)' "$KIT/suite.txt" | sort -k2,2nr | while read -r name min where build file opts; do
  [[ "$name" =~ $ONLY ]] || continue
  if [ "$where" = manual ]; then echo "$name	manual: see the header of tests/$file" >> "$OUT/skipped"; continue; fi
  if [[ "$opts" == *@installers* && -z "$INST" ]]; then echo "$name	needs --installers (installers.sh)" >> "$OUT/skipped"; continue; fi
  line="$name	$where	$build	$file	${opts//@installers/$INST}"
  if [ "$where" = vm ] || { [ "$where" = ec2 ] && [ "$INVM" = 1 ]; }; then
    if [ -z "$VM" ]; then echo "$name	needs a guest (--vm)" >> "$OUT/skipped"; else echo "$line" >> "$OUT/queue-vm"; fi
  else
    echo "$line" >> "$OUT/queue-ec2"
  fi
done

# Takes the first line off a queue, or prints nothing when it is empty.
pop() { flock "$1.lock" bash -c 'head -n 1 "$1"; sed -i 1d "$1"' _ "$1"; }

run_one() {  # target name where build file opts
  local target=$1 name=$2 where=$3 build=$4 file=$5 opts=$6 bin=$BIN t=$(date +%s)
  [ "$build" = debug ] && bin=$DBIN
  {
    case "$where" in
      unit) "$KIT/unit.sh" --box "$target" --out "$OUT/$name" ;;
      resize) "$KIT/tests/free-placement.sh" --box "$target" $opts --bin "$bin" --out "$OUT/$name" ;;
      vm|ec2)
        if [[ "$target" == ec2-* ]]; then
          "$KIT/run.sh" --box "$target" --bin "$bin" --out "$OUT/$name" $opts "$KIT/tests/$file"
        else
          "$KIT/hyperv/run-hyperv.sh" --vm "$target" --bin "$bin" --out "$OUT/$name" $opts "$KIT/tests/$file"
        fi ;;
    esac
  } > "$OUT/$name.txt" 2>&1
  echo "$name	$target	$(( $(date +%s) - t ))" >> "$OUT/times"
}

worker() {  # target queue
  local line
  while line=$(pop "$2") && [ -n "$line" ]; do
    IFS=$'\t' read -r name where build file opts <<< "$line"
    run_one "$1" "$name" "$where" "$build" "$file" "$opts"
  done
}

IFS=, read -r -a boxes <<< "$BOXES"
echo "suite: $(wc -l < "$OUT/queue-ec2") jobs on ${#boxes[@]} boxes, $(wc -l < "$OUT/queue-vm") in ${VM:-no guest}, $(wc -l < "$OUT/skipped") skipped; results in $OUT"
for b in "${boxes[@]}"; do worker "$b" "$OUT/queue-ec2" & sleep 2; done
[ -n "$VM" ] && worker "$VM" "$OUT/queue-vm" &
wait

# One table: passed and failed per job, the failures, the skipped lines.
fails=0
printf '\n%-20s %-11s %6s %6s %6s\n' job where passed failed sec
while IFS=$'\t' read -r name target sec; do
  tot=$(grep -E '^== total: ' "$OUT/$name.txt" | tail -n 1)
  p=$(sed -E 's/.*total: ([0-9]+) passed.*/\1/' <<< "$tot"); f=$(sed -E 's/.*passed, ([0-9]+) failed.*/\1/' <<< "$tot")
  if [ -z "$tot" ]; then p=-; f='did not run'; fails=$((fails + 1)); elif [ "$f" != 0 ]; then fails=$((fails + 1)); fi
  printf '%-20s %-11s %6s %6s %6s\n' "$name" "$target" "$p" "$f" "$sec"
done < <(sort "$OUT/times")
echo
for t in $(sort "$OUT/times" | cut -f1); do
  if grep -q -E '^\s+FAIL ' "$OUT/$t.txt" || ! grep -q '^== total: ' "$OUT/$t.txt"; then
    echo "-- $t"; grep -E '^\s+FAIL |^steps not started|^PARSE|^no |screenshots of the failures' "$OUT/$t.txt" | head -n 12
  fi
done
[ -s "$OUT/skipped" ] && { echo; echo "skipped:"; sed 's/^/   /' "$OUT/skipped"; }
echo
echo "suite: $(wc -l < "$OUT/times") jobs, $fails with failures, $(( ($(date +%s) - T0) / 60 )) min; results in $OUT"
[ "$fails" = 0 ]
