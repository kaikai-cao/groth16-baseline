# Experimental Environment

## 1. Project Information

* Project: `groth16-baseline`
* Purpose: Reproducible Groth16 zkSNARK performance baseline
* Current Research Stage: Baseline profiling and cost attribution
* Baseline Dataset Date: 2026-10-02
* Document Updated: 2026-10-05

The current stage focuses on identifying concrete computational cost
concentrations in the Groth16 prover before introducing any optimization.

---

## 2. Operating System

* OS: Windows 11 Home China Insider Preview
* Version: 25H2
* OS Build: 26220.9568
* Architecture: x86_64

The exact OS build is recorded because system-level scheduling and runtime
behavior may affect benchmark results.

---

## 3. Hardware

* CPU: Intel(R) Core(TM) i7-14700
* Physical Cores: 20
* Logical Processors: 28
* RAM: 32 GB

All formal experiments are performed on this same physical machine unless
otherwise stated.

---

## 4. Rust Environment

* Rust Version: 1.98.1
* Cargo Version: 1.98.1
* Rust Host: `x86_64-pc-windows-msvc`
* Rust Toolchain: `1.98.1-x86_64-pc-windows-msvc`
* Rust Target: `x86_64-pc-windows-msvc`

---

## 5. Compiler Environment

* C++ Toolchain: MSVC
* MSVC Linker: Working
* Visual Studio / Build Tools: Visual Studio 2026 Insiders

All reported timing measurements use the Rust Release build configuration.

---

## 6. Arkworks Environment

The current implementation is based on Arkworks 0.6.0 components:

* `ark-groth16`: 0.6.0
* `ark-relations`: 0.6.0
* `ark-ff`: 0.6.0
* `ark-ec`: 0.6.0
* `ark-snark`: 0.6.0

Cryptographic configuration:

* Curve: BN254
* Scalar Field: BN254 `Fr`

The repository vendors the modified `ark-groth16` and `ark-ec` sources used for
fine-grained prover and MSM instrumentation.

---

## 7. Build and Parallel Configuration

* Build Profile: `release`
* Optimization: Rust Release profile
* Parallel Runtime: Rayon

Formal performance comparisons explicitly control the Rayon thread count.

The main formal comparison uses:

```text
1 thread
20 threads
```

The thread-scaling experiment additionally evaluates:

```text
1, 2, 4, 8, 12, 16, 20 threads
```

Exploratory measurements using other or environment-default thread counts may
remain in the raw dataset but are excluded from the formal 1-thread /
20-thread comparison.

---

## 8. End-to-End Benchmark Configuration

The main repeated-squaring circuit is evaluated at:

```text
1,000
10,000
100,000
1,000,000 constraints
```

The standard end-to-end timing procedure uses:

```text
2 warm-up runs
5 formal runs
median used for timing summaries
```

Measured quantities include:

* Setup Time
* Witness Generation Time
* Prepare VK Time
* Prove Time
* Verify Time
* Proof Size

Timing measurements use:

```text
std::time::Instant
```

Proof size is measured using compressed proof serialization.

---

## 9. Fine-Grained MSM Configuration

The Groth16 prover is instrumented to measure five MSM operations:

```text
MSM C-H
MSM C-L
MSM A
MSM B-G1
MSM B-G2
```

The formal circuit sizes are:

```text
1,000
10,000
100,000
1,000,000 constraints
```

The formal thread configurations are:

```text
1 thread
20 threads
```

The processed analysis retains the latest five formal measurements for each
configuration and reports the median.

The primary MSM breakdown data is stored in:

```text
experiments/raw/msm_trace/msm_breakdown.csv
```

---

## 10. Primitive Microbenchmark Configuration

The repository also contains independent microbenchmarks for low-level
operations relevant to MSM and FFT behavior.

### Standalone MSM

Evaluated over:

* BN254 G1
* BN254 G2

with:

