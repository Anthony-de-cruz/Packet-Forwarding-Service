#!/usr/bin/env bash

# Reset host-side VM bridge connectivity rules.

# Fail mode
set -euo pipefail

# Grab config values.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

echo "[-] Removing VM bridge NAT and forwarding rules..."
while sudo iptables -t nat -D POSTROUTING -s "$VM_BRIDGE_SUBNET" -o "$HOST_LAN_IFACE" -j MASQUERADE 2>/dev/null; do :; done
while sudo iptables -D FORWARD -i "$VM_BRIDGE_NAME" -o "$HOST_LAN_IFACE" -s "$VM_BRIDGE_SUBNET" -j ACCEPT 2>/dev/null; do :; done
while sudo iptables -D FORWARD -i "$HOST_LAN_IFACE" -o "$VM_BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT 2>/dev/null; do :; done

echo "[!] VM bridge cleanup complete!"
