#!/usr/bin/env bash

# Script to setup hardware environment.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/reset.sh"

echo "[+] Enabling IPv4 forwarding on host..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring firewall mark policy routes..."
sudo ip rule add fwmark $FW_MARK_1 table $ROUTE_TABLE_1
sudo ip route add default via $NEXT_HOP_1 table $ROUTE_TABLE_1

sudo ip rule add fwmark $FW_MARK_2 table $ROUTE_TABLE_2
sudo ip route add default via $NEXT_HOP_2 table $ROUTE_TABLE_2

sudo ip rule add fwmark $FW_MARK_3 table $ROUTE_TABLE_3
sudo ip route add default via $NEXT_HOP_3 table $ROUTE_TABLE_3

sudo ip rule add fwmark $FW_MARK_4 table $ROUTE_TABLE_4
sudo ip route add default via $NEXT_HOP_4 table $ROUTE_TABLE_4

echo "[+] Redirecting all OUTPUT UDP dest port 9000 traffic to NFQueue $NFQUEUE_NUM..."
sudo iptables -t mangle -A OUTPUT \
    -p udp --dport 9000 \
    -j NFQUEUE --queue-num $NFQUEUE_NUM --queue-bypass
    # -p icmp --icmp-type echo-request \
    # -j LOG --log-prefix "MARKED_UDP: " \

echo "[!] Router setup complete!"
sudo iptables -t mangle -nvL OUTPUT --line-numbers # Print out mangle table.
sudo ip rule list
sudo ip route show table $ROUTE_TABLE_1
sudo ip route show table $ROUTE_TABLE_2
sudo ip route show table $ROUTE_TABLE_3
sudo ip route show table $ROUTE_TABLE_4
