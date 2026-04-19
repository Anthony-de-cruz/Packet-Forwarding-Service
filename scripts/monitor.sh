#!/usr/bin/env bash

# Fail mode
set -euo pipefail

# Grab config values.
source "$(dirname "${BASH_SOURCE[0]}")/config.sh" -euo pipefail

clear
while true
do
    printf "\033[H"   # move cursor to top-left
    #ip -s link show
    #ip netns exec "$NS_1" ip -s link show "$VETH_1_2"
    #tcpdump -i default
    tcpdump -i $VETH_1_1
    ip netns exec "$NS_1" tcpdump -i $VETH_1_2
    sleep 1
done
