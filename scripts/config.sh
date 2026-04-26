#!/usr/bin/env bash


# Configuration
HOST_IP="192.168.0.49" # Change to the server hosts IP.

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
NEXT_HOP_1="192.168.0.51"
NEXT_HOP_2="192.168.0.51"
NEXT_HOP_3="192.168.0.51"
NEXT_HOP_4="192.168.0.51"

# Network bridge for 
BRIDGE_NAME="br-5g"
BRIDGE_SUBNET="10.0.0.0/24"
BRIDGE_ADDR="10.0.0.1/24"
LAN_IFACE="enp0s13f0u4" # YOUR LAN INTERFACE HERE (run `ip a` to list them)
