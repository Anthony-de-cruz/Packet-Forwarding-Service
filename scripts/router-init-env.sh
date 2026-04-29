#!/usr/bin/env bash

# Script to setup hardware environment.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

add_fwmark_rule() {
    local mark="$1"
    local table="$2"

    if ! ip rule show | grep -q "fwmark $mark lookup $table"; then
        sudo ip rule add fwmark "$mark" table "$table"
    fi
}

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/router-reset-env.sh"

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1
sudo modprobe br_netfilter
sudo sysctl -w net.bridge.bridge-nf-call-iptables=1

echo "[+] Configuring firewall mark policy routes..."
add_fwmark_rule "$HOST_FW_MARK_1" "$HOST_ROUTE_TABLE_1"
sudo ip route replace default via "$NODE_HOP_1" table "$HOST_ROUTE_TABLE_1"

add_fwmark_rule "$HOST_FW_MARK_2" "$HOST_ROUTE_TABLE_2"
sudo ip route replace default via "$NODE_HOP_2" table "$HOST_ROUTE_TABLE_2"

add_fwmark_rule "$HOST_FW_MARK_3" "$HOST_ROUTE_TABLE_3"
sudo ip route replace default via "$NODE_HOP_3" table "$HOST_ROUTE_TABLE_3"

add_fwmark_rule "$HOST_FW_MARK_4" "$HOST_ROUTE_TABLE_4"
sudo ip route replace default via "$NODE_HOP_4" table "$HOST_ROUTE_TABLE_4"

echo "[+] Redirecting bridge ingress traffic to NFQueue $HOST_NFQUEUE_NUM before routing..."
# sudo iptables -t mangle -A OUTPUT \
#     -p udp --dport 9000 \
#     -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass
    # -p icmp --icmp-type echo-request \
    # -j LOG --log-prefix "MARKED_UDP: " \
sudo iptables -t mangle -C PREROUTING -i "$VM_BRIDGE_NAME" \
    -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass 2>/dev/null || \
sudo iptables -t mangle -A PREROUTING -i "$VM_BRIDGE_NAME" \
    -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass

echo "[!] Router setup complete!"
sudo iptables -t mangle -nvL PREROUTING --line-numbers
sudo ip rule list
sudo ip route show table "$HOST_ROUTE_TABLE_1"
sudo ip route show table "$HOST_ROUTE_TABLE_2"
sudo ip route show table "$HOST_ROUTE_TABLE_3"
sudo ip route show table "$HOST_ROUTE_TABLE_4"
