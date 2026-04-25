#!/usr/bin/env bash

# Script to setup VMs. Make sure not to run in sudo - this will break spice.

# Fail mode
set -euo pipefail

# Download the Ubuntu ISO if it does not exist already,
# then copy to the correct directory.
if [[ ! -d "ubuntu-22.04-open5gs" && ! -d "ubuntu-22.04-ueransim" ]]; then
    echo "[+] Installing quickemu VMs..."
    quickget ubuntu 22.04 && \
        rm -f ubuntu-22.04.conf && \
        cp -r ubuntu-22.04 ubuntu-22.04-open5gs && \
        mv ubuntu-22.04 ubuntu-22.04-ueransim
fi

# Setup network bridge + nat.
if ip link show br-5g >/dev/null 2>&1; then
    :
else
    echo "[+] Creating new network bridge \"br-5g\"..."
    sudo ip link add name br-5g type bridge
    sudo ip addr add 10.0.0.1/24 dev br-5g
    sudo ip link set br-5g up
    # sudo ip link set enp0s13f0u4 master br-5g # YOUR LAN INTERFACE HERE (run `ip a` to list them)

    echo "[+] Enabling IPv4 forwarding on host..."
    sysctl -w net.ipv4.ip_forward=1

    echo "[+] Configuring bridge POSTROUTING nat..."
    sudo iptables -t nat -A POSTROUTING -o enp0s13f0u4 -j MASQUERADE
fi


# Launch VMs.
echo "[+] Launching quickemu VMs..."
# quickemu --vm ubuntu-22.04-open5gs.conf --display spice
# quickemu --vm ubuntu-22.04-ueransim.conf --display spice

# NixOS specific workaround: https://discourse.nixos.org/t/quickgui-with-bridged-networking-failed-to-create-tun-device/75893
sudo -E quickemu --vm ubuntu-22.04-open5gs.conf --display spice --extra_args "-nic bridge,br=br-5g,model=virtio-net-pci,helper=/run/wrappers/bin/qemu-bridge-helper"
# sudo -E quickemu --vm ubuntu-22.04-ueransim.conf --display spice --extra_args "-nic bridge,br=br-5g,model=virtio-net-pci,helper=/run/wrappers/bin/qemu-bridge-helper"

echo "[!] Host VM setup complete!"
