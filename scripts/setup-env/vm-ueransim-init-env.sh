#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BUILD_PATH="${BUILD_PATH:-"$SCRIPT_DIR/../../../UERANSIM/build"}"

cleanup() {
  echo "[+] Stopping UERANSIM processes..."
  kill "${GNB_PID:-}" "${UE_PID:-}" 2>/dev/null || true
}

trap cleanup EXIT INT TERM

echo "[+] Starting UERANSIM gNB..."
"$BUILD_PATH/nr-gnb" -c "$SCRIPT_DIR/vm-ueransim-open5gs-gnb.yaml" &
GNB_PID=$!

sleep 2

echo "[+] Starting UERANSIM UE..."
sudo "$BUILD_PATH/nr-ue" -c "$SCRIPT_DIR/vm-ueransim-open5gs-ue-0.yaml" &
UE_PID=$!

wait "$GNB_PID" "$UE_PID"
