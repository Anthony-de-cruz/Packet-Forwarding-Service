#!/usr/bin/env bash

# Change the highlighted values according to your setup.
#
# For testing note that most WAPs wont implement/support L2 NAT.
# You will most likely need a switch.

HOST_IP="192.168.0.49" # YOUR HOST IP HERE.

HOST_NFQUEUE_NUM=10

HOST_FW_MARK_1=0x801
HOST_FW_MARK_2=0x802
HOST_FW_MARK_3=0x803

# Routing tables selected by firewall marks.
HOST_ROUTE_TABLE_1=101
HOST_ROUTE_TABLE_2=102
HOST_ROUTE_TABLE_3=103

HOST_LAN_IFACE="enp0s13f0u4" # YOUR LAN INTERFACE HERE.

# Next-hop nodes for hardware forwarding.
NODE_HOP_1="192.168.0.50" # RPI-0 ADDRESS HERE.
NODE_HOP_2="192.168.0.51" # RPI-1 ADDRESS HERE.
NODE_HOP_3="192.168.0.51" # RPI-2 ADDRESS HERE.

# Interface used by forwarding nodes to reach the host and upstream network.
NODE_LAN_IFACE="eth0"

# Network bridge for Open5GS/UERANSIM.
VM_BRIDGE_NAME="br-5g"
VM_BRIDGE_SUBNET="10.0.0.0/24"
VM_BRIDGE_ADDR="10.0.0.1/24"
VM_OPEN5GS_IP="10.0.0.2"
VM_UERANSIM_IP="10.0.0.3"
VM_UE_POOL_SUBNET="10.45.0.0/16"

# iperf3 configuration.
IPERF3_SERVER_IP="192.168.0.53" # YOUR IPERF3 SERVER IP HERE.
IPERF3_BIND_IP="10.45.0.2" # ANY UE TAP IP HERE.
IPERF3_CONN_INTERVAL=0.1
IPERF3_CONN_LIFETIME=10
