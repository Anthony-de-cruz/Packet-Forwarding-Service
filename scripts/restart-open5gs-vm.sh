#!/usr/bin/env bash

set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
    exec sudo -E bash "$0" "$@"
fi

echo "[+] Restarting Open5GS stack on $(hostname)..."

restart_if_present() {
    local service="$1"

    if systemctl list-unit-files --type=service --no-legend | awk '{print $1}' | grep -Fxq "$service"; then
        echo "[+] Restarting $service"
        systemctl restart "$service"
    else
        echo "[-] Skipping $service (not installed)"
    fi
}

# Subscriber data backend for Open5GS.
restart_if_present "mongod.service"

mapfile -t open5gs_services < <(
    systemctl list-unit-files 'open5gs-*.service' --type=service --no-legend \
        | awk '{print $1}' \
        | sort
)

if [[ "${#open5gs_services[@]}" -eq 0 ]]; then
    echo "[!] No Open5GS systemd services were found."
    exit 1
fi

for service in "${open5gs_services[@]}"; do
    echo "[+] Restarting $service"
    systemctl restart "$service"
done

echo
echo "[+] Service summary"
for service in "mongod.service" "${open5gs_services[@]}"; do
    if systemctl list-unit-files --type=service --no-legend | awk '{print $1}' | grep -Fxq "$service"; then
        state="$(systemctl is-active "$service" || true)"
        printf "    %-28s %s\n" "$service" "$state"
    fi
done

echo
echo "[+] Done"
