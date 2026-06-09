#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Grab config values.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

while true; do
	iperf3 -c "$IPERF3_SERVER_IP" -B "$IPERF3_BIND_IP" -t "$IPERF3_CONN_LIFETIME" >/dev/null 2>&1 &
	sleep "$IPERF3_CONN_INTERVAL"
done
