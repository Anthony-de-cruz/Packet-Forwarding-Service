#!/usr/bin/env bash

# Script to setup an environment for local testing.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/reset.sh"

echo "[+] Creating veth pairs..."
ip link add $VETH_1_1 type veth peer name $VETH_1_2
ip link add $VETH_SRC_1 type veth peer name $VETH_SRC_2

echo "[+] Assigning veth interfaces to namespaces..."
ip netns add $NS_1
ip link set $VETH_1_2 netns $NS_1
ip netns exec $NS_1 sysctl -w net.ipv4.ip_forward=1

ip netns add $NS_SRC
ip link set $VETH_SRC_1 netns $NS_SRC

echo "[+] Configuring IP addresses..."
ip addr add $VETH_1_1_IP dev $VETH_1_1
ip netns exec $NS_1 ip addr add $VETH_1_2_IP dev $VETH_1_2

ip netns exec $NS_SRC ip addr add $VETH_SRC_1_IP dev $VETH_SRC_1
ip addr add $VETH_SRC_2_IP dev $VETH_SRC_2

echo "[+] Bringing interfaces up..."
ip link set $VETH_1_1 up
ip netns exec $NS_1 ip link set lo up
ip netns exec $NS_1 ip link set $VETH_1_2 up

ip link set $VETH_SRC_2 up
ip netns exec $NS_SRC ip link set lo up
ip netns exec $NS_SRC ip link set $VETH_SRC_1 up

echo "[+] Configuring firewall mark lookup tables..."
ip route add default dev $VETH_1_1 table 101
# Check if the rule already exists
if ! ip rule show | grep -q "fwmark $FW_MARK_1 lookup 101"; then
    ip rule add fwmark $FW_MARK_1 lookup 101
fi

echo "[+] Redirecting all PREROUTING traffic from $VETH_1_1 to NFQueue $NFQUEUE_NUM..."
iptables -t mangle -A OUTPUT -j NFQUEUE --queue-num $NFQUEUE_NUM --queue-bypass

echo "[+] Setup complete!"
ip -s link show $VETH_1_1
ip netns exec $NS_1 ip -s link show $VETH_1_2
ip rule list
ip route show table 101
iptables -t mangle -L

echo "[+] Generating traffic..."
ping 8.8.8.8
