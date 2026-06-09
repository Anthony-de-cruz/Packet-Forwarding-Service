#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../test-env/out"
REMOTE_OUT_DIR='~/Packet-Forwarding-Service/scripts/test-env/out'

mkdir -p "$OUT_DIR"

scp "user@192.168.0.52:$REMOTE_OUT_DIR/udp-echo-server.csv" "$OUT_DIR/rpi-2-udp-echo-server.csv"
scp "user@10.0.0.3:$REMOTE_OUT_DIR/udp-echo-client-latency.csv" "$OUT_DIR/vm-udp-echo-client.csv"
