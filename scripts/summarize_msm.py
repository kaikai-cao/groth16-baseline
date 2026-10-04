from pathlib import Path
import csv
import statistics


RAW_DIR = Path("experiments/raw/microbench")
RESULT_DIR = Path("results/tables")
RESULT_FILE = RESULT_DIR / "msm.csv"

EXPECTED_RUNS = 5


def load_csv(path: Path):
    with path.open("r", newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def median(values):
    return statistics.median(values)


def process_group(rows, group_name):
    """
    Convert raw runs into one median result per
    (group, threads, N).

    Current raw-data convention:
    - 'default' thread records are exploratory and excluded.
    - For explicit thread settings, exactly 5 formal runs
      are expected for each (group, threads, N).
    """

    # Exclude exploratory runs whose thread value is "default".
    rows = [
        row for row in rows
        if row["threads"].strip().lower() != "default"
    ]

    groups = {}

    for row in rows:
        threads = row["threads"].strip()
        n = int(row["N"])

        key = (group_name, int(threads), n)

        groups.setdefault(key, []).append(
            float(row["time_ms"])
        )

    results = []

    for (group, threads, n), times in sorted(groups.items()):
        if len(times) < EXPECTED_RUNS:
            raise RuntimeError(
                f"{group}, threads={threads}, N={n}: "
                f"only {len(times)} formal runs found; "
                f"expected {EXPECTED_RUNS}"
            )

        # Use the five formal measurements.
        #
        # For the current experiment every configuration
        # has exactly five formal runs.
        if len(times) > EXPECTED_RUNS:
            print(
                f"Warning: {group}, threads={threads}, N={n} "
                f"has {len(times)} runs; using the last "
                f"{EXPECTED_RUNS} runs."
            )
            times = times[-EXPECTED_RUNS:]

        times_sorted = sorted(times)

        results.append({
            "group": group,
            "threads": threads,
            "N": n,
            "runs": len(times_sorted),
            "median_ms": median(times_sorted),
            "min_ms": min(times_sorted),
            "max_ms": max(times_sorted),
        })

    return results


def main():
    RESULT_DIR.mkdir(parents=True, exist_ok=True)

    g1_file = RAW_DIR / "msm_g1.csv"
    g2_file = RAW_DIR / "msm_g2.csv"

    if not g1_file.exists():
        raise FileNotFoundError(g1_file)

    if not g2_file.exists():
        raise FileNotFoundError(g2_file)

    g1_rows = load_csv(g1_file)
    g2_rows = load_csv(g2_file)

    results = []

    results.extend(
        process_group(g1_rows, "G1")
    )

    results.extend(
        process_group(g2_rows, "G2")
    )

    results.sort(
        key=lambda r: (
            r["group"],
            int(r["threads"]),
            r["N"],
        )
    )

    with RESULT_FILE.open(
        "w",
        newline="",
        encoding="utf-8"
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
            writer.writerow({
                "group": row["group"],
                "threads": row["threads"],
                "N": row["N"],
                "runs": row["runs"],
                "median_ms": f"{row['median_ms']:.3f}",
                "min_ms": f"{row['min_ms']:.3f}",
                "max_ms": f"{row['max_ms']:.3f}",
            })

    print(f"Summary written to: {RESULT_FILE}")
    print(f"Configurations: {len(results)}")


if __name__ == "__main__":
    main()