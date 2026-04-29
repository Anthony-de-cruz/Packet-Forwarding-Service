#!/usr/bin/env bash

BUILD_PATH="../../UERANSIM/build"

echo "[+] Starting UERANSIM gNB..."
"$BUILD_PATH"/nr-gnb -c ./vm-ueransim-open5gs-gnb.yaml &

echo "[+] Starting UERANSIM UE..."
sudo "$BUILD_PATH"/nr-ue -c ./vm-ueransim-open5gs-ue-0.yaml &
#sudo "$BUILD_PATH"/nr-ue -c ./vm-ueransim-open5gs-ue.yaml &
