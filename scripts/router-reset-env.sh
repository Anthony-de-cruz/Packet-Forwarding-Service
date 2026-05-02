#!/usr/bin/env bash

# Cleanup router policy and packet classification rules.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"


# Flush project routing policy.
echo "[-] Removing policy routing rules..."
while sudo ip rule del fwmark "$HOST_FW_MARK_1" table "$HOST_ROUTE_TABLE_1" 2>/dev/null; do :; done
while sudo ip rule del fwmark "$HOST_FW_MARK_2" table "$HOST_ROUTE_TABLE_2" 2>/dev/null; do :; done
while sudo ip rule del fwmark "$HOST_FW_MARK_3" table "$HOST_ROUTE_TABLE_3" 2>/dev/null; do :; done
# Remove the old fourth route path from previous configurations.
while sudo ip rule del fwmark 0x804 table 104 2>/dev/null; do :; done

sudo ip route flush table $HOST_ROUTE_TABLE_1 2>/dev/null || true
sudo ip route flush table $HOST_ROUTE_TABLE_2 2>/dev/null || true
sudo ip route flush table $HOST_ROUTE_TABLE_3 2>/dev/null || true
# Flush the old fourth route table from previous configurations.
sudo ip route flush table 104 2>/dev/null || true
sudo ip route del "$VM_UE_POOL_SUBNET" via "$VM_OPEN5GS_IP" dev "$VM_BRIDGE_NAME" 2>/dev/null || true

# Remove project packet handling rules.
echo "[-] Removing project iptables rules..."
while sudo iptables -t mangle -D OUTPUT -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -t mangle -D OUTPUT -p udp --dport 9000 -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -t mangle -D PREROUTING -i "$VM_BRIDGE_NAME" -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass 2>/dev/null; do :; done
while sudo iptables -D FORWARD -i "$VM_BRIDGE_NAME" -o "$HOST_LAN_IFACE" -s "$VM_UE_POOL_SUBNET" -j ACCEPT 2>/dev/null; do :; done
while sudo iptables -D FORWARD -i "$HOST_LAN_IFACE" -o "$VM_BRIDGE_NAME" -d "$VM_UE_POOL_SUBNET" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT 2>/dev/null; do :; done

echo "[!] Cleanup complete!"
