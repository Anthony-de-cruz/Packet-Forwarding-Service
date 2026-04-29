#!/usr/bin/env bash

# Run this script on all forwarding nodes.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring nat POSTROUTING nat ip table..."
sudo iptables -t nat -C POSTROUTING -s "$VM_BRIDGE_SUBNET" -j MASQUERADE 2>/dev/null || \
    sudo iptables -t nat -A POSTROUTING -s "$VM_BRIDGE_SUBNET" -j MASQUERADE
sudo iptables -t nat -C POSTROUTING -s "$VM_UE_POOL_SUBNET" -j MASQUERADE 2>/dev/null || \
    sudo iptables -t nat -A POSTROUTING -s "$VM_UE_POOL_SUBNET" -j MASQUERADE

echo "[+] Configuring return routes..."
sudo ip route replace "$VM_BRIDGE_SUBNET" via "$HOST_IP" dev eth0
sudo ip route replace "$VM_UE_POOL_SUBNET" via "$HOST_IP" dev eth0

echo "[!] Forwarding node setup complete!"
