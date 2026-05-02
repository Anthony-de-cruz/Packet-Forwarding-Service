#!/usr/bin/env python3

import socket
import time
from pathlib import Path

from PIL import Image

SAMPLE_TYPE_DIR_PATHS = [
    "../../Packet-Classifier/data/test/google-meet",
    "../../Packet-Classifier/data/test/instagram",
    "../../Packet-Classifier/data/test/tiktok",
    "../../Packet-Classifier/data/test/twitter",
    "../../Packet-Classifier/data/test/youtube",
]
DEST_ADDR = "192.168.0.54", 9000

CHUNK_SIZE = 1200
DELAY_SECONDS = 0.1
SAMPLE_SIZE = (28, 28)


def extract_samples(sample_type_dirs: list[str]) -> list[list[bytes]]:
    # Find all samples.
    sample_type_paths = []
    for dir in sample_type_dirs:
        sample_type_paths.append(
            sorted(
                child
                for child in Path(dir).iterdir()
                if child.is_file() and child.suffix == ".png"
            )[:10]
        )

    # Extract and preprocess each sample.
    sample_bytes = []
    for sample_dir in sample_type_paths:
        for sample_path in sample_dir:
            with Image.open(sample_path) as image:
                sample_bytes.append(image.convert("L").resize(SAMPLE_SIZE).tobytes())

    return sample_bytes


def main() -> None:
    sample_set = extract_samples(SAMPLE_TYPE_DIR_PATHS)

    while True:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            for samples in sample_set:
                for sample in samples:
                    packet_count = 0
                    for offset in range(0, len(sample), CHUNK_SIZE):
                        chunk = sample[offset : offset + CHUNK_SIZE]
                        sock.sendto(chunk, DEST_ADDR)
                        packet_count += 1
                        time.sleep(DELAY_SECONDS)

                    print(
                        f"sent {sample} as {packet_count} UDP packet(s) "
                        f"to {DEST_ADDR[0]}:{DEST_ADDR[1]}"
                    )


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
