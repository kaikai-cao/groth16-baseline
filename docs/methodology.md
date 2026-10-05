# Benchmark Methodology

## 1. Objective

This repository establishes a reproducible performance baseline for the
Groth16 proving system before introducing any optimization.

The methodology is designed to answer three questions in sequence:

1. How does Groth16 performance scale with circuit size?
2. Which prover components account for most of the observed computation?
3. Which low-level operations are suitable candidates for subsequent
   optimization research?

The current experimental scope includes:

* Setup Time
* Witness Generation Time
* Verifying-Key Preparation Time
* Prove Time
* Verify Time
* Proof Size
* Peak Working Set
* Coarse-Grained Prover Profiling
* Fine-Grained MSM Profiling
* Standalone MSM Microbenchmarks
* G1/G2 Group-Operation Microbenchmarks
* WNAF Prefix Microbenchmarks
* FFT / IFFT Microbenchmarks
* Thread-Scaling Experiments

No optimization is applied when establishing the baseline.

---

## 2. Fixed Experimental Environment

Unless otherwise stated, experiments use the same physical machine and
software environment:

* CPU: Intel Core i7-14700
* Physical cores: 20
* Logical processors: 28
* Memory: 32 GB
* OS: Windows 11 Home China Insider Preview
* OS version: 25H2
* OS build: 26220.9568
* Rust: 1.98.1
* Target: `x86_64-pc-windows-msvc`
* Arkworks: 0.6.0
* Curve: BN254
* Scalar field: BN254 `Fr`
* Build profile: Release

Formal experiments explicitly control the Rayon thread count when comparing
parallel execution.

The main comparison uses:

* 1 Rayon thread
* 20 Rayon threads

Exploratory runs with other or environment-default thread counts may remain in
the raw dataset but are not used in the formal 1-thread / 20-thread comparison.

---

## 3. Circuit and Scaling Model

The benchmark uses a repeated-squaring circuit:

```text
z_1 = x^2
z_2 = z_1^2
...
z_N = z_{N-1}^2
```

The final output `z_N` is public.

Each iteration introduces one R1CS constraint, so the benchmark parameter `N`
directly represents the circuit constraint count.

The main end-to-end and prover-profile experiments use:

```text
N = 1,000
N = 10,000
N = 100,000
N = 1,000,000
```

This construction provides a simple and deterministic workload whose size can
be scaled while keeping the underlying circuit structure unchanged.

---

## 4. End-to-End Timing Definitions

### Setup Time

Measured around Groth16 parameter generation.

It represents the cost of constructing the proving and verification
parameters for the selected circuit.

### Witness Generation Time

Measured separately around host-side witness construction.

Witness generation is reported independently and is not included in the
measured Groth16 proving interval.

### Prepare VK Time

Measured separately around verifying-key preprocessing.

### Prove Time

Measured around Groth16 proof generation.

The reported prover time represents the measured proving operation under the
selected experimental configuration.

### Verify Time

Measured around the final Groth16 proof-verification operation.

### Proof Size

The generated proof is serialized using compressed serialization.

The resulting serialized byte length is reported as Proof Size.

Because Groth16 proof structure is constant-size for a fixed curve and
serialization format, the current baseline reports the same compressed proof
size across the tested circuit sizes.

---

## 5. Repetition and Statistical Treatment

The repository contains several experiment families, and their raw repetition
procedures are recorded separately.

### 5.1 End-to-End Baseline

The benchmark command accepts warm-up and formal-run counts. The formal baseline
procedure uses:

```text
2 warm-up runs
5 formal runs
median used for the timing summary
```

Warm-up runs are excluded from formal timing statistics.

### 5.2 Fine-Grained MSM Profiling

The Groth16-internal MSM profiler records the same five MSM categories across
the selected circuit sizes and thread configurations.

The processed MSM analysis retains the latest five formal runs for each
configuration and reports their median.

The five measured MSM categories are:

```text
MSM C-H
MSM C-L
MSM A
MSM B-G1
MSM B-G2
```

### 5.3 Standalone and Primitive Microbenchmarks

Primitive microbenchmarks may collect more formal runs than are finally used
for summary statistics.

For the current group-operation and WNAF-prefix microbenchmarks:

```text
2 warm-up runs
7 formal measurement runs
```

The analysis scripts use the latest five formal runs from the relevant
configuration when generating the processed summary.

This policy allows the raw dataset to retain all recorded measurements while
keeping the summary convention consistent with the main five-run analysis.

### 5.4 Thread-Scaling Experiments

Thread-scaling experiments use multiple thread counts rather than only the two
formal comparison points.

The current thread-scaling study uses:

```text
1, 2, 4, 8, 12, 16, 20 threads
```

For each configuration, the processed result records the number of runs used
for that configuration. The current dataset contains repeated measurements
with more repetitions for selected larger-thread configurations.

Thread-scaling results are used to study scaling behavior rather than to define
the primary end-to-end baseline.

---

## 6. Fine-Grained MSM Measurement

