#!/usr/bin/env bash
# Usage: sbx.sh <name> [timeout-seconds] < commands
# Sends a command file to the demo helper in the Sandbox and waits for it to finish.
EV=${SBX_EVIDENCE:-/c/qual/film-kit/evidence}
name=$1; timeout=${2:-120}
S=$EV/cmd/status.txt
before=$(wc -l < "$S" 2>/dev/null || echo 0)
cat > "$EV/cmd/$name.tmp" && mv "$EV/cmd/$name.tmp" "$EV/cmd/$name.cmd"
for ((i = 0; i < timeout * 2; i++)); do
  grep -q "done $name\$" "$S" 2>/dev/null && break
  sleep 0.5
done
tail -n +$((before + 1)) "$S"
grep -q "done $name\$" "$S" || { echo "TIMEOUT waiting for $name"; exit 1; }
