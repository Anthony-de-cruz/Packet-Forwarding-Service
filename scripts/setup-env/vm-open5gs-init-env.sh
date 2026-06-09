#!/usr/bin/env bash

# Run this inside of the open5gs VM.

# Fail mode
set -euo pipefail

# Grab config values.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Removing Open5GS-side NAT for routed UE pool visibility..."
while sudo iptables -t nat -D POSTROUTING -s "$VM_UE_POOL_SUBNET" ! -o ogstun -j MASQUERADE 2>/dev/null; do :; done
while sudo iptables -t nat -D POSTROUTING -s "$VM_UE_POOL_SUBNET" -j MASQUERADE 2>/dev/null; do :; done

echo "[+] Accepting all forwarded traffic ingressing via ogstun (UE upstream)..."
sudo iptables -C FORWARD -i ogstun -j ACCEPT 2>/dev/null || \
    sudo iptables -I FORWARD 1 -i ogstun -j ACCEPT

echo "[+] Accepting forwarded traffic egressing via ogstun (UE downstream)..."
sudo iptables -C FORWARD -o ogstun -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT 2>/dev/null || \
    sudo iptables -I FORWARD 1 -o ogstun -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT

echo "[!] Open5GS VM setup complete!"
