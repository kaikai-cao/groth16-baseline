from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd


ROOT = Path(__file__).resolve().parents[1]

RAW_CSV = ROOT / "experiments" / "raw" / "msm_trace" / "msm_breakdown.csv"
TABLE_DIR = ROOT / "results" / "tables"
FIG_DIR = ROOT / "results" / "figures"

TABLE_DIR.mkdir(parents=True, exist_ok=True)
FIG_DIR.mkdir(parents=True, exist_ok=True)


MSM_COLUMNS = [
    "msm_c_h_ms",
    "msm_c_l_ms",
    "msm_a_ms",
    "msm_b_g1_ms",
    "msm_b_g2_ms",
]

FORMAL_RUNS = 5


def load_formal_data() -> pd.DataFrame:
    if not RAW_CSV.exists():
        raise FileNotFoundError(f"Raw data not found: {RAW_CSV}")

    df = pd.read_csv(RAW_CSV)

    required_columns = {
        "N",
        "threads",
        "run",
        "setup_ms",
        "witness_ms",
        "prepare_vk_ms",
        "prove_ms",
        "verify_ms",
        "proof_bytes",
        *MSM_COLUMNS,
    }

    missing = required_columns - set(df.columns)
    if missing:
        raise ValueError(f"Missing columns: {sorted(missing)}")

    df["threads"] = df["threads"].astype(str)
    df["N"] = pd.to_numeric(df["N"], errors="raise")
    df["run"] = pd.to_numeric(df["run"], errors="raise")

    # "default" thread records are exploratory and are excluded.
    df = df[df["threads"].isin(["1", "20"])].copy()

    if df.empty:
        raise ValueError("No formal 1T/20T data found.")

    # Preserve original file order.
    #
    # msm_breakdown.csv does not currently contain a session_id.
    # Therefore the file order is the only reliable way to identify
    # the newest complete measurement block.
    df["_row_order"] = range(len(df))

    selected_groups = []

    for (n, threads), group in df.groupby(
        ["N", "threads"],
        sort=False,
    ):
        group = group.sort_values("_row_order").reset_index(drop=True)

        # Split measurements into sessions using the run number.
        #
        # A normal formal session is:
        #   1, 2, 3, 4, 5
        #
        # When another experiment is appended, the run number returns
        # from 5 to 1, which identifies a new session.
        sessions = []
        current = []

        for _, row in group.iterrows():
            run = int(row["run"])

            if current and run <= int(current[-1]["run"]):
                sessions.append(current)
                current = []

            current.append(row)

        if current:
            sessions.append(current)

        # Select the newest complete 1..5 session.
        complete_session = None

        for session in reversed(sessions):
            run_sequence = [int(row["run"]) for row in session]

            if run_sequence == list(range(1, FORMAL_RUNS + 1)):
                complete_session = session
                break

        if complete_session is None:
            raise ValueError(
                f"N={int(n)}, threads={threads}: "
                f"no complete formal session of runs "
                f"1..{FORMAL_RUNS} found."
            )

        session_df = pd.DataFrame(complete_session)
        selected_groups.append(session_df)

    selected = pd.concat(
        selected_groups,
        ignore_index=True,
    )

    return selected.drop(columns=["_row_order"])


def build_summary(df: pd.DataFrame) -> pd.DataFrame:
    rows = []

    for (n, threads), group in df.groupby(
        ["N", "threads"],
        sort=True,
    ):
        row = {
            "N": int(n),
            "threads": int(threads),
            "runs": len(group),
            "setup_ms": group["setup_ms"].median(),
            "witness_ms": group["witness_ms"].median(),
            "prepare_vk_ms": group["prepare_vk_ms"].median(),
            "prove_ms": group["prove_ms"].median(),
            "verify_ms": group["verify_ms"].median(),
            "proof_bytes": int(group["proof_bytes"].median()),
        }

        for col in MSM_COLUMNS:
            row[col] = group[col].median()

        row["msm_total_ms"] = (
            group[MSM_COLUMNS]
            .sum(axis=1)
            .median()
        )

        row["msm_ratio"] = (
            row["msm_total_ms"] /
            row["prove_ms"]
        )

        row["b_g2_ratio_of_msm"] = (
            row["msm_b_g2_ms"] /
            row["msm_total_ms"]
        )

        rows.append(row)

    return (
        pd.DataFrame(rows)
        .sort_values(["N", "threads"])
        .reset_index(drop=True)
    )


def build_speedup_table(summary: pd.DataFrame) -> pd.DataFrame:
    rows = []

    for n in sorted(summary["N"].unique()):
        subset = summary[
            summary["N"] == n
        ].set_index("threads")

        if 1 not in subset.index or 20 not in subset.index:
            continue

        row = {
            "N": int(n),
            "prove_speedup": (
                subset.loc[1, "prove_ms"]
                / subset.loc[20, "prove_ms"]
            ),
            "msm_total_speedup": (
                subset.loc[1, "msm_total_ms"]
                / subset.loc[20, "msm_total_ms"]
            ),
            "msm_c_h_speedup": (
                subset.loc[1, "msm_c_h_ms"]
                / subset.loc[20, "msm_c_h_ms"]
            ),
            "msm_c_l_speedup": (
                subset.loc[1, "msm_c_l_ms"]
                / subset.loc[20, "msm_c_l_ms"]
            ),
            "msm_a_speedup": (
                subset.loc[1, "msm_a_ms"]
                / subset.loc[20, "msm_a_ms"]
            ),
            "msm_b_g1_speedup": (
                subset.loc[1, "msm_b_g1_ms"]
                / subset.loc[20, "msm_b_g1_ms"]
            ),
            "msm_b_g2_speedup": (
                subset.loc[1, "msm_b_g2_ms"]
                / subset.loc[20, "msm_b_g2_ms"]
            ),
        }

        rows.append(row)

    return (
        pd.DataFrame(rows)
        .sort_values("N")
        .reset_index(drop=True)
    )


