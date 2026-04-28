#!/usr/bin/env bash

# Script to setup VMs. Make sure not to run in sudo - this will break spice.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh" -euo pipefail

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
if ip link show "$BRIDGE_NAME" >/dev/null 2>&1; then
    :
else
    echo "[+] Creating new network bridge \"$BRIDGE_NAME\"..."
    sudo ip link add name "$BRIDGE_NAME" type bridge
    sudo ip addr add "$BRIDGE_ADDR" dev "$BRIDGE_NAME"
    sudo ip link set "$BRIDGE_NAME" up
fi

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring bridge NAT and forwarding rules..."
sudo iptables -t nat -C POSTROUTING -s "$BRIDGE_SUBNET" -o "$LAN_IFACE" -j MASQUERADE 2>/dev/null || \
sudo iptables -t nat -A POSTROUTING -s "$BRIDGE_SUBNET" -o "$LAN_IFACE" -j MASQUERADE
sudo iptables -C FORWARD -i "$BRIDGE_NAME" -o "$LAN_IFACE" -s "$BRIDGE_SUBNET" -j ACCEPT 2>/dev/null || \
    sudo iptables -A FORWARD -i "$BRIDGE_NAME" -o "$LAN_IFACE" -s "$BRIDGE_SUBNET" -j ACCEPT
sudo iptables -C FORWARD -i "$LAN_IFACE" -o "$BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT 2>/dev/null || \
    sudo iptables -A FORWARD -i "$LAN_IFACE" -o "$BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Launch VMs.
echo "[+] Launching quickemu VMs..."
# quickemu --vm ubuntu-22.04-open5gs.conf --display spice
# quickemu --vm ubuntu-22.04-ueransim.conf --display spice

# NixOS specific workaround: https://discourse.nixos.org/t/quickgui-with-bridged-networking-failed-to-create-tun-device/75893
# - Bypass faulty bridger helper wrapper.
# - Configure MAC addresses (by default they are the same which breaks inter-vm connections).
sudo -E quickemu --vm ubuntu-22.04-open5gs.conf --display spice --extra_args "-nic bridge,br=$BRIDGE_NAME,model=virtio-net-pci,mac=52:54:00:12:34:57,helper=/run/wrappers/bin/qemu-bridge-helper"
sudo -E quickemu --vm ubuntu-22.04-ueransim.conf --display spice --extra_args "-nic bridge,br=$BRIDGE_NAME,model=virtio-net-pci,mac=52:54:00:12:34:58,helper=/run/wrappers/bin/qemu-bridge-helper"

echo "[!] Host VM setup complete!"
echo "[!] This setup does not include DHCP so configure the VM NIC manually:"
echo "    VM 1   address: 10.0.0.2/24"
echo "    VM 2   address: 10.0.0.3/24"
echo "    VM 1+2 gateway: 10.0.0.1"
echo "    VM 1+2 dns: 1.1.1.1"
