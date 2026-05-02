#!/usr/bin/env bash

# Run this script on all forwarding nodes.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring egress NAT for traffic selected by the classifier..."
while sudo iptables -t nat -D POSTROUTING -s "$VM_BRIDGE_SUBNET" -j MASQUERADE 2>/dev/null; do :; done
while sudo iptables -t nat -D POSTROUTING -s "$VM_UE_POOL_SUBNET" -j MASQUERADE 2>/dev/null; do :; done
sudo iptables -t nat -C POSTROUTING -s "$VM_BRIDGE_SUBNET" -o "$NODE_LAN_IFACE" -j MASQUERADE 2>/dev/null || \
    sudo iptables -t nat -A POSTROUTING -s "$VM_BRIDGE_SUBNET" -o "$NODE_LAN_IFACE" -j MASQUERADE
sudo iptables -t nat -C POSTROUTING -s "$VM_UE_POOL_SUBNET" -o "$NODE_LAN_IFACE" -j MASQUERADE 2>/dev/null || \
    sudo iptables -t nat -A POSTROUTING -s "$VM_UE_POOL_SUBNET" -o "$NODE_LAN_IFACE" -j MASQUERADE

echo "[+] Configuring return routes..."
sudo ip route replace "$VM_BRIDGE_SUBNET" via "$HOST_IP" dev "$NODE_LAN_IFACE"
sudo ip route replace "$VM_UE_POOL_SUBNET" via "$HOST_IP" dev "$NODE_LAN_IFACE"

echo "[!] Forwarding node setup complete!"
