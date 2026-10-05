from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd

ROOT = Path(__file__).resolve().parents[1]

RAW_CSV = ROOT / "experiments" / "raw" / "microbench" / "fft.csv"
TABLE_DIR = ROOT / "results" / "tables"
FIG_DIR = ROOT / "results" / "figures"

TABLE_DIR.mkdir(parents=True, exist_ok=True)
FIG_DIR.mkdir(parents=True, exist_ok=True)

FORMAL_RUNS = 5


def load_formal_data() -> pd.DataFrame:
    if not RAW_CSV.exists():
        raise FileNotFoundError(f"Raw data not found: {RAW_CSV}")

    df = pd.read_csv(RAW_CSV)

    required = {
        "N",
        "domain_size",
        "threads",
        "run",
        "forward_ms",
        "inverse_ms",
    }

    missing = required - set(df.columns)

    if missing:
        raise ValueError(f"Missing columns: {sorted(missing)}")

    df["threads"] = df["threads"].astype(str)
    df["N"] = pd.to_numeric(df["N"], errors="raise")
    df["run"] = pd.to_numeric(df["run"], errors="raise")

    # Keep only the formal 1-thread / 20-thread configurations.
    df = df[df["threads"].isin(["1", "20"])].copy()

    if df.empty:
        raise ValueError("No formal 1T/20T FFT data found.")

    # Preserve original file order.
    #
    # fft.csv currently has no session_id, so the order of rows
    # is used to detect the beginning of a new formal session.
    df["_row_order"] = range(len(df))

    selected_groups = []

    for (n, threads), group in df.groupby(
        ["N", "threads"],
        sort=False,
    ):
        group = group.sort_values("_row_order").reset_index(drop=True)

        sessions = []
        current = []

        for _, row in group.iterrows():
            run = int(row["run"])

            # A new session begins when run numbering restarts.
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

        selected_groups.append(pd.DataFrame(complete_session))

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
        rows.append(
            {
                "N": int(n),
                "domain_size": int(group["domain_size"].iloc[-1]),
                "threads": int(threads),
                "runs": len(group),
                "forward_ms": group["forward_ms"].median(),
                "inverse_ms": group["inverse_ms"].median(),
            }
        )

    return pd.DataFrame(rows).sort_values(["N", "threads"]).reset_index(drop=True)


def build_speedup(
    summary: pd.DataFrame,
) -> pd.DataFrame:
    rows = []

    for n in sorted(summary["N"].unique()):
        subset = summary[summary["N"] == n].set_index("threads")

        if 1 not in subset.index or 20 not in subset.index:
            continue

        rows.append(
            {
                "N": int(n),
                "forward_speedup": (
                    subset.loc[1, "forward_ms"] / subset.loc[20, "forward_ms"]
                ),
                "inverse_speedup": (
                    subset.loc[1, "inverse_ms"] / subset.loc[20, "inverse_ms"]
                ),
            }
        )

    return pd.DataFrame(rows).sort_values("N").reset_index(drop=True)


def save_tables(
    summary: pd.DataFrame,
    speedup: pd.DataFrame,
) -> None:
    summary_path = TABLE_DIR / "fft_summary.csv"

    speedup_path = TABLE_DIR / "fft_speedup.csv"

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


def plot_fft_scaling(
    summary: pd.DataFrame,
) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    for threads in [1, 20]:
        data = summary[summary["threads"] == threads].sort_values("N")

        label = "1 thread" if threads == 1 else "20 threads"

        ax.plot(
            data["N"],
            data["forward_ms"],
            marker="o",
            label=f"Forward FFT - {label}",
        )

        ax.plot(
            data["N"],
            data["inverse_ms"],
            marker="s",
            linestyle="--",
            label=f"Inverse FFT - {label}",
        )

    ax.set_xscale("log", base=2)
    ax.set_yscale("log")

    ax.set_xlabel("FFT size (N)")
    ax.set_ylabel("Time (ms)")
    ax.set_title("FFT / IFFT Time Scaling")

    ax.grid(
        True,
        which="both",
        alpha=0.25,
    )

    ax.legend()

    fig.tight_layout()

    path = FIG_DIR / "fft_time_scaling.png"

    fig.savefig(
        path,
        dpi=200,
    )

    plt.close(fig)

    print(f"Saved: {path}")


def plot_fft_speedup(
    speedup: pd.DataFrame,
) -> None:
    fig, ax = plt.subplots(figsize=(8, 5))

    ax.plot(
        speedup["N"],
        speedup["forward_speedup"],
        marker="o",
        label="Forward FFT",
    )

    ax.plot(
        speedup["N"],
        speedup["inverse_speedup"],
        marker="s",
        linestyle="--",
        label="Inverse FFT",
    )

    ax.set_xscale("log", base=2)

    ax.set_xlabel("FFT size (N)")
    ax.set_ylabel("Speedup (1 thread / 20 threads)")
    ax.set_title("FFT Parallel Speedup")

    ax.grid(
        True,
        which="both",
        alpha=0.25,
    )

    ax.legend()

    fig.tight_layout()

    path = FIG_DIR / "fft_parallel_speedup.png"

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
    speedup = build_speedup(summary)

    save_tables(
        summary,
        speedup,
    )

    plot_fft_scaling(summary)
    plot_fft_speedup(speedup)

    print()
    print("=== FFT Summary ===")
    print(summary.to_string(index=False))

    print()
    print("=== FFT Speedup Summary ===")
    print(speedup.to_string(index=False))


if __name__ == "__main__":
    main()
