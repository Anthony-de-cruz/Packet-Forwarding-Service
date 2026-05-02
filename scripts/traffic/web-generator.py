#!/usr/bin/env python3

import random
import subprocess
import threading
import time


URLS = [
    "https://www.youtube.com/",
    "https://www.youtube.com/results?search_query=cat+videos",
    "https://x.com/",
    "https://twitter.com/",
    "https://www.instagram.com/"
]

DURATION = 60000
CONCURRENCY = 2
UE_INTERFACES = ["uesimtun0"]
REQUEST_TIMEOUT = 15
MIN_SLEEP = 1.0
MAX_SLEEP = 4.0


def run_curl(url: str, iface: str, timeout: int) -> tuple[int, str]:
    cmd = [
        "curl",
        "--silent",
        "--show-error",
        "--location",
        "--max-time",
        str(timeout),
        "--interface",
        iface,
        "--output",
        "/dev/null",
        "--write-out",
        "code=%{http_code} time=%{time_total} size=%{size_download} url=%{url_effective}",
        url
    ]

    completed = subprocess.run(
        cmd,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    output = completed.stdout.strip()
    if completed.stderr.strip():
        output = f"{output} error={completed.stderr.strip()}"

    return completed.returncode, output


def worker_loop(
    worker_id: int,
    iface: str,
    end_time: float,
    timeout: int,
    min_sleep: float,
    max_sleep: float,
    print_lock: threading.Lock,
) -> None:
    while time.monotonic() < end_time:
        url = random.choice(URLS)
        status, output = run_curl(url, iface, timeout)

        with print_lock:
            print(
                f"worker={worker_id} iface={iface} "
                f"curl_status={status} {output}",
                flush=True,
            )

        time.sleep(random.uniform(min_sleep, max_sleep))


def main() -> None:
    end_time = time.monotonic() + DURATION
    print_lock = threading.Lock()
    threads: list[threading.Thread] = []

    print(
        f"[+] Starting web traffic for {DURATION}s with "
        f"{CONCURRENCY} workers per interface",
        flush=True,
    )

    for iface in UE_INTERFACES:
        for worker_id in range(1, CONCURRENCY + 1):
            thread = threading.Thread(
                target=worker_loop,
                args=(
                    worker_id,
                    iface,
                    end_time,
                    REQUEST_TIMEOUT,
                    MIN_SLEEP,
                    MAX_SLEEP,
                    print_lock,
                ),
            )
            thread.start()
            threads.append(thread)

    for thread in threads:
        thread.join()

    print("[!] Web traffic complete", flush=True)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
