#!/usr/bin/env bash

# Run this script on all forwarding nodes.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring POSTROUTING nat ip table..."
sudo iptables -t nat -A POSTROUTING -j MASQUERADE

echo "[+] Configuring gateway route..."
sudo ip route add $BRIDGE_SUBNET via $HOST_IP dev eth0

echo "[!] Forwarding node setup complete!"
