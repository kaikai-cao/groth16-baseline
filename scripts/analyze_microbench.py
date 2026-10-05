from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd

ROOT = Path(__file__).resolve().parents[1]
RAW_DIR = ROOT / "experiments" / "raw" / "microbench"
TABLE_DIR = ROOT / "results" / "tables"
FIG_DIR = ROOT / "results" / "figures"

TABLE_DIR.mkdir(parents=True, exist_ok=True)
FIG_DIR.mkdir(parents=True, exist_ok=True)


GROUP_ADD_RAW = RAW_DIR / "group_add.csv"
PREFIX_RAW = RAW_DIR / "prefix.csv"


def load_csv(path: Path, required_columns: set[str]) -> pd.DataFrame:
    if not path.exists():
        raise FileNotFoundError(f"Raw data not found: {path}")

    df = pd.read_csv(path)

    missing = required_columns - set(df.columns)
    if missing:
        raise ValueError(f"{path.name}: missing columns: {sorted(missing)}")

    return df


def keep_latest_five(df: pd.DataFrame, group_columns: list[str]) -> pd.DataFrame:
    """Keep the latest five formal runs for each configuration."""
    df = df.copy()
    df["session_id"] = pd.to_numeric(df["session_id"], errors="raise")
    df["run"] = pd.to_numeric(df["run"], errors="raise")

    return (
        df.sort_values([*group_columns, "session_id", "run"])
        .groupby(group_columns, group_keys=False)
        .tail(5)
        .copy()
    )


def build_group_add_summary() -> pd.DataFrame:
    required = {
        "session_id",
        "bucket_count",
        "updates",
        "group",
        "operation",
        "run",
        "total_ms",
        "ns_per_update",
    }

    df = load_csv(GROUP_ADD_RAW, required)
    df = keep_latest_five(df, ["bucket_count", "updates", "group", "operation"])

    rows = []
    for (bucket_count, updates, group, operation), data in df.groupby(
        ["bucket_count", "updates", "group", "operation"], sort=True
    ):
        rows.append(
            {
                "bucket_count": int(bucket_count),
                "updates": int(updates),
                "group": group,
                "operation": operation,
                "runs": len(data),
                "median_ms": data["total_ms"].median(),
                "median_ns_per_update": data["ns_per_update"].median(),
            }
        )

    summary = pd.DataFrame(rows)

    ratios = summary.pivot_table(
        index=["bucket_count", "updates", "operation"],
        columns="group",
        values="median_ns_per_update",
        aggfunc="first",
    ).reset_index()

    if "G1" in ratios.columns and "G2" in ratios.columns:
        ratios["g2_over_g1"] = ratios["G2"] / ratios["G1"]
    else:
        ratios["g2_over_g1"] = float("nan")

    summary = summary.merge(
        ratios[["bucket_count", "updates", "operation", "g2_over_g1"]],
        on=["bucket_count", "updates", "operation"],
        how="left",
    )

    return summary.sort_values(["bucket_count", "operation", "group"]).reset_index(
        drop=True
    )


def build_prefix_summary() -> pd.DataFrame:
    required = {
        "session_id",
        "bucket_count",
        "repeats",
        "total_additions",
        "group",
        "run",
        "total_ms",
        "ns_per_addition",
    }

    df = load_csv(PREFIX_RAW, required)
    df = keep_latest_five(
        df,
        ["bucket_count", "repeats", "total_additions", "group"],
    )

    rows = []
    for (bucket_count, repeats, total_additions, group), data in df.groupby(
        ["bucket_count", "repeats", "total_additions", "group"], sort=True
    ):
        rows.append(
            {
                "bucket_count": int(bucket_count),
                "repeats": int(repeats),
                "total_additions": int(total_additions),
                "group": group,
                "runs": len(data),
                "median_ms": data["total_ms"].median(),
                "median_ns_per_addition": data["ns_per_addition"].median(),
            }
        )

    summary = pd.DataFrame(rows)

    ratios = summary.pivot_table(
        index=["bucket_count", "repeats", "total_additions"],
        columns="group",
        values="median_ns_per_addition",
        aggfunc="first",
    ).reset_index()

    if "G1" in ratios.columns and "G2" in ratios.columns:
        ratios["g2_over_g1"] = ratios["G2"] / ratios["G1"]
    else:
        ratios["g2_over_g1"] = float("nan")

    summary = summary.merge(
        ratios[
            [
                "bucket_count",
                "repeats",
                "total_additions",
                "g2_over_g1",
            ]
        ],
        on=["bucket_count", "repeats", "total_additions"],
        how="left",
    )

    return summary.sort_values(["bucket_count", "group"]).reset_index(drop=True)


def build_ratio_table(group_add: pd.DataFrame, prefix: pd.DataFrame) -> pd.DataFrame:
    mixed = group_add[group_add["operation"] == "mixed"].copy()
    mixed_ratio = (
        mixed[["bucket_count", "g2_over_g1"]]
        .drop_duplicates("bucket_count")
        .rename(columns={"g2_over_g1": "group_add_mixed_g2_over_g1"})
    )

    prefix_ratio = (
        prefix[["bucket_count", "g2_over_g1"]]
        .drop_duplicates("bucket_count")
        .rename(columns={"g2_over_g1": "prefix_g2_over_g1"})
    )

    return (
        mixed_ratio.merge(prefix_ratio, on="bucket_count", how="outer")
        .sort_values("bucket_count")
        .reset_index(drop=True)
    )


