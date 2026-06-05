#!/usr/bin/env bash

scp user@192.168.0.52:~/Packet-Forwarding-Service/out/udp-echo-server.csv out/rpi-2-udp-echo-server.csv
scp user@10.0.0.3:~/Packet-Forwarding-Service/out/udp-echo-client-latency.csv out/vm-udp-echo-client.csv