```text
N = 2^10
N = 2^12
N = 2^14
N = 2^16
```

and:

```text
1 thread
20 threads
```

### Group Operations

G1 and G2 group updates are evaluated at bucket counts:

```text
256
2048
8192
32768
```

The benchmark includes addition, subtraction, and mixed update patterns.

### WNAF Prefix

G1 and G2 prefix-scan behavior is evaluated at the same bucket counts:

```text
256
2048
8192
32768
```

For the current group-operation and prefix microbenchmarks:

```text
2 warm-up runs
7 formal runs
```

The raw dataset retains all seven formal runs, while the analysis scripts use
the latest five formal runs when producing summary tables.

---

## 11. WNAF Diagnostic Trace

The vendorized `ark-ec` implementation contains diagnostic instrumentation
for the WNAF-based MSM path.

The representative trace configuration is:

```text
N = 100,000
threads = 1
```

The trace records internal stages including:

```text
Digit processing
Bucket accumulation
Prefix reduction
Window processing
Recombination
```

The diagnostic trace is retained at:

```text
experiments/raw/msm_trace/trace_n100000_1t.txt
```

This trace is an auxiliary diagnostic dataset and is used to explain the
internal structure of the observed B-G2 MSM cost.

---

## 12. FFT / IFFT Configuration

FFT and inverse FFT are evaluated over BN254 `Fr` at:

```text
2^10  = 1,024
2^12  = 4,096
2^14  = 16,384
2^16  = 65,536
```

The formal comparison uses:

```text
1 thread
20 threads
```

Forward FFT and inverse FFT are measured separately.

---

## 13. Peak Memory Measurement

Peak memory is measured separately from timing experiments.

The reported metric is:

```text
Peak Working Set
```

The measurement is obtained from an independent process run using an external
PowerShell monitoring procedure.

Peak memory is therefore treated as a resource observation rather than a
timing median.

The 1,000,000-constraint smoke-test and other auxiliary memory measurements
are retained separately from the formal timing datasets.

---

## 14. Randomness

The current implementation uses:

```text
ark_std::test_rng()
```

This is suitable for the current experimental baseline but should not be
interpreted as a production randomness configuration.

Randomness configuration is not currently considered an optimization target.

---

## 15. Data Organization

Raw experimental measurements are stored under:

```text
experiments/raw/
```

The current organization is:

```text
experiments/raw/
├── auxiliary/
├── baseline/
├── microbench/
├── msm_trace/
├── prover/
└── thread_scaling/
```

Processed summary data is stored under:

```text
results/tables/
```

Generated figures are stored under:

```text
results/figures/
```

The separation is intentional:

```text
Raw Measurements
        ↓
Processed Tables
        ↓
Figures and Analysis
```

Raw measurements should be preserved and should not be replaced by processed
results.

---

## 16. Reproducibility Notes

The following factors are kept fixed whenever possible:

* hardware;
* operating-system build;
* Rust toolchain;
* compiler target;
* Arkworks version;
* curve and scalar field;
* circuit construction;
* release build;
* benchmark procedure;
* thread configuration.

Any change that may affect performance should be documented in the experiment
record and committed to version control.

Because the operating system is a Windows Insider Preview build, the exact OS
build is explicitly recorded for reproducibility.

---

## 17. Interpretation Scope

The measurements characterize the current implementation under the documented
hardware and software environment.

They are not intended to represent universal performance bounds for Groth16.

In particular, observed timings may depend on:

* CPU architecture;
* cache and memory hierarchy;
* memory bandwidth;
* compiler optimizations;
* Rayon scheduling behavior;
* finite-field implementation;
* elliptic-curve implementation;
* MSM algorithm and window selection;
* data layout;
* operating-system background activity.

Therefore, performance conclusions should be reported as empirical observations
under the tested conditions.

The current environment serves as the fixed reference platform for subsequent
Groth16 optimization experiments.