def save_tables(
    group_add: pd.DataFrame,
    prefix: pd.DataFrame,
    ratios: pd.DataFrame,
) -> None:
    group_add_path = TABLE_DIR / "group_add_microbench.csv"
    prefix_path = TABLE_DIR / "prefix_microbench.csv"
    ratios_path = TABLE_DIR / "microbench_ratio_by_bucket.csv"

    group_add.to_csv(group_add_path, index=False, float_format="%.3f")
    prefix.to_csv(prefix_path, index=False, float_format="%.3f")
    ratios.to_csv(ratios_path, index=False, float_format="%.3f")

    print(f"Saved: {group_add_path}")
    print(f"Saved: {prefix_path}")
    print(f"Saved: {ratios_path}")


def plot_group_add_by_bucket(summary: pd.DataFrame) -> None:
    plot_df = summary[summary["operation"] == "mixed"].copy()

    fig, ax = plt.subplots(figsize=(8, 5))

    for group in ["G1", "G2"]:
        data = plot_df[plot_df["group"] == group].sort_values("bucket_count")
        ax.plot(
            data["bucket_count"],
            data["median_ns_per_update"],
            marker="o",
            label=group,
        )

    ax.set_xscale("log", base=2)
    ax.set_xlabel("Bucket count")
    ax.set_ylabel("Median time per mixed update (ns)")
    ax.set_title("G1/G2 Bucket Update Cost vs Bucket Count")
    ax.grid(True, which="both", alpha=0.25)
    ax.legend()
    fig.tight_layout()

    path = FIG_DIR / "group_add_microbench.png"
    fig.savefig(path, dpi=200)
    plt.close(fig)

    print(f"Saved: {path}")


def plot_group_add_ratio(summary: pd.DataFrame) -> None:
    plot_df = summary[summary["operation"] == "mixed"]
    plot_df = plot_df.drop_duplicates("bucket_count").sort_values("bucket_count")

    fig, ax = plt.subplots(figsize=(8, 5))
    ax.plot(
        plot_df["bucket_count"],
        plot_df["g2_over_g1"],
        marker="o",
    )
    ax.set_xscale("log", base=2)
    ax.set_xlabel("Bucket count")
    ax.set_ylabel("G2 / G1")
    ax.set_title("G2/G1 Ratio for Bucket Updates")
    ax.grid(True, which="both", alpha=0.25)
    fig.tight_layout()

    path = FIG_DIR / "group_add_g2_over_g1.png"
    fig.savefig(path, dpi=200)
    plt.close(fig)

    print(f"Saved: {path}")


def plot_prefix_by_bucket(summary: pd.DataFrame) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    for group in ["G1", "G2"]:
        data = summary[summary["group"] == group].sort_values("bucket_count")
        ax.plot(
            data["bucket_count"],
            data["median_ns_per_addition"],
            marker="o",
            label=group,
        )

    ax.set_xscale("log", base=2)
    ax.set_xlabel("Bucket count")
    ax.set_ylabel("Median time per group addition (ns)")
    ax.set_title("G1/G2 Prefix Reduction Cost vs Bucket Count")
    ax.grid(True, which="both", alpha=0.25)
    ax.legend()
    fig.tight_layout()

    path = FIG_DIR / "prefix_microbench.png"
    fig.savefig(path, dpi=200)
    plt.close(fig)

    print(f"Saved: {path}")


def plot_prefix_ratio(summary: pd.DataFrame) -> None:
    plot_df = summary.drop_duplicates("bucket_count").sort_values("bucket_count")

    fig, ax = plt.subplots(figsize=(8, 5))
    ax.plot(
        plot_df["bucket_count"],
        plot_df["g2_over_g1"],
        marker="o",
    )
    ax.set_xscale("log", base=2)
    ax.set_xlabel("Bucket count")
    ax.set_ylabel("G2 / G1")
    ax.set_title("G2/G1 Ratio for Prefix Reduction")
    ax.grid(True, which="both", alpha=0.25)
    fig.tight_layout()

    path = FIG_DIR / "prefix_g2_over_g1.png"
    fig.savefig(path, dpi=200)
    plt.close(fig)

    print(f"Saved: {path}")


def main() -> None:
    print(f"Reading group-add raw data: {GROUP_ADD_RAW}")
    print(f"Reading prefix raw data: {PREFIX_RAW}")

    group_add = build_group_add_summary()
    prefix = build_prefix_summary()
    ratios = build_ratio_table(group_add, prefix)

    save_tables(group_add, prefix, ratios)

    plot_group_add_by_bucket(group_add)
    plot_group_add_ratio(group_add)
    plot_prefix_by_bucket(prefix)
    plot_prefix_ratio(prefix)

    print()
    print("=== Group Add Summary ===")
    print(group_add.to_string(index=False))

    print()
    print("=== Prefix Summary ===")
    print(prefix.to_string(index=False))

    print()
    print("=== Microbenchmark Ratios by Bucket Count ===")
    print(ratios.to_string(index=False))


if __name__ == "__main__":
    main()
