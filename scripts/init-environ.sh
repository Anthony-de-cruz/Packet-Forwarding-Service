#!/usr/bin/env bash

# Script to setup hardware environment.

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# Run reset script with absolute path.
"$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/reset.sh"

echo "[+] Enabling IPv4 forwarding on host..."
sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring firewall mark DNAT rules..."
iptables -t nat -A OUTPUT \
    -m mark --mark $FW_MARK_1 \
    -j DNAT --to-destination $DEST_IP_1 
iptables -t nat -A OUTPUT \
    -m mark --mark $FW_MARK_2 \
    -j DNAT --to-destination $DEST_IP_2
iptables -t nat -A OUTPUT \
    -m mark --mark $FW_MARK_3 \
    -j DNAT --to-destination $DEST_IP_3
iptables -t nat -A OUTPUT \
    -m mark --mark $FW_MARK_4 \
    -j DNAT --to-destination $DEST_IP_4

# Should probably update with conntrack to try an avoid marked packets.

echo "[+] Redirecting all OUTPUT UDP dest port 9000 traffic to NFQueue $NFQUEUE_NUM..."
iptables -t mangle -A OUTPUT \
    -p udp --dport 9000 \
    -j NFQUEUE --queue-num $NFQUEUE_NUM --queue-bypass
    # -p icmp --icmp-type echo-request \
    # -j LOG --log-prefix "MARKED_UDP: " \

echo "[+] Setup complete!"
iptables -t mangle -nvL OUTPUT --line-numbers # Print out mangle table.
iptables -t nat -nvL OUTPUT --line-numbers # Print out nat table.
