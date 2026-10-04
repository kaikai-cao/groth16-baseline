# Groth16 Baseline

A reproducible experimental baseline for the Groth16 zkSNARK proving system.

## Research Objective

This repository establishes a controlled Groth16 baseline for subsequent profiling and optimization research.

The current experiments focus on:

- Setup Time
- Witness Generation Time
- Verifying-Key Preparation Time
- Prove Time
- Verify Time
- Proof Size
- Peak Working Set
- Constraint Scaling

The baseline is intended to provide a reproducible reference point for later investigations into prover optimization, computational cost reduction, and proof-system performance.

## Experimental Environment

| Item | Value |
|---|---|
| CPU | Intel Core i7-14700 |
| Physical Cores | 20 |
| Logical Processors | 28 |
| RAM | 32 GB |
| OS | Windows 11 Home China Insider Preview |
| OS Version | 25H2 |
| OS Build | 26220.9568 |
| Rust | 1.98.1 |
| Target | x86_64-pc-windows-msvc |
| Arkworks | 0.6.0 |
| Curve | BN254 |
| Build | Release |
| Rayon Threads | 20 |

## Benchmark Protocol

For each circuit size, the benchmark uses:

- 2 warm-up runs
- 5 formal measurement runs
- Median used for timing summary

The benchmark reports the following metrics separately:

- Witness generation time
- Setup time
- Verifying-key preparation time
- Prove time
- Verify time
- Proof size

Proof size is measured after compressed serialization.

Peak Working Set is measured separately in an independent run and is reported as a memory metric rather than being included in the timing median.

## Baseline Results

The current end-to-end Groth16 baseline results are:

| Constraints | Setup (ms) | Witness (ms) | Prepare VK (ms) | Prove (ms) | Verify (ms) | Proof (B) | Peak Memory (MB) |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1,000 | 11.242 | 0.023 | 1.050 | 20.394 | 1.128 | 128 | 11.29 |
| 10,000 | 61.102 | 0.162 | 0.928 | 80.503 | 0.976 | 128 | 23.46 |
| 100,000 | 442.122 | 1.533 | 0.845 | 481.373 | 0.857 | 128 | 114.75 |
| 1,000,000 | 6236.741 | 34.445 | 1.168 | 6394.521 | 1.335 | 128 | 892.23 |

The baseline shows that proof size remains constant at 128 bytes across the tested circuit sizes, while proving and setup costs increase substantially as the number of constraints grows.

![Groth16 Prover Scaling](results/figures/prove_time_vs_constraints.png)

## Repository Structure

```text
groth16-baseline/
├── docs/
│   ├── environment.md
│   └── methodology.md
├── experiments/
│   ├── raw/
│   │   ├── auxiliary/
│   │   ├── baseline_n*.csv
│   │   ├── memory_n*.err
│   │   ├── peak_memory.csv
│   │   ├── prover_n*.csv
│   │   └── prover_n*.err
│   └── results/
│       ├── figures/
│       └── tables/
├── results/
│   ├── figures/
│   │   └── prove_time_vs_constraints.png
│   └── tables/
│       └── baseline_summary.csv
├── scripts/
│   ├── plot_baseline.py
│   ├── plot_prover_profile.py
│   └── requirements.txt
├── src/
│   ├── benchmark.rs
│   ├── circuit.rs
│   ├── groth16.rs
│   └── main.rs
├── Cargo.toml
├── Cargo.lock
├── README.md
└── rust-toolchain.toml
```

## Groth16 Prover Profiling

This repository also records coarse-grained profiling results for the Groth16 prover implementation based on `ark-groth16`. The profiling uses the same repeated-squaring circuit as the baseline benchmark and evaluates four constraint scales: 1,000, 10,000, 100,000, and 1,000,000 constraints. Each scale is executed with 2 warm-up runs and 5 formal measurement runs in the release configuration.

The profiling results were collected in a separate experimental run from the original end-to-end baseline. Therefore, the profiling timing values are not expected to exactly match the original baseline timing measurements.

The prover trace exposes the major implementation stages:

```text
Constraint synthesis
        ↓
Inlining LCs
        ↓
R1CS to QAP witness map
        ↓
Compute C
        ↓
Compute A
        ↓
Compute B in G1
        ↓
Compute B in G2
        ↓
Finish C
```

These measurements are coarse-grained implementation stages rather than instruction-level measurements. In particular, `Compute A`, `Compute B`, and `Compute C` may contain multiple group operations and MSM-related computations and should not be interpreted as pure MSM measurements.

The current profiling summary is:

| Constraints (N) | Prove (ms) | QAP (ms) | Compute C (ms) | Compute A (ms) | Compute B-G1 (ms) | Compute B-G2 (ms) | Verify (ms) | Proof Size (B) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1,000 | 21.244 | 3.139 | 5.990 | 3.066 | 2.898 | 5.476 | 1.049 | 128 |
| 10,000 | 77.647 | 11.895 | 22.599 | 9.071 | 9.154 | 22.408 | 1.410 | 128 |
| 100,000 | 450.286 | 76.869 | 120.248 | 52.133 | 51.586 | 138.594 | 1.046 | 128 |
| 1,000,000 | 4,024.170 | 681.383 | 978.854 | 478.513 | 547.475 | 1,284.000 | 1.018 | 128 |

The profiling results show that prover cost increases substantially with the constraint scale, while the Groth16 proof remains constant at 128 bytes in these experiments. At larger circuit sizes, `Compute B in G2`, `Compute C`, and `R1CS to QAP witness map` are major observed components of prover execution time.

These results are used to guide subsequent fine-grained profiling. They do not by themselves establish a definitive algorithmic bottleneck. The next stage is to further decompose the expensive prover stages and quantify the contribution of MSM, polynomial processing, and other group operations.

The processed profiling data are available in:

- [`prover_summary.csv`](experiments/results/tables/prover_summary.csv)
- [`prover_stage_share.csv`](experiments/results/tables/prover_stage_share.csv)
- [`prover_scaling.csv`](experiments/results/tables/prover_scaling.csv)
- [`prover_summary.md`](experiments/results/tables/prover_summary.md)

The generated figures are available in:

- [`prover_time_vs_constraints.png`](experiments/results/figures/prover_time_vs_constraints.png)
- [`prover_stage_breakdown.png`](experiments/results/figures/prover_stage_breakdown.png)
- [`prover_stage_share.png`](experiments/results/figures/prover_stage_share.png)

The corresponding raw profiling logs and benchmark outputs are stored under:

```text
experiments/raw/
```

The `.err` files contain the complete execution output, including the prover trace and benchmark output. The corresponding `.csv` files contain the structured formal benchmark results.

**Reproducing the Experiments**

Build and run the end-to-end benchmark with:

```powershell
cargo run --release -- bench 1000 2 5
cargo run --release -- bench 10000 2 5
cargo run --release -- bench 100000 2 5
cargo run --release -- bench 1000000 2 5
```

For prover profiling, the raw execution output can be recorded with:

```powershell
cmd /c "cargo run --release -- bench 1000 2 5 > experiments\raw\prover_n1000.err 2>&1"
cmd /c "cargo run --release -- bench 10000 2 5 > experiments\raw\prover_n10000.err 2>&1"
cmd /c "cargo run --release -- bench 100000 2 5 > experiments\raw\prover_n100000.err 2>&1"
cmd /c "cargo run --release -- bench 1000000 2 5 > experiments\raw\prover_n1000000.err 2>&1"
```

The corresponding structured benchmark rows contain the five formal measurement runs.

After collecting the raw profiling results, regenerate the profiling tables and figures with:

```powershell
python scripts/plot_prover_profile.py
```

Because the benchmark uses 2 warm-up runs and 5 formal runs, the raw prover trace contains 7 prover traces for each circuit size. The profiling parser uses only the final 5 formal traces when generating the reported profiling statistics.

**Current Research Direction**

The baseline and coarse-grained profiling establish the first experimental reference point for investigating Groth16 prover performance.

The current evidence indicates that prover cost grows substantially with circuit size and is distributed across multiple implementation stages. In particular, the profiling results identify `Compute B in G2`, `Compute C`, and `R1CS to QAP witness map` as important stages for further investigation at larger constraint scales.

The immediate research direction is therefore:

```text
Baseline
    ↓
Coarse-grained profiling
    ↓
Fine-grained profiling
    ↓
MSM / polynomial-operation analysis
    ↓
Optimization target identification
    ↓
Experimental evaluation
```

The purpose of the next stage is to determine which concrete operations account for the observed costs before selecting an optimization target. Any optimization claim will be evaluated against the baseline using the same experimental methodology and reporting metrics.

**Reproducibility**

The repository is organized so that the experimental results can be traced back to the implementation, benchmark configuration, raw execution output, and result-generation scripts.

The intended workflow is:

```text
Source Code
    ↓
Benchmark Configuration
    ↓
Raw Experimental Output
    ↓
Processed Tables
    ↓
Generated Figures
    ↓
Performance Analysis
```

This structure is intended to support reproducible experiments and provide a stable baseline for future Groth16 performance and optimization studies.