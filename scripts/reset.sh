#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh" -euo pipefail

# Cleanup 
echo "[-] Cleaning up existing virtual network test environment..."
ip netns del $NS_1 2>/dev/null || true
ip netns del $NS_2 2>/dev/null || true
ip link del $VETH_1_1 2>/dev/null || true
ip link del $VETH_1_2 2>/dev/null || true

# Flush all rules.
echo "[-] Flushing iptable rules..."
iptables -F

echo "[-] Cleanup complete!"
