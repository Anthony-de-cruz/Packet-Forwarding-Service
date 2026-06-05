#!/usr/bin/env python3

import os

os.environ.setdefault("MPLCONFIGDIR", "/tmp/matplotlib")

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

from utils import OUT_DIR, load_csv

INPUT_CSV = OUT_DIR / "processed-timeseries.csv"
NODE_PACKET_RATES_TIME_WINDOW = (5 * 60, 6 * 60)


def title(column):
    return column.removeprefix("class_").replace("_", " ").title()


def save_plot(filename):
    output = OUT_DIR / filename
    output.parent.mkdir(parents=True, exist_ok=True)
    plt.tight_layout()
    plt.savefig(output, dpi=150)
    plt.close()
    print(f"Wrote {output}")


def line_chart(rows, column, filename, chart_title, ylabel):
    times = [float(row["t_sec"]) for row in rows if row.get(column)]
    values = [float(row[column]) for row in rows if row.get(column)]

    if not values:
        print(f"Skipped {filename}: no data in {column}")
        return

    plt.figure(figsize=(10, 5))
    plt.plot(times, values)
    plt.xlabel("Time (s)")
    plt.ylabel(ylabel)
    plt.title(chart_title)
    plt.grid(True)
    save_plot(filename)


def multi_line_chart(rows, series, filename, chart_title, ylabel, time_window=None):
    plotted = False
    plt.figure(figsize=(10, 5))

    start_sec, end_sec = time_window if time_window else (None, None)

    for column, label in series:
        points = [
            (float(row["t_sec"]), float(row[column]))
            for row in rows
            if row.get(column)
            and (start_sec is None or float(row["t_sec"]) >= start_sec)
            and (end_sec is None or float(row["t_sec"]) <= end_sec)
        ]
        if not points:
            continue

        times, values = zip(*points)
        if not values:
            continue

        plt.plot(times, values, label=label)
        plotted = True

    if not plotted:
        plt.close()
        print(f"Skipped {filename}: no data")
        return

    plt.xlabel("Time (s)")
    plt.ylabel(ylabel)
    plt.title(chart_title)
    if time_window:
        plt.xlim(start_sec, end_sec)
    plt.legend()
    plt.grid(True)
    save_plot(filename)


def cumulative_line_chart(rows, series, filename, chart_title, ylabel, scale=1):
    plotted = False
    plt.figure(figsize=(10, 5))

    for column, label in series:
        cumulative = 0
        points = []

        for row in rows:
            if column not in row or not row.get("t_sec"):
                continue

            cumulative += float(row.get(column) or 0)
            points.append((float(row["t_sec"]), cumulative / scale))

        if not points:
            continue

        times, values = zip(*points)
        plt.plot(times, values, label=label)
        plotted = True

    if not plotted:
        plt.close()
        print(f"Skipped {filename}: no data")
        return

    plt.xlabel("Time (s)")
    plt.ylabel(ylabel)
    plt.title(chart_title)
    plt.legend()
    plt.grid(True)
    save_plot(filename)


def latency_vs_throughput(rows):
    points = []

    for row in rows:
        if not row.get("client_avg_latency_ms"):
            continue

        throughput_mbps = sum(
            float(row.get(column) or 0)
            for column in ("echo_node_1_mbps", "echo_node_2_mbps")
        )
        points.append((throughput_mbps, float(row["client_avg_latency_ms"])))

    if not points:
        print("Skipped latency-vs-throughput.png: no data")
        return

    throughputs, latencies = zip(*points)

    plt.figure(figsize=(10, 5))
    plt.scatter(throughputs, latencies, s=14, alpha=0.7)
    plt.xlabel("Total Node Throughput (Mbps)")
    plt.ylabel("Client Average Latency (ms)")
    plt.title("Latency vs Throughput")
    plt.grid(True)
    save_plot("latency-vs-throughput.png")


def add_optimised_percent(rows):
    for row in rows:
        if row.get("unoptimised_percent"):
            row["optimised_percent"] = f"{100 - float(row['unoptimised_percent']):.2f}"


def add_process_rss_mib(rows):
    for row in rows:
        if row.get("process_rss_bytes"):
            row["process_rss_mib"] = f"{float(row['process_rss_bytes']) / 1024 / 1024:.2f}"


def classification_totals(rows):
    class_columns = [column for column in rows[0] if column.startswith("class_")]
    counts = [sum(float(row[column] or 0) for row in rows) for column in class_columns]

    plt.figure(figsize=(10, 5))
    plt.bar([title(column) for column in class_columns], counts)
    plt.xlabel("Classification Type")
    plt.ylabel("Total Classifications")
    plt.title("Classification Totals")
    plt.grid(True, axis="y")
    save_plot("classification-totals.png")


def main():
    rows = load_csv(INPUT_CSV)
    add_optimised_percent(rows)
    add_process_rss_mib(rows)

    line_chart(
        rows, "flow_count", "flow-count.png", "Flow Count Over Time", "Active Flows"
    )
    line_chart(
        rows,
        "optimised_percent",
        "optimised-packet-percent.png",
        "Percentage of Optimised Packets Over Time",
        "Optimised packets (%)",
    )
    line_chart(
        rows,
        "client_avg_latency_ms",
        "client-latency.png",
        "UDP Echo Client Average Latency Over Time",
        "Latency (ms)",
    )
    line_chart(
        rows,
        "process_cpu_percent",
        "process-cpu-percent.png",
        "Router CPU Usage Over Time",
        "CPU (%)",
    )
    line_chart(
        rows,
        "process_rss_mib",
        "process-memory-rss.png",
        "Router RSS Memory Usage Over Time",
        "RSS (MiB)",
    )
    multi_line_chart(
        rows,
        [
            ("echo_node_1_mbps", "Node 1"),
            ("echo_node_2_mbps", "Node 2"),
        ],
        "node-packet-rates.png",
        "Node Throughput Over Time (Minute 5 to 6)",
        "Throughput (Mbps)",
        time_window=NODE_PACKET_RATES_TIME_WINDOW,
    )
    cumulative_line_chart(
        rows,
        [
            ("echo_node_1_bytes", "Node 1"),
            ("echo_node_2_bytes", "Node 2"),
        ],
        "node-cumulative-bytes.png",
        "Cumulative Bytes by Node",
        "Data Transferred (MiB)",
        scale=1024 * 1024,
    )
    latency_vs_throughput(rows)
    classification_totals(rows)


if __name__ == "__main__":
    main()
