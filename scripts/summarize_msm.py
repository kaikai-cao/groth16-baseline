from pathlib import Path
import csv
import statistics

ROOT = Path(__file__).resolve().parents[1]

RAW_DIR = ROOT / "experiments" / "raw" / "microbench"
RESULT_DIR = ROOT / "results" / "tables"
RESULT_FILE = RESULT_DIR / "msm.csv"

EXPECTED_RUNS = 5


def load_csv(path: Path):
    if not path.exists():
        raise FileNotFoundError(path)

    with path.open(
        "r",
        newline="",
        encoding="utf-8",
    ) as f:
        rows = list(csv.DictReader(f))

    if not rows:
        raise RuntimeError(f"No data found in {path}")

    return rows


def median(values):
    return statistics.median(values)


def process_group(rows, group_name):
    """
    Convert raw MSM measurements into one summary row per
    (group, threads, N).

    Raw MSM microbenchmark format:

        timestamp_ms,threads,group,N,time_ms

    Each row is one measurement, so there is no run/session field.

    Current processing rule:
    - exclude exploratory 'default' thread measurements;
    - group by explicit thread count and N;
    - sort by timestamp;
    - use the latest five measurements;
    - report median/min/max.
    """

    formal_rows = [row for row in rows if row["threads"].strip().lower() != "default"]

    configurations = {}

    for row in formal_rows:
        threads = int(row["threads"].strip())

        n = int(row["N"])

        key = (
            threads,
            n,
        )

        configurations.setdefault(
            key,
            [],
        ).append(row)

    results = []

    for (threads, n), config_rows in sorted(configurations.items()):
        config_rows = sorted(
            config_rows,
            key=lambda row: int(row["timestamp_ms"]),
        )

        if len(config_rows) < EXPECTED_RUNS:
            raise RuntimeError(
                f"{group_name}, threads={threads}, N={n}: "
                f"only {len(config_rows)} measurements found; "
                f"expected at least {EXPECTED_RUNS}"
            )

        # Use the newest five measurements.
        latest_rows = config_rows[-EXPECTED_RUNS:]

        times = [float(row["time_ms"]) for row in latest_rows]

        times_sorted = sorted(times)

        results.append(
            {
                "group": group_name,
                "threads": str(threads),
                "N": n,
                "runs": len(times),
                "median_ms": median(times),
                "min_ms": min(times),
                "max_ms": max(times),
            }
        )

    return results


def main():
    RESULT_DIR.mkdir(
        parents=True,
        exist_ok=True,
    )

    g1_file = RAW_DIR / "msm_g1.csv"

    g2_file = RAW_DIR / "msm_g2.csv"

    g1_rows = load_csv(g1_file)
    g2_rows = load_csv(g2_file)

    results = []

    results.extend(
        process_group(
            g1_rows,
            "G1",
        )
    )

    results.extend(
        process_group(
            g2_rows,
            "G2",
        )
    )

    results.sort(
        key=lambda row: (
            row["group"],
            int(row["threads"]),
            row["N"],
        )
    )

    with RESULT_FILE.open(
        "w",
        newline="",
        encoding="utf-8",
    ) as f:

        writer = csv.DictWriter(
            f,
            fieldnames=[
                "group",
                "threads",
                "N",
                "runs",
                "median_ms",
                "min_ms",
                "max_ms",
            ],
        )

        writer.writeheader()

        for row in results:
            writer.writerow(
                {
                    "group": row["group"],
                    "threads": row["threads"],
                    "N": row["N"],
                    "runs": row["runs"],
                    "median_ms": (f"{row['median_ms']:.3f}"),
                    "min_ms": (f"{row['min_ms']:.3f}"),
                    "max_ms": (f"{row['max_ms']:.3f}"),
                }
            )

    print(f"Summary written to: {RESULT_FILE}")

    print(f"Configurations: {len(results)}")


if __name__ == "__main__":
    main()
