#!/usr/bin/env bash

# Run this inside of the open5gs VM.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Enabling NAT masquerade for UE pool subnet traffic forwarded to WAN..."
sudo iptables -t nat -A POSTROUTING -s "$VM_UE_POOL_SUBNET" ! -o ogstun -j MASQUERADE

echo "[+] Accepting all forwarded traffic ingressing via ogstun (UE upstream)..."
sudo iptables -I FORWARD 1 -i ogstun -j ACCEPT

echo "[+] Accepting forwarded traffic egressing via ogstun (UE downstream)..."
sudo iptables -I FORWARD 1 -o ogstun -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT

echo "[!] Open5GS VM setup complete!"
