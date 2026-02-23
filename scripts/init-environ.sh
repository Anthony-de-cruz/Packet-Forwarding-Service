#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/reset.sh"

echo "[+] Creating network namespaces..."
ip netns add $NS_1
#ip netns add $NS2

echo "[+] Creating veth pair..."
ip link add $VETH_1_1 type veth peer name $VETH_1_2

echo "[+] Assigning veth interfaces to namespaces..."
ip link set $VETH_1_2 netns $NS_1

echo "[+] Configuring IP addresses..."
ip addr add $VETH_1_1_IP dev $VETH_1_1
ip netns exec $NS_1 ip addr add $VETH_1_1_IP dev $VETH_1_2

echo "[+] Bringing interfaces up..."
ip link set $VETH_1_1 up
ip netns exec $NS_1 ip link set lo up
ip netns exec $NS_1 ip link set $VETH_1_2 up

echo "[+] Enabling IPv4 forwarding on host..."
sysctl -w net.ipv4.ip_forward=1 > /dev/null

echo "[+] Configuring firewall mark lookup tables..."
ip rule add fwmark 0x1 lookup 1001
ip route add default dev $VETH_1_1 table 1001

echo "[+] Setup complete!"
ip -s link show $VETH_1_1
ip netns exec $NS_1 ip -s link show $VETH_1_2