def save_tables(
    summary: pd.DataFrame,
    speedup: pd.DataFrame,
) -> None:
    summary_path = (
        TABLE_DIR / "msm_breakdown_summary.csv"
    )

    speedup_path = (
        TABLE_DIR / "msm_speedup.csv"
    )

    summary.to_csv(
        summary_path,
        index=False,
        float_format="%.3f",
    )

    speedup.to_csv(
        speedup_path,
        index=False,
        float_format="%.3f",
    )

    print(f"Saved: {summary_path}")
    print(f"Saved: {speedup_path}")


def plot_msm_scaling(summary: pd.DataFrame) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    for threads in [1, 20]:
        data = (
            summary[
                summary["threads"] == threads
            ]
            .sort_values("N")
        )

        label = (
            "1 thread"
            if threads == 1
            else "20 threads"
        )

        ax.plot(
            data["N"],
            data["msm_total_ms"],
            marker="o",
            label=label,
        )

    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("Number of constraints (N)")
    ax.set_ylabel("MSM total time (ms)")
    ax.set_title("Groth16 Prover MSM Time Scaling")
    ax.grid(True, which="both", alpha=0.25)
    ax.legend()

    fig.tight_layout()

    path = (
        FIG_DIR / "msm_time_scaling.png"
    )

    fig.savefig(
        path,
        dpi=200,
    )

    plt.close(fig)

    print(f"Saved: {path}")


def plot_parallel_speedup(
    speedup: pd.DataFrame,
) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    ax.plot(
        speedup["N"],
        speedup["msm_total_speedup"],
        marker="o",
        label="MSM total",
    )

    ax.plot(
        speedup["N"],
        speedup["prove_speedup"],
        marker="o",
        label="Prover",
    )

    ax.set_xscale("log")
    ax.set_xlabel("Number of constraints (N)")
    ax.set_ylabel(
        "Speedup (1 thread / 20 threads)"
    )
    ax.set_title("Groth16 Parallel Speedup")
    ax.grid(True, which="both", alpha=0.25)
    ax.legend()

    fig.tight_layout()

    path = (
        FIG_DIR / "msm_parallel_speedup.png"
    )

    fig.savefig(
        path,
        dpi=200,
    )

    plt.close(fig)

    print(f"Saved: {path}")


def plot_b_g2_breakdown(
    summary: pd.DataFrame,
) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    for threads in [1, 20]:
        data = (
            summary[
                summary["threads"] == threads
            ]
            .sort_values("N")
        )

        label = (
            "1 thread"
            if threads == 1
            else "20 threads"
        )

        ax.plot(
            data["N"],
            data["b_g2_ratio_of_msm"] * 100,
            marker="o",
            label=label,
        )

    ax.set_xscale("log")
    ax.set_xlabel("Number of constraints (N)")
    ax.set_ylabel("B-G2 share of total MSM (%)")
    ax.set_title("B-G2 MSM Share")
    ax.grid(True, which="both", alpha=0.25)
    ax.legend()

    fig.tight_layout()

    path = (
        FIG_DIR / "b_g2_msm_share.png"
    )

    fig.savefig(
        path,
        dpi=200,
    )

    plt.close(fig)

    print(f"Saved: {path}")


def plot_prover_msm_ratio(
    summary: pd.DataFrame,
) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    for threads in [1, 20]:
        data = (
            summary[
                summary["threads"] == threads
            ]
            .sort_values("N")
        )

        label = (
            "1 thread"
            if threads == 1
            else "20 threads"
        )

        ax.plot(
            data["N"],
            data["msm_ratio"] * 100,
            marker="o",
            label=label,
        )

    ax.set_xscale("log")
    ax.set_xlabel("Number of constraints (N)")
    ax.set_ylabel("MSM / Prover time (%)")
    ax.set_title(
        "MSM Contribution to Groth16 Prover"
    )
    ax.grid(True, which="both", alpha=0.25)
    ax.legend()

    fig.tight_layout()

    path = (
        FIG_DIR / "msm_prover_ratio.png"
    )

    fig.savefig(
        path,
        dpi=200,
    )

    plt.close(fig)

    print(f"Saved: {path}")


def main() -> None:
    print(f"Reading: {RAW_CSV}")

    df = load_formal_data()

    summary = build_summary(df)
    speedup = build_speedup_table(summary)

    save_tables(
        summary,
        speedup,
    )

    plot_msm_scaling(summary)
    plot_parallel_speedup(speedup)
    plot_b_g2_breakdown(summary)
    plot_prover_msm_ratio(summary)

    print()
    print("=== MSM Summary ===")
    print(
        summary.to_string(index=False)
    )

    print()
    print("=== Speedup Summary ===")
    print(
        speedup.to_string(index=False)
    )


if __name__ == "__main__":
    main()