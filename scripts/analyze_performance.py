from pathlib import Path
import csv
import statistics

import matplotlib.pyplot as plt

ROOT = Path(__file__).resolve().parents[1]

# Existing repository data.
THREAD_RAW = ROOT / "experiments" / "raw" / "thread_scaling" / "prove_scaling.csv"

TABLE_DIR = ROOT / "results" / "tables"
FIG_DIR = ROOT / "results" / "figures"

TABLE_DIR.mkdir(parents=True, exist_ok=True)
FIG_DIR.mkdir(parents=True, exist_ok=True)


def read_csv(path: Path):
    with path.open("r", encoding="utf-8-sig", newline="") as f:
        return list(csv.DictReader(f))


def median(values):
    return statistics.median(values)


def experiment_key(row):
    """
    The existing prove_scaling.csv contains multiple experiments.

    The setup/witness/prepare-vk values form a useful fingerprint for one
    experiment block, while run is the repetition number inside that block.

    This lets us separate:
      - the clean thread-scaling blocks
      - later WNAF/window experiments

    without assuming that every row in the file belongs to one experiment.
    """
    return (
        int(row["N"]),
        int(row["threads"]),
        row["setup_ms"],
        row["witness_ms"],
        row["prepare_vk_ms"],
    )


def build_blocks(rows):
    """
    Group rows into experimental blocks.

    A block is identified by:
        N + threads + setup_ms + witness_ms + prepare_vk_ms

    and ordered by first appearance in the raw file.
    """
    blocks = []
    current = None
    current_key = None

    for row in rows:
        key = experiment_key(row)

        if key != current_key:
            current = {
                "key": key,
                "first_index": len(blocks),
                "rows": [],
            }
            blocks.append(current)
            current_key = key

        current["rows"].append(row)

    return blocks


def choose_thread_scaling_blocks(rows, target_n=100000):
    """
    Recover the afternoon clean thread-scaling dataset.

    Selection rule:
      1. Only N=100000.
      2. For each thread count, choose the block with the largest number
         of recorded runs.
      3. If several blocks have the same run count, choose the latest block.

    This intentionally prefers the 10-run clean reruns for 8/12/16 threads,
    while preserving the available 5-run formal blocks for other thread counts.
    """
    candidates = [
        block
        for block in build_blocks(rows)
        if block["key"][0] == target_n and block["key"][1] in {1, 2, 4, 8, 12, 16, 20}
    ]

    selected = {}

    for block in candidates:
        threads = block["key"][1]
        run_count = len(block["rows"])

        current = selected.get(threads)

        if current is None:
            selected[threads] = block
            continue

        current_count = len(current["rows"])

        if run_count > current_count:
            selected[threads] = block
        elif run_count == current_count:
            # Keep the later block.
            selected[threads] = block

    if set(selected) != {1, 2, 4, 8, 12, 16, 20}:
        missing = sorted({1, 2, 4, 8, 12, 16, 20} - set(selected))
        raise RuntimeError(
            f"Could not recover all thread configurations; missing: {missing}"
        )

    return selected


def summarize_thread_scaling(selected):
    rows = []

    baseline_thread = min(selected)
    baseline_block = selected[baseline_thread]

    baseline_prove = median([float(r["prove_ms"]) for r in baseline_block["rows"]])
    baseline_msm = median([float(r["msm_total_ms"]) for r in baseline_block["rows"]])

    for threads in sorted(selected):
        block = selected[threads]
        data = block["rows"]

        prove_ms = median([float(r["prove_ms"]) for r in data])
        msm_ms = median([float(r["msm_total_ms"]) for r in data])

        rows.append(
            {
                "N": 100000,
                "threads": threads,
                "runs_used": len(data),
                "prove_ms": prove_ms,
                "msm_ms": msm_ms,
                "prove_speedup": baseline_prove / prove_ms,
                "msm_speedup": baseline_msm / msm_ms,
                "msm_share_pct": msm_ms / prove_ms * 100.0,
                "setup_ms": float(data[0]["setup_ms"]),
            }
        )

    return rows


