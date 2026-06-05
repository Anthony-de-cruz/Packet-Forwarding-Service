#!/usr/bin/env python3

import random
import socket
import time
from pathlib import Path

from PIL import Image

from utils import open_csv_writer, utc_timestamp


UDP_BIND_INTERFACE = "uesimtun0"
UDP_DEST_ADDR = "192.168.0.52", 9000
# UDP_DEST_ADDR = "127.0.0.1", 9000

SAMPLE_TYPE_DIR_PATHS = [
    "../Packet-Classifier/data/test/google-meet",
    "../Packet-Classifier/data/test/instagram",
    "../Packet-Classifier/data/test/tiktok",
    "../Packet-Classifier/data/test/twitter",
    "../Packet-Classifier/data/test/youtube",
]
CHUNK_SIZE = 1200
DELAY_SECONDS = 0.01
ECHO_TIMEOUT_SECONDS = 2.0
LOG_INTERVAL_SECONDS = 1.0
SAMPLE_SIZE = (28, 28)
LOG_PATH = Path("../out/udp-echo-client-latency.csv")

CSV_COLUMNS = [
    "timestamp_utc",
    "timestamp_unix_ms",
    "avg_latency_ms",
]


def write_log(writer, avg_latency_ms: float) -> None:
    time_text, time_ms = utc_timestamp()
    writer.writerow(
        {
            "timestamp_utc": time_text,
            "timestamp_unix_ms": time_ms,
            "avg_latency_ms": f"{avg_latency_ms:.3f}",
        }
    )


def extract_samples(sample_type_dirs: list[str]) -> list[list[bytes]]:
    # Find all samples.
    sample_set_paths: list[list[str]] = []
    for dir in sample_type_dirs:
        sample_set_paths.append(
            sorted(
                str(child)
                for child in Path(dir).iterdir()
                if child.is_file() and child.suffix == ".png"
            )[:50]
        )

    # Extract and preprocess each sample.
    sample_bytes: list[list[bytes]] = []
    for sample_dir in sample_set_paths:
        sample_type_bytes: list[bytes] = []

        for sample_path in sample_dir:
            with Image.open(sample_path) as image:
                sample_type_bytes.append(
                    image.convert("L").resize(SAMPLE_SIZE).tobytes()
                )
        sample_bytes.append(sample_type_bytes)

    return sample_bytes

def main() -> None:
    sample_set = extract_samples(SAMPLE_TYPE_DIR_PATHS)
    latency_total_ms = 0.0
    latency_samples = 0
    total_packets = 0
    total_timeouts = 0
    total_mismatches = 0

    log_file, log_writer = open_csv_writer(LOG_PATH, CSV_COLUMNS)
    try:
        last_log = time.monotonic()
        while True:
            for samples in sample_set:
                print("Opening socket...")
                with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                    print(f"Binding socket to {UDP_BIND_INTERFACE}...")
                    sock.setsockopt(
                        socket.SOL_SOCKET,
                        socket.SO_BINDTODEVICE,
                        UDP_BIND_INTERFACE.encode() + b"\0",
                    )
                    random.shuffle(samples)
                    sock.settimeout(ECHO_TIMEOUT_SECONDS)
                    for sample in samples:
                        start_time = time.perf_counter()
                        sock.sendto(sample, UDP_DEST_ADDR)
                        total_packets += 1
                        try:
                            echo, address = sock.recvfrom(CHUNK_SIZE)
                        except socket.timeout:
                            total_timeouts += 1
                            print(
                                f"TX UDP to {UDP_DEST_ADDR[0]}:{UDP_DEST_ADDR[1]}, "
                                f"echo timeout after {ECHO_TIMEOUT_SECONDS:.1f}s, "
                                f"sent={total_packets}, "
                                f"timeouts={total_timeouts}, "
                                f"mismatches={total_mismatches}"
                            )
                        else:
                            round_trip_ms = (time.perf_counter() - start_time) * 1000
                            if echo == sample:
                                latency_total_ms += round_trip_ms
                                latency_samples += 1
                                print(
                                    f"TX/RX UDP via {address[0]}:{address[1]}, "
                                    f"round_trip={round_trip_ms:.2f}ms, "
                                    f"sent={total_packets}, "
                                    f"timeouts={total_timeouts}, "
                                    f"mismatches={total_mismatches}"
                                )
                            else:
                                total_mismatches += 1
                                print(
                                    f"TX UDP to {UDP_DEST_ADDR[0]}:{UDP_DEST_ADDR[1]}, "
                                    f"echo mismatch from {address[0]}:{address[1]} "
                                    f"after {round_trip_ms:.2f}ms, "
                                    f"sent={total_packets}, "
                                    f"timeouts={total_timeouts}, "
                                    f"mismatches={total_mismatches}"
                                )

                        now = time.monotonic()
                        if now - last_log >= LOG_INTERVAL_SECONDS:
                            if latency_samples > 0:
                                avg_latency_ms = latency_total_ms / latency_samples
                                write_log(log_writer, avg_latency_ms)
                                log_file.flush()
                                print(f"avg_latency={avg_latency_ms:.2f}ms")
                            latency_total_ms = 0.0
                            latency_samples = 0
                            last_log = now
                        time.sleep(DELAY_SECONDS)
    finally:
        log_file.close()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
