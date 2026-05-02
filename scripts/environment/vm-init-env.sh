#!/usr/bin/env bash

# Script to setup VMs. Make sure not to run in sudo - this will break spice.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/vm-reset-env.sh"

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
if ip link show "$VM_BRIDGE_NAME" >/dev/null 2>&1; then
    :
else
    echo "[+] Creating new network bridge \"$VM_BRIDGE_NAME\"..."
    sudo ip link add name "$VM_BRIDGE_NAME" type bridge
    sudo ip addr add "$VM_BRIDGE_ADDR" dev "$VM_BRIDGE_NAME"
    sudo ip link set "$VM_BRIDGE_NAME" up
fi

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring bridge NAT and forwarding rules..."
sudo iptables -t nat -C POSTROUTING -s "$VM_BRIDGE_SUBNET" -o "$HOST_LAN_IFACE" -j MASQUERADE 2>/dev/null || \
sudo iptables -t nat -A POSTROUTING -s "$VM_BRIDGE_SUBNET" -o "$HOST_LAN_IFACE" -j MASQUERADE
sudo iptables -C FORWARD -i "$VM_BRIDGE_NAME" -o "$HOST_LAN_IFACE" -s "$VM_BRIDGE_SUBNET" -j ACCEPT 2>/dev/null || \
    sudo iptables -A FORWARD -i "$VM_BRIDGE_NAME" -o "$HOST_LAN_IFACE" -s "$VM_BRIDGE_SUBNET" -j ACCEPT
sudo iptables -C FORWARD -i "$HOST_LAN_IFACE" -o "$VM_BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT 2>/dev/null || \
    sudo iptables -A FORWARD -i "$HOST_LAN_IFACE" -o "$VM_BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Launch VMs.
echo "[+] Launching quickemu VMs..."
# quickemu --vm ubuntu-22.04-open5gs.conf --display spice
# quickemu --vm ubuntu-22.04-ueransim.conf --display spice

# NixOS specific workaround: https://discourse.nixos.org/t/quickgui-with-bridged-networking-failed-to-create-tun-device/75893
# - Bypass faulty bridger helper wrapper.
# - Configure MAC addresses (by default they are the same which breaks inter-vm connections).
sudo -E quickemu --vm ubuntu-22.04-open5gs.conf --display spice --extra_args "-nic bridge,br=$VM_BRIDGE_NAME,model=virtio-net-pci,mac=52:54:00:12:34:57,helper=/run/wrappers/bin/qemu-bridge-helper"
sudo -E quickemu --vm ubuntu-22.04-ueransim.conf --display spice --extra_args "-nic bridge,br=$VM_BRIDGE_NAME,model=virtio-net-pci,mac=52:54:00:12:34:58,helper=/run/wrappers/bin/qemu-bridge-helper"

echo "[!] Host VM setup complete!"
echo "[!] This setup does not include DHCP so configure the VM NIC manually:"
echo "    VM 1   address: $VM_OPEN5GS_IP/24"
echo "    VM 2   address: $VM_UERANSIM_IP/24"
echo "    VM 1+2 gateway: ${VM_BRIDGE_ADDR%/*}"
echo "    VM 1+2 dns: 1.1.1.1"
