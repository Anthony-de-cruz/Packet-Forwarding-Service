#!/usr/bin/env python3

import socket
import time
from pathlib import Path

SAMPLES = (
    ("./samples/google-meet", ("8.8.8.8", 9000)),
    ("./samples/instagram", ("7.7.7.7", 9000)),
    ("./samples/tiktok", ("6.6.6.6", 9000)),
    ("./samples/twitter", ("5.5.5.5", 9000)),
    ("./samples/youtube", ("4.4.4.4", 9000))
)
CHUNK_SIZE = 1200
DELAY_SECONDS = 0.00005


def get_images(path: str) -> list[Path]:
    source = Path(path)
    if source.is_file():
        return [source]

    return sorted(
        child for child in source.iterdir() if child.is_file() and child.suffix == ".png"
    )


def main() -> None:
    traffic = []
    for path, addr in SAMPLES:
        traffic.append((get_images(path), addr))
    
    while True:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            for samples, addr in traffic:
                for sample in samples:
                    data = sample.read_bytes()
                    packet_count = 0
                    for offset in range(0, len(data), CHUNK_SIZE):
                        chunk = data[offset : offset + CHUNK_SIZE]
                        sock.sendto(chunk, addr)
                        packet_count += 1
                        time.sleep(DELAY_SECONDS)

                    print(
                        f"sent {sample} as {packet_count} UDP packet(s) "
                        f"to {addr[0]}:{addr[1]}")

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
