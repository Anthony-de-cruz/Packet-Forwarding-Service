#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Redirect all outbound default gateway traffic to NF queue 0.
sudo iptables -A OUTPUT -o $(ip route show default | awk '{print $5; exit}') -j NFQUEUE --queue-num 0
