#!/usr/bin/env bash

set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
    exec sudo -E bash "$0" "$@"
fi

echo "[+] Restarting Open5GS stack..."

mapfile -t open5gs_services < <(
    systemctl list-unit-files 'open5gs-*.service' --type=service --no-legend \
        | awk '{print $1}' \
        | sort
)

if [[ "${#open5gs_services[@]}" -eq 0 ]]; then
    echo "[!] No Open5GS systemd services were found."
    exit 1
fi

systemctl restart mongod.service 2>/dev/null || true

for service in "${open5gs_services[@]}"; do
    systemctl restart "$service"
done

for service in "${open5gs_services[@]}"; do
    systemctl --no-pager --lines=0 status "$service"
done

echo "[+] Done"
