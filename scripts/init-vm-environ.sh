#!/usr/bin/env bash

# Script to setup VMs.

# Fail mode
set -euo pipefail

# Download the Ubuntu ISO if it does not exist already,
# then copy to the correct directory.
if [[ ! -d "ubuntu-22.04-open5gs" && ! -d "ubuntu-22.04-ueransim" ]]; then
    quickget ubuntu 22.04 && \
        rm -f ubuntu-22.04.conf && \
        cp -r ubuntu-22.04 ubuntu-22.04-open5gs && \
        mv ubuntu-22.04 ubuntu-22.04-ueransim
fi

# Setup network bridge.
if ip link show br-5g >/dev/null 2>&1; then
    :
else
    sudo ip link add name br-5g type bridge
fi
sudo ip link set br-5g up
# sudo ip link set your_interface master br-5g

# Launch VMs.
quickemu --vm ubuntu-22.04-open5gs.conf --display spice
quickemu --vm ubuntu-22.04-ueransim.conf --display spice
