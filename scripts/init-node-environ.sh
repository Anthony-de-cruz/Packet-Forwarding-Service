#!/usr/bin/env bash

# Fail mode
set -euo pipefail

echo "-> Enabling IPv4 forwarding..."
sysctl -w net.ipv4.ip_forward=1

echo "-> Configuring POSTROUTING nat ip table..."
iptables -t nat -A POSTROUTING -j MASQUERADE

echo "-> Forwarding node setup complete!"
