#!/usr/bin/env bash

# Script to setup hardware environment.

# Fail mode
# set -euo pipefail

# Grab config values.
# source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/router-reset-env.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1
sudo modprobe br_netfilter
sudo sysctl -w net.bridge.bridge-nf-call-iptables=1

echo "[+] Configuring firewall mark policy routes..."
sudo ip rule add fwmark $HOST_FW_MARK_1 table $HOST_ROUTE_TABLE_1
sudo ip route add default via $NODE_HOP_1 table $HOST_ROUTE_TABLE_1

sudo ip rule add fwmark $HOST_FW_MARK_2 table $HOST_ROUTE_TABLE_2
sudo ip route add default via $NODE_HOP_2 table $HOST_ROUTE_TABLE_2

sudo ip rule add fwmark $HOST_FW_MARK_3 table $HOST_ROUTE_TABLE_3
sudo ip route add default via $NODE_HOP_3 table $HOST_ROUTE_TABLE_3

sudo ip rule add fwmark $HOST_FW_MARK_4 table $HOST_ROUTE_TABLE_4
sudo ip route add default via $NODE_HOP_4 table $HOST_ROUTE_TABLE_4

echo "[+] Redirecting bridge ingress traffic to NFQueue $NFQUEUE_NUM before routing..."
# sudo iptables -t mangle -A OUTPUT \
#     -p udp --dport 9000 \
#     -j NFQUEUE --queue-num $NFQUEUE_NUM --queue-bypass
    # -p icmp --icmp-type echo-request \
    # -j LOG --log-prefix "MARKED_UDP: " \
sudo iptables -t mangle -A PREROUTING \
    -i "$VM_BRIDGE_NAME" \
    -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass

echo "[!] Router setup complete!"
sudo iptables -t mangle -nvL PREROUTING --line-numbers
sudo ip rule list
sudo ip route show table $HOST_ROUTE_TABLE_1
sudo ip route show table $HOST_ROUTE_TABLE_2
sudo ip route show table $HOST_ROUTE_TABLE_3
sudo ip route show table $HOST_ROUTE_TABLE_4