The Groth16 prover implementation is instrumented to measure the five MSM
operations used during proof generation.

The measurements are taken around the actual MSM calls in the prover rather
than inferred from coarse-grained stage timings.

This distinction is important because coarse prover stages may contain several
types of operations, while the fine-grained instrumentation isolates the
measured MSM calls themselves.

For each formal configuration, the analysis reports:

* individual MSM time;
* total measured MSM time;
* MSM fraction of Prove Time;
* B-G2 fraction of total measured MSM time;
* 1-thread to 20-thread speedup.

The current results consistently show that measured MSM time occupies a large
fraction of prover time, making MSM the primary low-level computational
component examined in subsequent analysis.

---

## 7. Standalone MSM Microbenchmark

A separate standalone benchmark measures variable-base MSM over:

* BN254 G1
* BN254 G2

The current evaluation sizes are:

```text
N = 2^10
N = 2^12
N = 2^14
N = 2^16
```

The benchmark compares:

```text
1 thread
20 threads
```

The purpose is not to replace the Groth16 measurements, but to determine
whether the relative behavior of the underlying MSM primitive is consistent
with what is observed inside the prover.

This provides an additional layer of evidence before investigating lower-level
group operations.

---

## 8. G1/G2 Group-Operation Microbenchmark

The group-operation microbenchmark measures the basic elliptic-curve updates
used during WNAF bucket processing.

The current experiment compares G1 and G2 under identical update patterns and
multiple bucket sizes:

```text
256
2048
8192
32768
```

The benchmark evaluates:

* addition;
* subtraction;
* mixed update behavior.

The purpose is to determine whether the higher cost of G2 MSM can be explained,
at least in part, by the higher cost of the underlying G2 group operations.

The experiment is interpreted as a component-level cost analysis rather than
as a complete model of MSM performance.

---

## 9. WNAF Prefix Microbenchmark

A separate prefix benchmark measures the prefix-scan portion of the WNAF bucket
reduction process.

The current bucket sizes are:

```text
256
2048
8192
32768
```

Both G1 and G2 are evaluated using repeated prefix-scan workloads.

The purpose is to compare the relative cost of prefix processing against the
bucket-update stage and to determine whether prefix reduction is a dominant
source of G2 MSM cost.

The prefix benchmark is therefore used together with the internal WNAF trace,
not as a standalone proof of the overall MSM bottleneck.

---

## 10. WNAF Internal Trace

The current vendorized `ark-ec` implementation exposes an additional diagnostic
trace for the WNAF-based MSM path.

For a representative large workload, the trace separates the WNAF execution
into stages including:

```text
Digit processing
Bucket accumulation
Prefix reduction
Recombination
```

The representative fine-grained experiment uses:

```text
N = 100,000
threads = 1
```

with the resulting diagnostic trace preserved under:

```text
experiments/raw/msm_trace/trace_n100000_1t.txt
```

The trace shows that, for the measured B-G2 MSM, bucket accumulation dominates
the internal WNAF execution time, while prefix reduction is substantially
smaller.

This trace is treated as an auxiliary diagnostic dataset. The formal
`msm_breakdown.csv` dataset remains the primary source for the cross-scale
MSM timing tables.

---

## 11. FFT / IFFT Microbenchmark

FFT and inverse FFT are evaluated over the BN254 scalar field.

The tested domain sizes are:

```text
2^10  = 1,024
2^12  = 4,096
2^14  = 16,384
2^16  = 65,536
```

The benchmark compares:

```text
1 thread
20 threads
```

Forward FFT and inverse FFT are measured separately.

The purpose of the FFT experiment is to provide a controlled comparison
against MSM and to understand whether field-transform operations constitute a
competitive source of cost under the tested workload sizes.

The FFT results are not interpreted as proof that FFT is universally cheaper
than MSM; they characterize the current implementation and benchmark
conditions.

---

## 12. Peak Memory Measurement

Peak Working Set is measured separately from timing experiments.

For each main circuit size, an independent process run is used to obtain a
memory observation.

Therefore Peak Working Set is treated as:

* a separate resource metric;
* a single-run observation for each configuration;
* not a five-run timing median.

Peak Working Set should not be directly combined with timing measurements as
though both were obtained under the same repetition procedure.

---

## 13. Randomness

The current benchmark uses:

```text
ark_std::test_rng()
```

This is appropriate for the current experimental baseline, but it is not
intended to represent a production randomness configuration.

Randomness handling is therefore not treated as a performance optimization
target in the current baseline study.

---

## 14. Data Organization

The repository separates raw measurements, processed summaries, figures, and
methodology documentation.

```text
experiments/raw/
        ↓
Raw Experimental Data
        ↓
results/tables/
        ↓
Processed Summary Data
        ↓
results/figures/
        ↓
Generated Figures
```

The main raw-data organization is:

```text
experiments/raw/
├── auxiliary/
├── baseline/
├── microbench/
├── msm_trace/
├── prover/
└── thread_scaling/
```

In particular:

* `baseline/` contains end-to-end baseline measurements and related memory
  observations;
