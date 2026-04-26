#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh" -euo pipefail

# Cleanup 

# Flush project routing policy.
echo "[-] Removing policy routing rules..."
while sudo ip rule del fwmark "$FW_MARK_1" table "$ROUTE_TABLE_1" 2>/dev/null; do :; done
while sudo ip rule del fwmark "$FW_MARK_2" table "$ROUTE_TABLE_2" 2>/dev/null; do :; done
while sudo ip rule del fwmark "$FW_MARK_3" table "$ROUTE_TABLE_3" 2>/dev/null; do :; done
while sudo ip rule del fwmark "$FW_MARK_4" table "$ROUTE_TABLE_4" 2>/dev/null; do :; done

sudo ip route flush table $ROUTE_TABLE_1 2>/dev/null || true
sudo ip route flush table $ROUTE_TABLE_2 2>/dev/null || true
sudo ip route flush table $ROUTE_TABLE_3 2>/dev/null || true
sudo ip route flush table $ROUTE_TABLE_4 2>/dev/null || true

# Remove project packet handling rules.
echo "[-] Removing project iptables rules..."
while sudo iptables -t mangle -D OUTPUT -j NFQUEUE --queue-num "$NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -t mangle -D OUTPUT -p udp --dport 9000 -j NFQUEUE --queue-num "$NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -t mangle -D PREROUTING -i "$BRIDGE_NAME" -j NFQUEUE --queue-num "$NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -t mangle -D FORWARD -i "$BRIDGE_NAME" -o "$LAN_IFACE" -j NFQUEUE --queue-num "$NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -t nat -D POSTROUTING -j MASQUERADE 2>/dev/null; do :; done
while sudo iptables -t nat -D POSTROUTING -s "$BRIDGE_SUBNET" -o "$LAN_IFACE" -j MASQUERADE 2>/dev/null; do :; done
while sudo iptables -D FORWARD -i "$BRIDGE_NAME" -o "$LAN_IFACE" -s "$BRIDGE_SUBNET" -j ACCEPT 2>/dev/null; do :; done
while sudo iptables -D FORWARD -i "$LAN_IFACE" -o "$BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT 2>/dev/null; do :; done

echo "[!] Cleanup complete!"
