#!/usr/bin/env bash
set -euo pipefail

BUILD_PATH="../../UERANSIM/build" # CHANGE TO YOUR UERANSIM BUILD PATH.

cleanup() {
  echo "[+] Stopping UERANSIM processes..."
  kill "${GNB_PID:-}" "${UE_PID:-}" 2>/dev/null || true
}

trap cleanup EXIT INT TERM

echo "[+] Starting UERANSIM gNB..."
"$BUILD_PATH/nr-gnb" -c ./vm-ueransim-open5gs-gnb.yaml &
GNB_PID=$!

sleep 2

echo "[+] Starting UERANSIM UE..."
sudo "$BUILD_PATH/nr-ue" -c ./vm-ueransim-open5gs-ue-0.yaml &
UE_PID=$!

wait "$GNB_PID" "$UE_PID"