* `prover/` contains coarse-grained prover profiling data;
* `microbench/` contains primitive-level measurements such as MSM, FFT,
  group-operation, and prefix benchmarks;
* `msm_trace/` contains fine-grained MSM timing data and WNAF diagnostic traces;
* `thread_scaling/` contains thread-scaling measurements;
* `auxiliary/` contains smoke tests and other supporting measurements.

Processed CSV and Markdown summaries are stored under:

```text
results/tables/
```

Generated figures are stored under:

```text
results/figures/
```

Raw measurements are preserved separately and should not be overwritten by
processed results.

---

## 15. Derived Metrics

The main derived metrics include:

### MSM Ratio

```text
MSM Ratio = Total Measured MSM Time / Prove Time
```

This measures how much of the measured prover interval is occupied by the
instrumented MSM operations.

### B-G2 Share

```text
B-G2 Share = B-G2 MSM Time / Total Measured MSM Time
```

This identifies the relative contribution of B-G2 within the measured MSM
workload.

### Parallel Speedup

For a thread-count comparison:

```text
Speedup = T_1thread / T_Nthreads
```

Thus:

* speedup > 1 indicates faster execution with the larger thread count;
* speedup ≈ 1 indicates little measurable benefit;
* speedup < 1 indicates the larger thread count is slower.

Parallel speedup is an empirical systems metric and is not a theoretical
complexity measure.

---

## 16. Interpretation Rules

The experiments are intended to establish evidence for optimization research,
not to assume an optimization target in advance.

The following interpretation rules are used.

### 16.1 System-Level vs Component-Level Results

A primitive-level improvement cannot automatically be translated into the same
percentage improvement for end-to-end proving.

For example, a 50% reduction in the runtime of one component does not imply a
50% reduction in total prover time.

The end-to-end effect depends on the component's share of total execution time
and on interactions with the remaining stages.

### 16.2 Profiling Results vs Causal Explanation

A large measured component identifies where execution time is concentrated, but
does not by itself explain why that component is expensive.

Additional source-level or microbenchmark evidence is required before claiming
a concrete implementation cause.

### 16.3 Empirical Scaling vs Asymptotic Complexity

Measured timing curves characterize the current implementation under the tested
environment.

They should not be presented as the theoretical asymptotic complexity of MSM,
FFT, Groth16, or any underlying mathematical primitive.

### 16.4 Cross-Experiment Comparison

Different experiment families may be executed in separate processes or at
different times.

Therefore absolute timing values from independent datasets may differ even
when the nominal workload is the same.

The analysis emphasizes:

* reproducible trends;
* relative cost;
* scaling behavior;
* cross-checking between complementary measurements.

---

## 17. Current Evidence Chain

The current research methodology follows a staged evidence chain:

```text
End-to-End Groth16 Baseline
        ↓
Coarse-Grained Prover Profiling
        ↓
Fine-Grained MSM Profiling
        ↓
Standalone MSM Microbenchmark
        ↓
G1/G2 Group-Operation Microbenchmark
        ↓
WNAF Prefix Microbenchmark
        ↓
WNAF Internal Trace
        ↓
Cost Attribution
```

This sequence is intentional.

The purpose is to avoid jumping directly from:

```text
"MSM is expensive"
```

to:

```text
"therefore optimize MSM"
```

Instead, the methodology progressively narrows the question from system-level
cost to a specific implementation-level candidate.

The current evidence indicates that MSM is a dominant prover component and that
B-G2 is the largest individual MSM component under the tested conditions.
Within the representative WNAF trace, B-G2 time is dominated by bucket
accumulation.

These observations identify a concrete candidate for the next optimization
stage, while preserving the distinction between an experimentally localized
cost and a validated optimization opportunity.

---

## 18. Reproducibility Requirements for Future Optimization

Any future optimization should be evaluated against the established baseline
under controlled conditions.

Unless an experiment explicitly studies a changed condition, the following
should remain fixed:

* circuit family;
* constraint-count definition;
* curve and scalar field;
* compiler and Rust toolchain;
* build profile;
* hardware;
* thread configuration;
* warm-up policy;
* formal-run policy;
* data-processing convention;
* serialization format.

A reported optimization result should include both:

1. local evidence for the optimized component; and
2. end-to-end prover evidence.

Raw measurements must be retained so that processed tables and figures can be
reproduced or rechecked later.

---

## 19. Scope and Limitations

This repository is an experimental baseline for the current implementation,
not a universal benchmark of all Groth16 implementations.

The measured results may be affected by:

* CPU architecture;
* cache and memory hierarchy;
* memory bandwidth;
* compiler optimizations;
* thread scheduling;
* Rayon runtime behavior;
* field arithmetic implementation;
* elliptic-curve representation;
* MSM algorithm and window selection;
* data layout;
* operating-system background activity.

Accordingly, conclusions should be stated as observations about the tested
implementation and environment.

The current baseline is intended to provide a stable reference for subsequent
optimization research and comparative experiments.
