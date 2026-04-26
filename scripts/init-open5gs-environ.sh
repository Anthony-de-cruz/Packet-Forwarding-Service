#!/usr/bin/env bash

# Run this inside of the open5gs VM.

# Fail mode
set -euo pipefail

sudo sysctl -w net.ipv4.ip_forward=1
sudo iptables -t nat -A POSTROUTING -s 10.45.0.0/16 ! -o ogstun -j MASQUERADE
sudo iptables -I FORWARD 1 -i ogstun -j ACCEPT
sudo iptables -I FORWARD 1 -o ogstun -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
