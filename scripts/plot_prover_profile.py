from pathlib import Path
import csv
import re
import statistics

import matplotlib.pyplot as plt


# ============================================================
# Project paths
# ============================================================

ROOT = Path(__file__).resolve().parents[1]

RAW_DIR = ROOT / "experiments" / "raw"
TABLE_DIR = ROOT / "experiments" / "results" / "tables"
FIGURE_DIR = ROOT / "experiments" / "results" / "figures"

TABLE_DIR.mkdir(parents=True, exist_ok=True)
FIGURE_DIR.mkdir(parents=True, exist_ok=True)


# ============================================================
# Prover stages to extract from ark-groth16 trace
# ============================================================

PHASES = [
    ("Constraint synthesis", "constraint_ms"),
    ("Inlining LCs", "inlining_lcs_ms"),
    ("R1CS to QAP witness map", "qap_ms"),
    ("Compute C", "compute_c_ms"),
    ("Compute A", "compute_a_ms"),
    ("Compute B in G1", "compute_b_g1_ms"),
    ("Compute B in G2", "compute_b_g2_ms"),
    ("Finish C", "finish_c_ms"),
]


# ============================================================
# Time parsing
# ============================================================

def parse_duration(value: str, unit: str) -> float:
    """
    Convert a duration into milliseconds.
    """

    value = float(value)

    if unit in ("µs", "μs", "us"):
        return value / 1000.0

    if unit == "ms":
        return value

    if unit == "s":
        return value * 1000.0

    raise ValueError(f"Unsupported time unit: {unit}")


def extract_duration(text: str, label: str):
    """
    Extract a stage duration from an ark trace.

    Example:
        End:     Compute C ............ 5.682ms
    """

    pattern = re.compile(
        rf"End:\s+{re.escape(label)}\s+.*?"
        rf"([0-9]+(?:\.[0-9]+)?)\s*"
        rf"(µs|μs|us|ms|s)\s*$",
        re.MULTILINE,
    )

    match = pattern.search(text)

    if match is None:
        return None

    return parse_duration(
        match.group(1),
        match.group(2),
    )


def extract_prover_total_ms(text: str):
    """
    Extract:
        End:     Groth16::Prover ............ 22.947ms
    """

    pattern = re.compile(
        r"End:\s+Groth16::Prover\s+.*?"
        r"([0-9]+(?:\.[0-9]+)?)\s*"
        r"(µs|μs|us|ms|s)\s*$",
        re.MULTILINE,
    )

    match = pattern.search(text)

    if match is None:
        return None

    return parse_duration(
        match.group(1),
        match.group(2),
    )


# ============================================================
# Prover block parsing
# ============================================================

def split_prover_blocks(text: str):
    """
    Split the complete log into Groth16::Prover blocks.

    A command such as:

        bench 1000 2 5

    produces:

        2 warm-up Prover blocks
        +
        5 formal Prover blocks

    Therefore the parser must later keep only the
    final formal blocks.
    """

    pattern = re.compile(
        r"Start:\s+Groth16::Prover\s*"
        r"(.*?)"
        r"(?=\nStart:\s+Groth16::Prover|\Z)",
        re.DOTALL,
    )

    return pattern.findall(text)


# ============================================================
# CSV loading
# ============================================================

def read_benchmark_csv(path: Path):
    """
    Read the clean benchmark CSV generated from the raw log.
    """

    with path.open(
        "r",
        encoding="utf-8-sig",
        newline="",
    ) as f:

        rows = list(csv.DictReader(f))

    if not rows:
        raise RuntimeError(
            f"No benchmark rows found in {path}"
        )

    return rows


# ============================================================
# Raw trace loading
# ============================================================

def read_trace(path: Path):
    """
    Read the full raw .err trace.
    """

    text = path.read_text(
        encoding="utf-8",
        errors="replace",
    )

    blocks = split_prover_blocks(text)

    if not blocks:
        raise RuntimeError(
            f"No Groth16::Prover blocks found in {path}"
        )

    return blocks


# ============================================================
# Statistics
# ============================================================

def median_of(values):
    """
    Median of non-empty values.
    """

    values = [
        value
        for value in values
        if value is not None
    ]

    if not values:
        return None

    return statistics.median(values)


