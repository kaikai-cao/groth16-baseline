from pathlib import Path
import csv

import matplotlib.pyplot as plt


INPUT_FILE = Path("results/tables/msm.csv")
OUTPUT_FILE = Path("results/figures/msm_microbench.png")


def load_data():
    rows = []

    with INPUT_FILE.open(
        "r",
        newline="",
        encoding="utf-8",
    ) as f:
        reader = csv.DictReader(f)

        for row in reader:
            rows.append({
                "group": row["group"],
                "threads": int(row["threads"]),
                "N": int(row["N"]),
                "median_ms": float(row["median_ms"]),
            })

    return rows


def main():
    rows = load_data()

    OUTPUT_FILE.parent.mkdir(
        parents=True,
        exist_ok=True,
    )

    # --------------------------------
    # Build four series
    # --------------------------------

    series = {}

    for row in rows:
        key = (
            row["group"],
            row["threads"],
        )

        series.setdefault(key, []).append(row)

    for values in series.values():
        values.sort(key=lambda x: x["N"])

    # --------------------------------
    # Plot
    # --------------------------------

    plt.figure(figsize=(9, 6))

    for (group, threads), values in sorted(series.items()):
        x = [v["N"] for v in values]
        y = [v["median_ms"] for v in values]

        plt.plot(
            x,
            y,
            marker="o",
            linewidth=2,
            label=f"{group}, {threads} thread"
        )

    # N spans 2^10 ... 2^16
    plt.xscale("log", base=2)

    # Time also spans a fairly large range
    plt.yscale("log")

    plt.xlabel("MSM Input Size N")
    plt.ylabel("Median Time (ms)")
    plt.title("BN254 MSM Scaling")

    plt.xticks(
        [1024, 4096, 16384, 65536],
        ["2^10", "2^12", "2^14", "2^16"],
    )

    plt.grid(
        True,
        which="both",
        alpha=0.25,
    )

    plt.legend()

    plt.tight_layout()

    plt.savefig(
        OUTPUT_FILE,
        dpi=300,
        bbox_inches="tight",
    )

    plt.close()

    print(
        f"Figure written to: {OUTPUT_FILE}"
    )


if __name__ == "__main__":
    main()