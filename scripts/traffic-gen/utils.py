import csv
from datetime import UTC, datetime
from pathlib import Path
from typing import TextIO


ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "out"

UDP_LISTEN_ADDR = "0.0.0.0", 9000


def utc_timestamp(timespec: str = "milliseconds") -> tuple[str, int]:
    now = datetime.now(UTC)
    return now.isoformat(timespec=timespec).replace("+00:00", "Z"), int(
        now.timestamp() * 1000
    )


def load_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as file:
        return list(csv.DictReader(file))


def open_csv_writer(
    path: Path, fieldnames: list[str], mode: str = "a"
) -> tuple[TextIO, csv.DictWriter]:
    path.parent.mkdir(parents=True, exist_ok=True)
    file = path.open(mode, newline="")
    writer = csv.DictWriter(file, fieldnames=fieldnames)

    if mode == "w" or path.stat().st_size == 0:
        writer.writeheader()
        file.flush()

    return file, writer
