from pathlib import Path
import csv
import statistics

ROOT = Path(__file__).resolve().parents[1]

RAW_DIR = ROOT / "experiments" / "raw" / "microbench"
RESULT_DIR = ROOT / "results" / "tables"
RESULT_FILE = RESULT_DIR / "msm.csv"

# One complete benchmark session contains 7 formal runs.
EXPECTED_SESSION_RUNS = 7

# The reported statistic uses the latest 5 formal runs
# from the selected complete session.
REPORTED_RUNS = 5


def load_csv(path: Path):
    if not path.exists():
        raise FileNotFoundError(f"Raw data not found: {path}")

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

    Expected raw format:

        session_id,threads,group,N,run,time_ms

    Processing rule:
    1. Exclude exploratory 'default' thread measurements.
    2. Group measurements by (threads, N).
    3. For each configuration, identify complete sessions
       containing exactly the formal run sequence 1..7.
    4. Select the newest complete session.
    5. Use the latest 5 formal runs from that session.
    6. Report median/min/max.
    """

    required_columns = {
        "session_id",
        "threads",
        "group",
        "N",
        "run",
        "time_ms",
    }

    if not rows:
        raise RuntimeError(f"{group_name}: no rows found")

    missing = required_columns - set(rows[0].keys())

    if missing:
        raise ValueError(f"{group_name}: missing columns: " f"{sorted(missing)}")

    # Exclude exploratory measurements using the environment-default
    # Rayon thread count.
    formal_rows = [row for row in rows if row["threads"].strip().lower() != "default"]

    if not formal_rows:
        raise RuntimeError(f"{group_name}: no formal thread measurements found")

    # Group by explicit thread count and MSM input size.
    configurations = {}

    for row in formal_rows:
        threads = int(row["threads"].strip())
        n = int(row["N"])
        session = int(row["session_id"])
        run = int(row["run"])
        time_ms = float(row["time_ms"])

        key = (threads, n)

        configurations.setdefault(
            key,
            [],
        ).append(
            {
                "session_id": session,
                "run": run,
                "time_ms": time_ms,
            }
        )

    results = []

    for (threads, n), config_rows in sorted(configurations.items()):
        # Group measurements by session_id.
        sessions = {}

        for row in config_rows:
            session = row["session_id"]

            sessions.setdefault(
                session,
                [],
            ).append(row)

        complete_sessions = []

        for session, session_rows in sessions.items():
            session_rows = sorted(
                session_rows,
                key=lambda row: row["run"],
            )

            run_sequence = [row["run"] for row in session_rows]

            expected_sequence = list(range(1, EXPECTED_SESSION_RUNS + 1))

            if run_sequence == expected_sequence:
                complete_sessions.append((session, session_rows))

        if not complete_sessions:
            raise RuntimeError(
                f"{group_name}, threads={threads}, N={n}: "
                f"no complete session with runs "
                f"1..{EXPECTED_SESSION_RUNS} found"
            )

        # Select the newest complete session.
        session, session_rows = max(
            complete_sessions,
            key=lambda item: item[0],
        )

        # Use the latest 5 formal measurements from that session.
        latest_rows = session_rows[-REPORTED_RUNS:]

        times = [row["time_ms"] for row in latest_rows]

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
            writer.writerow(row)

    print(f"Summary written to: {RESULT_FILE}")

    print(f"Configurations: {len(results)}")

    print()
    print("=== MSM Summary ===")

    for row in results:
        print(
            f"{row['group']:>2} "
            f"{row['threads']:>2}T "
            f"N={row['N']:>6} "
            f"runs={row['runs']} "
            f"median={row['median_ms']:.3f} ms "
            f"min={row['min_ms']:.3f} ms "
            f"max={row['max_ms']:.3f} ms"
        )


if __name__ == "__main__":
    main()
