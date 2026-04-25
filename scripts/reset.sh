#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh" -euo pipefail

# Cleanup 
# echo "[-] Cleaning up existing virtual network test environment..."
# ip netns del $NS_1 2>/dev/null || true
# ip netns del $NS_SRC 2>/dev/null || true
# ip link del $VETH_1_1 2>/dev/null || true
# ip link del $VETH_1_2 2>/dev/null || true
# ip link del $VETH_SRC_1 2>/dev/null || true
# ip link del $VETH_SRC_2 2>/dev/null || true

# Flush project routing policy.
echo "[-] Removing policy routing rules..."
ip rule del fwmark $FW_MARK_1 table $ROUTE_TABLE_1 2>/dev/null || true
ip rule del fwmark $FW_MARK_2 table $ROUTE_TABLE_2 2>/dev/null || true
ip rule del fwmark $FW_MARK_3 table $ROUTE_TABLE_3 2>/dev/null || true
ip rule del fwmark $FW_MARK_4 table $ROUTE_TABLE_4 2>/dev/null || true

ip route flush table $ROUTE_TABLE_1 2>/dev/null || true
ip route flush table $ROUTE_TABLE_2 2>/dev/null || true
ip route flush table $ROUTE_TABLE_3 2>/dev/null || true
ip route flush table $ROUTE_TABLE_4 2>/dev/null || true

# Flush packet handling rules.
echo "[-] Flushing mangle & nat iptable rules..."
iptables -t mangle -F
iptables -t nat -F

echo "[!] Cleanup complete!"
