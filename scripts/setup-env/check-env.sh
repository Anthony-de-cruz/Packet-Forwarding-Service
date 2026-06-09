#!/usr/bin/env bash

# Check host-side networking state for the VM bridge, router policy, and nodes.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

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

check_iptables_absent() {
    local table="$1"
    shift

    if sudo iptables -t "$table" -C "$@" >/dev/null 2>&1; then
        fail "iptables $table rule should be absent: $*"
    else
        pass "iptables $table rule absent: $*"
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

check_main_route() {
    local subnet="$1"
    local next_hop="$2"
    local iface="$3"

    if ip route show "$subnet" | grep -q "$subnet via $next_hop dev $iface"; then
        pass "main route sends $subnet via $next_hop dev $iface"
    else
        fail "main route missing $subnet via $next_hop dev $iface"
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

echo "[+] Checking routed UE pool path..."
check_main_route "$VM_UE_POOL_SUBNET" "$VM_OPEN5GS_IP" "$VM_BRIDGE_NAME"
check_iptables nat POSTROUTING -s "$VM_UE_POOL_SUBNET" -o "$HOST_LAN_IFACE" -m mark --mark 0x0 -j MASQUERADE
check_iptables_absent nat POSTROUTING -s "$VM_UE_POOL_SUBNET" -o "$HOST_LAN_IFACE" -j MASQUERADE
check_iptables_absent nat POSTROUTING -s "$VM_UE_POOL_SUBNET" -j MASQUERADE
check_iptables filter FORWARD -i "$VM_BRIDGE_NAME" -o "$HOST_LAN_IFACE" -s "$VM_UE_POOL_SUBNET" -j ACCEPT
check_iptables filter FORWARD -i "$HOST_LAN_IFACE" -o "$VM_BRIDGE_NAME" -d "$VM_UE_POOL_SUBNET" -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

echo "[+] Checking router NFQUEUE rule..."
check_iptables mangle PREROUTING -i "$VM_BRIDGE_NAME" -j NFQUEUE --queue-num "$HOST_NFQUEUE_NUM" --queue-bypass

echo "[+] Checking policy routing..."
check_fwmark_rule "$HOST_FW_MARK_1" "$HOST_ROUTE_TABLE_1"
check_route_table "$HOST_ROUTE_TABLE_1" "$NODE_HOP_1"
check_fwmark_rule "$HOST_FW_MARK_2" "$HOST_ROUTE_TABLE_2"
check_route_table "$HOST_ROUTE_TABLE_2" "$NODE_HOP_2"
check_fwmark_rule "$HOST_FW_MARK_3" "$HOST_ROUTE_TABLE_3"
check_route_table "$HOST_ROUTE_TABLE_3" "$NODE_HOP_3"

echo "[+] Checking VM reachability..."
check_ping "$VM_OPEN5GS_IP" "Open5GS VM"
check_ping "$VM_UERANSIM_IP" "UERANSIM VM"

echo "[+] Checking forwarding node reachability..."
check_ping "$NODE_HOP_1" "node hop 1"
check_ping "$NODE_HOP_2" "node hop 2"
check_ping "$NODE_HOP_3" "node hop 3"

echo "[+] Summary: $checks checks, $failures failures, $warnings warnings"

if (( failures > 0 )); then
    exit 1
fi
