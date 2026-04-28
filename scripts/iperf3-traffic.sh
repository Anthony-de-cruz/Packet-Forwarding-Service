#!/usr/bin/env bash

set -euo pipefail

SERVER_IP="192.168.0.53"
BIND_IP="10.45.0.2"
CONN_TOTAL=20000
CONN_INTERVAL=0.2
CONN_LIFETIME=3

for _ in $(seq 1 "$CONN_TOTAL"); do
	iperf3 -c "$SERVER_IP" -B "$BIND_IP" -t "$CONN_LIFETIME" >/dev/null 2>&1 &
	sleep "$CONN_INTERVAL"
done

wait