# ============================================================
# Main processing
# ============================================================

def main():

    csv_files = sorted(
        RAW_DIR.glob("prover_n*.csv"),
        key=lambda path: int(
            re.search(
                r"prover_n(\d+)",
                path.name,
            ).group(1)
        ),
    )

    if not csv_files:
        raise RuntimeError(
            "No prover_n*.csv files found in experiments/raw/"
        )

    summary_rows = []
    stage_share_rows = []

    # ========================================================
    # Process every N
    # ========================================================

    for csv_path in csv_files:

        match = re.search(
            r"prover_n(\d+)",
            csv_path.name,
        )

        if match is None:
            continue

        n = int(match.group(1))

        err_path = (
            RAW_DIR /
            f"prover_n{n}.err"
        )

        if not err_path.exists():
            raise RuntimeError(
                f"Missing raw trace: {err_path}"
            )

        benchmark_rows = read_benchmark_csv(
            csv_path
        )

        all_prover_blocks = read_trace(
            err_path
        )

        formal_run_count = len(
            benchmark_rows
        )

        # ----------------------------------------------------
        # Warm-up handling
        # ----------------------------------------------------
        #
        # bench N 2 5
        #
        # => 2 warm-ups
        # => 5 formal runs
        #
        # CSV only contains the 5 formal runs.
        # The .err trace contains all 7 Prover blocks.
        #
        # Therefore use the LAST 5 blocks.
        # ----------------------------------------------------

        if len(all_prover_blocks) < formal_run_count:
            raise RuntimeError(
                f"N={n}: trace blocks "
                f"({len(all_prover_blocks)}) "
                f"< benchmark runs "
                f"({formal_run_count})"
            )

        prover_blocks = all_prover_blocks[
            -formal_run_count:
        ]

        print()
        print(f"N = {n}")
        print(
            f"Benchmark runs: "
            f"{len(benchmark_rows)}"
        )
        print(
            f"All prover trace runs: "
            f"{len(all_prover_blocks)}"
        )
        print(
            f"Formal prover trace runs used: "
            f"{len(prover_blocks)}"
        )

        # ----------------------------------------------------
        # Baseline CSV statistics
        # ----------------------------------------------------

        prove_times = [
            float(row["prove_ms"])
            for row in benchmark_rows
        ]

        verify_times = [
            float(row["verify_ms"])
            for row in benchmark_rows
        ]

        proof_sizes = [
            int(row["proof_bytes"])
            for row in benchmark_rows
        ]

        prove_median = statistics.median(
            prove_times
        )

        verify_median = statistics.median(
            verify_times
        )

        # All runs should normally have identical
        # proof size.
        proof_bytes = proof_sizes[0]

        if any(
            size != proof_bytes
            for size in proof_sizes
        ):
            raise RuntimeError(
                f"N={n}: proof size changed "
                f"between runs: {proof_sizes}"
            )

        # ----------------------------------------------------
        # Prover trace statistics
        # ----------------------------------------------------

        trace_total_times = [
            extract_prover_total_ms(block)
            for block in prover_blocks
        ]

        trace_total_median = median_of(
            trace_total_times
        )

        if trace_total_median is None:
            raise RuntimeError(
                f"N={n}: could not extract "
                f"Groth16::Prover total time"
            )

        phase_values = {}

        for label, field_name in PHASES:

            values = [
                extract_duration(
                    block,
                    label,
                )
                for block in prover_blocks
            ]

            phase_values[field_name] = (
                median_of(values)
            )

        # ----------------------------------------------------
        # Reconciliation / unattributed time
        # ----------------------------------------------------

        profiled_sum = sum(
            value
            for value in phase_values.values()
            if value is not None
        )

        reconciliation_gap = (
            trace_total_median -
            profiled_sum
        )

        # We do NOT call this blindly "Other".
        # If negative, the trace phases overlap or have
        # measurement boundary differences.
        #
        # In the output table we keep the raw gap.
        # This makes the limitation visible.
        # ----------------------------------------------------

        summary = {
            "N": n,

            # Baseline outer timing
            "prove_ms_median": prove_median,
            "verify_ms_median": verify_median,

            # Ark internal timing
            "trace_prover_ms_median":
                trace_total_median,

            "constraint_ms":
                phase_values["constraint_ms"],

            "inlining_lcs_ms":
                phase_values["inlining_lcs_ms"],

            "qap_ms":
                phase_values["qap_ms"],

            "compute_c_ms":
                phase_values["compute_c_ms"],

            "compute_a_ms":
                phase_values["compute_a_ms"],

            "compute_b_g1_ms":
                phase_values["compute_b_g1_ms"],

            "compute_b_g2_ms":
                phase_values["compute_b_g2_ms"],

            "finish_c_ms":
                phase_values["finish_c_ms"],

            "reconciliation_gap_ms":
                reconciliation_gap,

            "proof_bytes":
                proof_bytes,

            "trace_runs_used":
                len(prover_blocks),
        }

        summary_rows.append(summary)

        # ----------------------------------------------------
        # Stage share
        # ----------------------------------------------------

        stages = [
            (
                "Constraint synthesis",
                phase_values["constraint_ms"],
            ),
            (
                "Inlining LCs",
                phase_values["inlining_lcs_ms"],
            ),
            (
                "R1CS → QAP",
                phase_values["qap_ms"],
            ),
            (
                "Compute C",
                phase_values["compute_c_ms"],
            ),
            (
                "Compute A",
                phase_values["compute_a_ms"],
            ),
            (
                "Compute B in G1",
                phase_values["compute_b_g1_ms"],
            ),
            (
                "Compute B in G2",
                phase_values["compute_b_g2_ms"],
            ),
            (
                "Finish C",
                phase_values["finish_c_ms"],
            ),
            (
                "Reconciliation gap",
                reconciliation_gap,
            ),
        ]

        for stage_name, time_ms in stages:

            share = None

            if (
                time_ms is not None
                and trace_total_median != 0
            ):
                share = (
                    time_ms /
                    trace_total_median *
                    100.0
                )

            stage_share_rows.append(
                {
                    "N": n,
                    "stage": stage_name,
                    "time_ms": time_ms,
                    "share_pct": share,
                }
            )

    # ========================================================
    # Sort all output
    # ========================================================

    summary_rows.sort(
        key=lambda row: row["N"]
    )

    stage_share_rows.sort(
        key=lambda row: (
            row["N"],
            row["stage"],
        )
    )

    # ========================================================
    # Output 1:
    # prover_summary.csv
    # ========================================================

    summary_path = (
        TABLE_DIR /
        "prover_summary.csv"
    )

    summary_fields = [
        "N",
        "prove_ms_median",
        "trace_prover_ms_median",
        "constraint_ms",
        "inlining_lcs_ms",
        "qap_ms",
        "compute_c_ms",
        "compute_a_ms",
        "compute_b_g1_ms",
        "compute_b_g2_ms",
        "finish_c_ms",
        "reconciliation_gap_ms",
        "verify_ms_median",
        "proof_bytes",
        "trace_runs_used",
    ]

    with summary_path.open(
        "w",
        encoding="utf-8-sig",
        newline="",
    ) as f:

        writer = csv.DictWriter(
            f,
            fieldnames=summary_fields,
        )

        writer.writeheader()

        for row in summary_rows:

            output = {}

            for field in summary_fields:

                value = row[field]

                if isinstance(
                    value,
                    float,
                ):
                    output[field] = (
                        f"{value:.6f}"
                    )
                else:
                    output[field] = value

            writer.writerow(output)

    # ========================================================
    # Output 2:
    # prover_stage_share.csv
    # ========================================================

    share_path = (
        TABLE_DIR /
        "prover_stage_share.csv"
    )

    with share_path.open(
        "w",
        encoding="utf-8-sig",
        newline="",
    ) as f:

        writer = csv.DictWriter(
            f,
            fieldnames=[
                "N",
                "stage",
                "time_ms",
                "share_pct",
            ],
        )

        writer.writeheader()

        for row in stage_share_rows:

            writer.writerow(
                {
                    "N": row["N"],
                    "stage": row["stage"],
                    "time_ms": (
                        f"{row['time_ms']:.6f}"
                        if row["time_ms"]
                        is not None
                        else ""
                    ),
                    "share_pct": (
                        f"{row['share_pct']:.4f}"
                        if row["share_pct"]
                        is not None
                        else ""
                    ),
                }
            )

    # ========================================================
    # Output 3:
    # prover_scaling.csv
    # ========================================================

    scaling_path = (
        TABLE_DIR /
        "prover_scaling.csv"
    )

    scaling_fields = [
        "N",
        "prove_ms",
        "qap_ms",
        "compute_c_ms",
        "compute_a_ms",
        "compute_b_g1_ms",
        "compute_b_g2_ms",
        "prove_growth_vs_previous",
    ]

    with scaling_path.open(
        "w",
        encoding="utf-8-sig",
        newline="",
    ) as f:

        writer = csv.DictWriter(
            f,
            fieldnames=scaling_fields,
        )

        writer.writeheader()

        previous = None

        for row in summary_rows:

            prove_growth = ""

            if (
                previous is not None
                and previous["prove_ms_median"]
                != 0
            ):
                prove_growth = (
                    row["prove_ms_median"] /
                    previous["prove_ms_median"]
                )

            writer.writerow(
                {
                    "N": row["N"],
                    "prove_ms":
                        f"{row['prove_ms_median']:.6f}",
                    "qap_ms":
                        f"{row['qap_ms']:.6f}",
                    "compute_c_ms":
                        f"{row['compute_c_ms']:.6f}",
                    "compute_a_ms":
                        f"{row['compute_a_ms']:.6f}",
                    "compute_b_g1_ms":
                        f"{row['compute_b_g1_ms']:.6f}",
                    "compute_b_g2_ms":
                        f"{row['compute_b_g2_ms']:.6f}",
                    "prove_growth_vs_previous":
                        (
                            f"{prove_growth:.6f}"
                            if prove_growth != ""
                            else ""
                        ),
                }
            )

            previous = row

    # ========================================================
    # Output 4:
    # GitHub-friendly Markdown table
    # ========================================================

    markdown_path = (
        TABLE_DIR /
        "prover_summary.md"
    )

    with markdown_path.open(
        "w",
        encoding="utf-8",
    ) as f:

        f.write(
            "# Groth16 Prover Profiling Summary\n\n"
        )

        f.write(
            "| N | Prove median (ms) | "
            "Trace median (ms) | "
            "QAP (ms) | Compute C (ms) | "
            "Compute A (ms) | B-G1 (ms) | "
            "B-G2 (ms) | Verify (ms) | "
            "Proof (B) |\n"
        )

        f.write(
            "|---:|---:|---:|---:|---:|"
            "---:|---:|---:|---:|---:|\n"
        )

        for row in summary_rows:

            f.write(
                f"| {row['N']} "
                f"| {row['prove_ms_median']:.3f} "
                f"| {row['trace_prover_ms_median']:.3f} "
                f"| {row['qap_ms']:.3f} "
                f"| {row['compute_c_ms']:.3f} "
                f"| {row['compute_a_ms']:.3f} "
                f"| {row['compute_b_g1_ms']:.3f} "
                f"| {row['compute_b_g2_ms']:.3f} "
                f"| {row['verify_ms_median']:.3f} "
                f"| {row['proof_bytes']} |\n"
            )

        f.write("\n")

        f.write(
            "## Methodology\n\n"
        )

        f.write(
            "- Each benchmark uses 2 warm-up runs "
            "and 5 formal runs.\n"
        )

        f.write(
            "- Baseline timing is taken from "
            "`prover_n*.csv`.\n"
        )

        f.write(
            "- Internal stage timing is taken from "
            "`prover_n*.err`.\n"
        )

        f.write(
            "- Only the final 5 `Groth16::Prover` "
            "trace blocks are used.\n"
        )

        f.write(
            "- Stage trace is coarse-grained and "
            "is not instruction-level profiling.\n"
        )

        f.write(
            "- Stage time is not automatically "
            "interpreted as pure MSM/FFT time.\n"
        )

    # ========================================================
    # Figure 1:
    # Prove time vs constraints
    # ========================================================

    ns = [
        row["N"]
        for row in summary_rows
    ]

    prove_ms = [
        row["prove_ms_median"]
        for row in summary_rows
    ]

    plt.figure(figsize=(8, 5))

    plt.plot(
        ns,
        prove_ms,
        marker="o",
    )

    plt.xscale("log")

    plt.xlabel(
        "Constraints (N)"
    )

    plt.ylabel(
        "Prove time (ms)"
    )

    plt.title(
        "Groth16 Prove Time vs Constraints"
    )

    plt.grid(
        True,
        which="both",
        alpha=0.3,
    )

    plt.tight_layout()

    prove_figure = (
        FIGURE_DIR /
        "prover_time_vs_constraints.png"
    )

    plt.savefig(
        prove_figure,
        dpi=200,
    )

    plt.close()

    # ========================================================
    # Figure 2:
    # Stage breakdown
    # ========================================================

    stage_order = [
        "Constraint synthesis",
        "Inlining LCs",
        "R1CS → QAP",
        "Compute C",
        "Compute A",
        "Compute B in G1",
        "Compute B in G2",
        "Finish C",
        "Reconciliation gap",
    ]

    lookup = {}

    for row in stage_share_rows:

        lookup.setdefault(
            row["N"],
            {},
        )

        lookup[
            row["N"]
        ][row["stage"]] = (
            row["time_ms"]
            if row["time_ms"] is not None
            else 0.0
        )

    x = list(range(len(ns)))

    plt.figure(figsize=(11, 6))

    bottom = [0.0] * len(ns)

    for stage in stage_order:

        values = [
            lookup[n].get(
                stage,
                0.0,
            )
            for n in ns
        ]

        plt.bar(
            x,
            values,
            bottom=bottom,
            label=stage,
        )

        bottom = [
            b + v
            for b, v in zip(
                bottom,
                values,
            )
        ]

    plt.xticks(
        x,
        [str(n) for n in ns],
    )

    plt.xlabel(
        "Constraints (N)"
    )

    plt.ylabel(
        "Time (ms)"
    )

    plt.title(
        "Groth16 Prover Stage Breakdown"
    )

    plt.legend(
        fontsize=8,
    )

    plt.tight_layout()

    stage_figure = (
        FIGURE_DIR /
        "prover_stage_breakdown.png"
    )

    plt.savefig(
        stage_figure,
        dpi=200,
    )

    plt.close()

    # ========================================================
    # Figure 3:
    # Stage share
    # ========================================================

    share_lookup = {}

    for row in stage_share_rows:

        share_lookup.setdefault(
            row["N"],
            {},
        )

        value = row["share_pct"]

        share_lookup[
            row["N"]
        ][row["stage"]] = (
            value
            if value is not None
            else 0.0
        )

    plt.figure(figsize=(11, 6))

    bottom = [0.0] * len(ns)

    for stage in stage_order:

        values = [
            share_lookup[n].get(
                stage,
                0.0,
            )
            for n in ns
        ]

        plt.bar(
            x,
            values,
            bottom=bottom,
            label=stage,
        )

        bottom = [
            b + v
            for b, v in zip(
                bottom,
                values,
            )
        ]

    plt.xticks(
        x,
        [str(n) for n in ns],
    )

    plt.xlabel(
        "Constraints (N)"
    )

    plt.ylabel(
        "Share of trace Prover time (%)"
    )

    plt.title(
        "Groth16 Prover Stage Share"
    )

    plt.legend(
        fontsize=8,
    )

    plt.tight_layout()

    share_figure = (
        FIGURE_DIR /
        "prover_stage_share.png"
    )

    plt.savefig(
        share_figure,
        dpi=200,
    )

    plt.close()

    # ========================================================
    # Final output
    # ========================================================

    print()
    print("=" * 60)
    print("Groth16 Prover profiling analysis completed.")
    print("=" * 60)

    print()
    print("Tables:")
    print(
        f"  {summary_path}"
    )
    print(
        f"  {share_path}"
    )
    print(
        f"  {scaling_path}"
    )
    print(
        f"  {markdown_path}"
    )

    print()
    print("Figures:")
    print(
        f"  {prove_figure}"
    )
    print(
        f"  {stage_figure}"
    )
    print(
        f"  {share_figure}"
    )


if __name__ == "__main__":
    main()