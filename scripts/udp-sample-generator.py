#!/usr/bin/env python3

import socket
import time
from pathlib import Path


IMAGE_PATH = "./sample"
DEST_HOST = "8.8.8.8" # This serves as a dummy destination.
DEST_PORT = 9000
CHUNK_SIZE = 1200
DELAY_SECONDS = 0.005


def get_images(path: str) -> list[Path]:
    source = Path(path)
    if source.is_file():
        return [source]

    return sorted(
        child for child in source.iterdir() if child.is_file() and child.suffix == ".png"
    )


def main() -> None:
    images = get_images(IMAGE_PATH)
    if not images:
        raise SystemExit(f"No images found at {IMAGE_PATH}")

    address = (DEST_HOST, DEST_PORT)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    try:
        for image in images:
            data = image.read_bytes()
            packet_count = 0

            for offset in range(0, len(data), CHUNK_SIZE):
                chunk = data[offset : offset + CHUNK_SIZE]
                sock.sendto(chunk, address)
                packet_count += 1
                time.sleep(DELAY_SECONDS)

            print(f"sent {image} as {packet_count} UDP packet(s)")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
