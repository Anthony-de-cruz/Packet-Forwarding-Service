#!/usr/bin/env bash

# Fail mode
set -euo pipefail

echo "[+] Enabling IPv4 forwarding..."
sudo sysctl -w net.ipv4.ip_forward=1

echo "[+] Configuring POSTROUTING nat ip table..."
sudo iptables -t nat -A POSTROUTING -j MASQUERADE

echo "[!] Forwarding node setup complete!"
