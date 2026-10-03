# Benchmark Methodology

## 1. Objective

This experiment establishes a reproducible performance baseline for the
Groth16 proving system before any optimization is introduced.

The baseline focuses on:

- Setup Time
- Witness Generation Time
- Verifying-Key Preparation Time
- Prove Time
- Verify Time
- Proof Size
- Peak Working Set

## 2. Fixed Conditions

All formal measurements use:

- BN254
- arkworks 0.6.0
- Rust 1.98.1
- x86_64-pc-windows-msvc
- Release build
- 20 Rayon threads
- Fixed circuit construction
- Fixed benchmark procedure

## 3. Circuit Scaling

The benchmark uses a repeated squaring circuit:

z_1 = x^2
z_2 = z_1^2
...
z_N = z_{N-1}^2

The final output z_N is public.

Each iteration introduces one R1CS constraint, so N directly represents
the number of constraints.

## 4. Timing Definitions

### Setup Time

Measured only around the Groth16 parameter-generation function.

### Witness Generation Time

Measured separately around host-side witness preparation.

This value is not included in Prove Time.

### Prepare VK Time

Measured separately around verifying-key preprocessing.

### Prove Time

Measured only around Groth16 proof generation.

### Verify Time

Measured only around the final Groth16 proof verification operation.

### Proof Size

The proof is compressed and serialized. The resulting byte length is
reported as Proof Size.

## 5. Repetition

Each circuit size uses:

- 2 warm-up runs
- 5 formal measurement runs

Only the five formal runs are used to calculate the median timing result.

## 6. Peak Memory

Peak Working Set is measured separately by monitoring the benchmark process
during one independent run for each circuit size.

Therefore Peak Memory is a single-run measurement and is not a five-run median.

## 7. Randomness

The current benchmark uses `ark_std::test_rng()`.

This is suitable for the current experimental implementation but should not
be interpreted as a production randomness configuration.

## 8. Data Organization

Raw timing results are stored in:

`experiments/raw/`

Processed summary results are stored in:

`results/tables/`

Figures generated from the summary data are stored in:

`results/figures/`

Auxiliary smoke-test and memory-probe outputs are kept separately from the
formal baseline timing data.

## 9. Interpretation

The results characterize this implementation under the fixed experimental
environment. They are not intended to be universal performance bounds for
Groth16.

The baseline is used as the reference point for future profiling and
optimization experiments.