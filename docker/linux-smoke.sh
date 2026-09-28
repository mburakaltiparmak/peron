#!/usr/bin/env bash
# Smoke test: install the built .deb on a clean Ubuntu, run it on a virtual display with a test
# listener, and save a screenshot to dist-linux/smoke.png. Runs inside ubuntu:22.04.
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq ./dist-linux/*.deb xvfb xauth dbus-x11 imagemagick python3 >/dev/null

python3 -m http.server 8765 >/dev/null 2>&1 &
export LANG=en_US.UTF-8
xvfb-run -a -s "-screen 0 1280x800x24" bash -c '
  dbus-launch --exit-with-session peron > /tmp/peron.log 2>&1 &
  sleep 12
  import -window root dist-linux/smoke.png
  pgrep -x peron >/dev/null && echo "PERON RUNNING" || { echo "PERON NOT RUNNING"; cat /tmp/peron.log; exit 1; }
'
echo "--- app log (tail)"; tail -n 20 /tmp/peron.log || true
