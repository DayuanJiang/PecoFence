#!/bin/bash
# EC2 Windows test boxes for scripts/wintest.
#   box.sh create <name>     launch a new box (ready about 20 minutes later)
#   box.sh up <name>         start it if stopped, wait for ssh, keep an RDP session open
#   box.sh down <name>       end the RDP session and stop the instance
#   box.sh status            state of every box
#   box.sh ssh <name> [cmd]  PowerShell on the box as Administrator
# Private settings (key, pf's password, network ids, the list of boxes) live outside the repo in
# ~/.config/pecofence-wintest: see README.md.
set -uo pipefail
CONF=${WINTEST_CONF:-$HOME/.config/pecofence-wintest}
# shellcheck source=/dev/null
source "$CONF/config.sh"
export AWS_REGION=${WINTEST_REGION:-ap-northeast-1}
STATE=${WINTEST_STATE:-/tmp/pecofence-wintest}
mkdir -p "$STATE"

box_line() { grep -E "^$1 " "$CONF/boxes" 2>/dev/null | head -1; }
box_id() { box_line "$1" | awk '{print $2}'; }
box_ip() { box_line "$1" | awk '{print $3}'; }

ssh_box() {
  local name=$1; shift
  ssh -i "$CONF/pfwin" -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new \
    -o UserKnownHostsFile="$CONF/known_hosts" -o ConnectTimeout=10 -o ServerAliveInterval=30 \
    -o ControlMaster=auto -o ControlPath="$STATE/ssh-$name" -o ControlPersist=15m \
    "Administrator@$(box_ip "$name")" "$@"
}

rdp_loop() {
  # One RDP session for pf at the given size, reconnecting until the stop file appears.
  local name=$1 ip
  ip=$(box_ip "$name")
  rm -f "$STATE/rdp-$name.stop"
  while true; do
    xvfb-run -a -s "-screen 0 ${WINTEST_SIZE:-1600x900}x24" xfreerdp3 "/v:$ip" /u:pf "/p:$WINTEST_PF_PASSWORD" \
      "/size:${WINTEST_SIZE:-1600x900}" /cert:ignore /log-level:WARN +auto-reconnect >> "$STATE/rdp-$name.log" 2>&1
    [ -f "$STATE/rdp-$name.stop" ] && exit 0
    sleep 5
  done
}

case "${1:-}" in
  create)
    name=${2:?name}
    [ -n "$(box_id "$name")" ] && { echo "box $name exists"; exit 1; }
    iid=$(aws ec2 run-instances --image-id "$WINTEST_AMI" --instance-type "${WINTEST_TYPE:-m6i.xlarge}" \
      --subnet-id "$WINTEST_SUBNET" --security-group-ids "$WINTEST_SG" --user-data "file://$CONF/userdata.ps1" \
      --block-device-mappings 'DeviceName=/dev/sda1,Ebs={VolumeSize=60,VolumeType=gp3,DeleteOnTermination=true}' \
      --metadata-options HttpTokens=required \
      --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=pecofence-wintest-$name},{Key=Purpose,Value=PecoFence desktop tests}]" \
      --query 'Instances[0].InstanceId' --output text) || exit 1
    aws ec2 wait instance-running --instance-ids "$iid"
    ip=$(aws ec2 describe-instances --instance-ids "$iid" --query 'Reservations[0].Instances[0].PrivateIpAddress' --output text)
    echo "$name $iid $ip" >> "$CONF/boxes"
    echo "created $name $iid $ip; user data needs about 20 minutes, then: box.sh up $name"
    ;;
  up)
    name=${2:?name}; iid=$(box_id "$name")
    [ -z "$iid" ] && { echo "no box $name in $CONF/boxes"; exit 1; }
    st=$(aws ec2 describe-instances --instance-ids "$iid" --query 'Reservations[0].Instances[0].State.Name' --output text)
    if [ "$st" = stopped ]; then aws ec2 start-instances --instance-ids "$iid" >/dev/null; fi
    [ "$st" != running ] && aws ec2 wait instance-running --instance-ids "$iid"
    for _ in $(seq 1 60); do
      ssh_box "$name" 'Test-Path C:\userdata.done' 2>/dev/null | grep -q True && break
      sleep 10
    done
    ssh_box "$name" 'Test-Path C:\userdata.done' 2>/dev/null | grep -q True || { echo "box $name: no ssh or user data not done"; exit 1; }
    if ! pgrep -f "xfreerdp3 /v:$(box_ip "$name") " >/dev/null; then
      setsid nohup "$0" _rdp "$name" >/dev/null 2>&1 < /dev/null &
    fi
    # pf's session counts as ready once it is an RDP session (the console is only 800x600).
    for _ in $(seq 1 30); do
      ssh_box "$name" 'query user; exit 0' 2>/dev/null | grep -q 'rdp-tcp' && break
      sleep 3
    done
    # query user exits with 1 on Server 2025 even when it lists the session.
    ssh_box "$name" 'Stop-Service wuauserv -Force -ErrorAction SilentlyContinue; query user; exit 0' 2>/dev/null | tail -1
    ;;
  _rdp) rdp_loop "${2:?name}" ;;
  down)
    name=${2:?name}; iid=$(box_id "$name")
    touch "$STATE/rdp-$name.stop"
    pkill -f "xfreerdp3 /v:$(box_ip "$name") " 2>/dev/null
    ssh -O exit -o ControlPath="$STATE/ssh-$name" x 2>/dev/null
    aws ec2 stop-instances --instance-ids "$iid" --query 'StoppingInstances[0].CurrentState.Name' --output text
    ;;
  status)
    while read -r name iid ip; do
      [ -z "$name" ] && continue
      st=$(aws ec2 describe-instances --instance-ids "$iid" --query 'Reservations[0].Instances[0].State.Name' --output text)
      lock=$(cat "$STATE/locks/$name/owner" 2>/dev/null | head -1)
      rdp=$(pgrep -f "xfreerdp3 /v:$ip " >/dev/null && echo rdp || echo no-rdp)
      echo "$name $iid $ip $st $rdp ${lock:+locked: $lock}"
    done < "$CONF/boxes"
    ;;
  ssh) name=${2:?name}; shift 2; ssh_box "$name" "$@" ;;
  *) sed -n '2,9p' "$0"; exit 1 ;;
esac
