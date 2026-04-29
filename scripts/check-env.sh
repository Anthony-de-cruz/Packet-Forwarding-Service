#!/usr/bin/env bash

# Check host-side networking state for the VM bridge, router policy, and nodes.

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

OPEN5GS_VM_IP="${OPEN5GS_VM_IP:-10.0.0.2}"
UERANSIM_VM_IP="${UERANSIM_VM_IP:-10.0.0.3}"

checks=0
failures=0
warnings=0

pass() {
    checks=$((checks + 1))
    echo "[PASS] $*"
}

fail() {
    checks=$((checks + 1))
    failures=$((failures + 1))
    echo "[FAIL] $*"
}

warn() {
    warnings=$((warnings + 1))
    echo "[WARN] $*"
}

check_link() {
    local iface="$1"
    local label="$2"

    if ip link show "$iface" >/dev/null 2>&1; then
        pass "$label interface exists: $iface"
    else
        fail "$label interface missing: $iface"
    fi
}

check_ping() {
    local ip="$1"
    local label="$2"

    if ping -c 1 -W 1 "$ip" >/dev/null 2>&1; then
        pass "$label reachable: $ip"
    else
        warn "$label not reachable: $ip"
    fi
}

check_iptables() {
    local table="$1"
    shift

    if sudo iptables -t "$table" -C "$@" >/dev/null 2>&1; then
        pass "iptables $table rule present: $*"
    else
        fail "iptables $table rule missing: $*"
    fi
}

check_route_table() {
    local table="$1"
    local next_hop="$2"

    if ip route show table "$table" | grep -q "default via $next_hop"; then
        pass "route table $table has default via $next_hop"
    else
        fail "route table $table missing default via $next_hop"
    fi
}

check_fwmark_rule() {
    local mark="$1"
    local table="$2"

    if ip rule show | grep -q "fwmark $mark lookup $table"; then
        pass "fwmark $mark routes to table $table"
    else
        fail "fwmark $mark missing rule for table $table"
    fi
}

echo "[+] Checking configured interfaces..."
check_link "$HOST_LAN_IFACE" "host LAN"
check_link "$VM_BRIDGE_NAME" "VM bridge"

echo "[+] Checking VM bridge address..."
if ip -4 addr show dev "$VM_BRIDGE_NAME" 2>/dev/null | grep -q "$VM_BRIDGE_ADDR"; then
    pass "$VM_BRIDGE_NAME has address $VM_BRIDGE_ADDR"
else
    fail "$VM_BRIDGE_NAME missing address $VM_BRIDGE_ADDR"
fi

echo "[+] Checking IPv4 forwarding..."
if [[ "$(sysctl -n net.ipv4.ip_forward)" == "1" ]]; then
    pass "IPv4 forwarding enabled"
else
    fail "IPv4 forwarding disabled"
fi

echo "[+] Checking VM bridge NAT and forwarding rules..."
check_iptables nat POSTROUTING -s "$VM_BRIDGE_SUBNET" -o "$HOST_LAN_IFACE" -j MASQUERADE
check_iptables filter FORWARD -i "$VM_BRIDGE_NAME" -o "$HOST_LAN_IFACE" -s "$VM_BRIDGE_SUBNET" -j ACCEPT
check_iptables filter FORWARD -i "$HOST_LAN_IFACE" -o "$VM_BRIDGE_NAME" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

echo "[+] Checking router NFQUEUE rule..."
check_iptables mangle PREROUTING -i "$VM_BRIDGE_NAME" -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass

echo "[+] Checking policy routing..."
check_fwmark_rule "$HOST_FW_MARK_1" "$HOST_ROUTE_TABLE_1"
check_route_table "$HOST_ROUTE_TABLE_1" "$NODE_HOP_1"
check_fwmark_rule "$HOST_FW_MARK_2" "$HOST_ROUTE_TABLE_2"
check_route_table "$HOST_ROUTE_TABLE_2" "$NODE_HOP_2"
check_fwmark_rule "$HOST_FW_MARK_3" "$HOST_ROUTE_TABLE_3"
check_route_table "$HOST_ROUTE_TABLE_3" "$NODE_HOP_3"
check_fwmark_rule "$HOST_FW_MARK_4" "$HOST_ROUTE_TABLE_4"
check_route_table "$HOST_ROUTE_TABLE_4" "$NODE_HOP_4"

echo "[+] Checking VM reachability..."
check_ping "$OPEN5GS_VM_IP" "Open5GS VM"
check_ping "$UERANSIM_VM_IP" "UERANSIM VM"

echo "[+] Checking forwarding node reachability..."
check_ping "$NODE_HOP_1" "node hop 1"
check_ping "$NODE_HOP_2" "node hop 2"
check_ping "$NODE_HOP_3" "node hop 3"
check_ping "$NODE_HOP_4" "node hop 4"

echo "[+] Summary: $checks checks, $failures failures, $warnings warnings"

if (( failures > 0 )); then
    exit 1
fi
