from pathlib import Path
import csv


import matplotlib.pyplot as plt


INPUT_FILE = Path("results/tables/msm.csv")
OUTPUT_FILE = Path("results/figures/msm_speedup.png")


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

    data = {}

    for row in rows:
        key = (
            row["group"],
            row["N"],
        )

        data.setdefault(key, {})[
            row["threads"]
        ] = row["median_ms"]

    plt.figure(figsize=(9, 6))

    for group in ["G1", "G2"]:
        ns = []
        speedups = []

        for n in [1024, 4096, 16384, 65536]:
            values = data[(group, n)]

            t1 = values[1]
            t20 = values[20]

            speedup = t1 / t20

            ns.append(n)
            speedups.append(speedup)

        plt.plot(
            ns,
            speedups,
            marker="o",
            linewidth=2,
            label=group,
        )

    plt.xscale("log", base=2)

    plt.xlabel("MSM Input Size N")
    plt.ylabel("Speedup (1 thread / 20 threads)")
    plt.title("BN254 MSM Parallel Speedup")

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