def save_thread_table(rows):
    path = TABLE_DIR / "thread_scaling.csv"

    with path.open(
        "w",
        encoding="utf-8-sig",
        newline="",
    ) as f:
        writer = csv.DictWriter(
            f,
            fieldnames=[
                "N",
                "threads",
                "runs_used",
                "prove_ms",
                "msm_ms",
                "prove_speedup",
                "msm_speedup",
                "msm_share_pct",
                "setup_ms",
            ],
        )
        writer.writeheader()

        for row in rows:
            writer.writerow(
                {
                    "N": row["N"],
                    "threads": row["threads"],
                    "runs_used": row["runs_used"],
                    "prove_ms": f"{row['prove_ms']:.3f}",
                    "msm_ms": f"{row['msm_ms']:.3f}",
                    "prove_speedup": f"{row['prove_speedup']:.3f}",
                    "msm_speedup": f"{row['msm_speedup']:.3f}",
                    "msm_share_pct": f"{row['msm_share_pct']:.3f}",
                    "setup_ms": f"{row['setup_ms']:.3f}",
                }
            )

    return path


def save_thread_run_selection(selected):
    """
    Preserve exactly which raw rows were selected for the clean comparison.
    """
    path = TABLE_DIR / "thread_scaling_selected_runs.csv"

    with path.open(
        "w",
        encoding="utf-8-sig",
        newline="",
    ) as f:
        fieldnames = [
            "N",
            "threads",
            "run",
            "prove_ms",
            "msm_c_h_ms",
            "msm_c_l_ms",
            "msm_a_ms",
            "msm_b_g1_ms",
            "msm_b_g2_ms",
            "msm_total_ms",
            "setup_ms",
            "witness_ms",
            "prepare_vk_ms",
        ]

        writer = csv.DictWriter(
            f,
            fieldnames=fieldnames,
        )
        writer.writeheader()

        for threads in sorted(selected):
            for row in selected[threads]["rows"]:
                writer.writerow({field: row.get(field, "") for field in fieldnames})

    return path


def plot_thread_scaling(rows):
    threads = [r["threads"] for r in rows]
    prove = [r["prove_ms"] for r in rows]
    msm = [r["msm_ms"] for r in rows]

    plt.figure(figsize=(8, 5))
    plt.plot(threads, prove, marker="o", label="Prove")
    plt.plot(threads, msm, marker="o", label="MSM")
    plt.xlabel("Rayon threads")
    plt.ylabel("Median time (ms)")
    plt.title("Groth16 Thread Scaling (N=100k)")
    plt.grid(True, alpha=0.25)
    plt.legend()
    plt.tight_layout()

    path = FIG_DIR / "thread_scaling_time.png"
    plt.savefig(path, dpi=300, bbox_inches="tight")
    plt.close()

    plt.figure(figsize=(8, 5))
    plt.plot(
        threads,
        [r["prove_speedup"] for r in rows],
        marker="o",
        label="Prove",
    )
    plt.plot(
        threads,
        [r["msm_speedup"] for r in rows],
        marker="o",
        label="MSM",
    )
    plt.xlabel("Rayon threads")
    plt.ylabel("Speedup (1 thread / T threads)")
    plt.title("Groth16 Parallel Speedup (N=100k)")
    plt.grid(True, alpha=0.25)
    plt.legend()
    plt.tight_layout()

    speedup_path = FIG_DIR / "thread_scaling_speedup.png"
    plt.savefig(speedup_path, dpi=300, bbox_inches="tight")
    plt.close()

    return path, speedup_path


def main():
    if not THREAD_RAW.exists():
        raise FileNotFoundError(
            f"Existing repository raw file not found:\n{THREAD_RAW}"
        )

    rows = read_csv(THREAD_RAW)

    selected = choose_thread_scaling_blocks(rows)
    summary = summarize_thread_scaling(selected)

    table_path = save_thread_table(summary)
    selected_path = save_thread_run_selection(selected)
    figure_path, speedup_path = plot_thread_scaling(summary)

    print()
    print("=== Thread Scaling Dataset Selected ===")

    for row in summary:
        print(
            f"N={row['N']}, "
            f"threads={row['threads']}, "
            f"runs={row['runs_used']}, "
            f"prove={row['prove_ms']:.3f} ms, "
            f"msm={row['msm_ms']:.3f} ms"
        )

    print()
    print("=== Outputs ===")
    print(f"Table: {table_path}")
    print(f"Selected runs: {selected_path}")
    print(f"Figure: {figure_path}")
    print(f"Figure: {speedup_path}")
    print()
    print(
        "Note: window and microbenchmark analyses are intentionally not "
        "run here because their raw datasets are not yet present in the "
        "repository under their final experiment paths."
    )


if __name__ == "__main__":
    main()
