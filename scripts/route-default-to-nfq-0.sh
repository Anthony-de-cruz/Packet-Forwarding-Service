#!/bin/bash
# Redirect all default gateway traffic to NF queue 0.
sudo iptables -A OUTPUT -o $(ip route | grep default | awk '{print $5}' | head -n1) -j NFQUEUE --queue-num 0
