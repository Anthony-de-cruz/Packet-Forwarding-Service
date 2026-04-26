#!/usr/bin/env bash


# Configuration
NFQUEUE_NUM=10

FW_MARK_1=0x801
FW_MARK_2=0x802
FW_MARK_3=0x803
FW_MARK_4=0x804

# Routing tables selected by firewall marks.
ROUTE_TABLE_1=101
ROUTE_TABLE_2=102
ROUTE_TABLE_3=103
ROUTE_TABLE_4=104

# Next-hop nodes for hardware forwarding.
NEXT_HOP_1="192.168.0.45"
NEXT_HOP_2="192.168.0.181"
NEXT_HOP_3="192.168.0.45"
NEXT_HOP_4="192.168.0.181"

# Network bridge for 
BRIDGE_NAME="br-5g"
BRIDGE_SUBNET="10.0.0.0/24"
BRIDGE_ADDR="10.0.0.1/24"
LAN_IFACE="enp0s13f0u4" # YOUR LAN INTERFACE HERE (run `ip a` to list them)

# Virtual ports and namespaces for local testing.
VETH_SRC_1="veth-test-src-1"
VETH_SRC_2="veth-test-src-2"
VETH_SRC_1_IP="192.168.0.249/24"
VETH_SRC_2_IP="192.168.0.248/24"
NS_SRC="ns-test-src"

VETH_1_1="veth-test-1-1"
VETH_1_2="veth-test-1-2"
VETH_2_1="veth-test-2-1"
VETH_2_2="veth-test-2-2"

NS_1="ns-test-1"

VETH_1_1_IP="192.168.0.201/24"
VETH_1_2_IP="192.168.0.202/24"
VETH_2_1_IP="192.168.0.203/24"
VETH_2_2_IP="192.168.0.204/24"
