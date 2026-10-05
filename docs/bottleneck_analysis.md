# Bottleneck Analysis

## 1. Purpose

This document summarizes the current evidence for identifying computational
hotspots in the Groth16 prover.

The analysis follows a progressively finer-grained approach:

```text
End-to-End Prover
        ↓
Coarse-Grained Profiling
        ↓
MSM Breakdown
        ↓
Standalone MSM
        ↓
G1/G2 Group Operations
        ↓
WNAF Internal Trace
        ↓
Cost Attribution
```

The objective is to identify a concrete optimization candidate without
assuming the target in advance.

---

## 2. System-Level Observation

The end-to-end baseline shows that Groth16 proving time increases substantially
as the constraint count grows.

Coarse-grained prover profiling indicates that a large fraction of the
execution time is concentrated in the computation of the A, B, and C-related
components after the R1CS-to-QAP transformation.

However, coarse-grained stages contain multiple operations and therefore cannot
be directly treated as individual algorithmic bottlenecks.

A finer-grained measurement is required.

---

## 3. MSM as a Major Prover Component

Direct instrumentation of the five MSM operations in the Groth16 prover shows
that measured MSM time accounts for a large fraction of Prove Time.

For 1-thread execution:

| Constraints | MSM / Prove |
| ----------: | ----------: |
|       1,000 |       92.6% |
|      10,000 |       92.4% |
|     100,000 |       91.7% |
|   1,000,000 |       90.1% |

For 20-thread execution:

| Constraints | MSM / Prove |
| ----------: | ----------: |
|       1,000 |       70.8% |
|      10,000 |       80.1% |
|     100,000 |       82.6% |
|   1,000,000 |       82.3% |

Therefore, under the tested implementation and environment, MSM is a dominant
computational component of the Groth16 prover.

This conclusion is based on direct measurement of the MSM calls rather than
inference from coarse-grained stages.

---

## 4. B-G2 as the Main MSM Component

Among the five measured MSM operations:

```text
MSM C-H
MSM C-L
MSM A
MSM B-G1
MSM B-G2
```

B-G2 is consistently the most expensive individual MSM.

Its contribution to total measured MSM time is approximately 40% at the larger
tested circuit sizes.

For example:

```text
N = 100,000
1 thread  → 41.9%
20 threads → 38.9%

N = 1,000,000
1 thread  → 43.4%
20 threads → 40.1%
```

Therefore, B-G2 is the primary component selected for lower-level analysis.

---

## 5. G2 Group-Operation Cost

The group-operation microbenchmark compares G1 and G2 updates under matched
workloads.

The measured mixed-update G2/G1 ratios are approximately:

| Bucket Count | G2 / G1 |
| -----------: | ------: |
|          256 |   3.29× |
|        2,048 |   3.29× |
|        8,192 |   3.27× |
|       32,768 |   3.03× |

This demonstrates that G2 group updates are substantially more expensive than
G1 updates under the tested conditions.

This result helps explain why G2-based MSM is more expensive, but it does not
by itself identify the dominant internal stage of B-G2.

---

## 6. WNAF Internal Cost Attribution

The vendorized `ark-ec` implementation was instrumented to expose the internal
stages of the WNAF-based MSM path.

A representative experiment uses:

```text
N = 100,000
threads = 1
```

For B-G2, the measured execution is approximately:

```text
B-G2 MSM                 ≈ 1,165 ms
Bucket accumulation      ≈ 1,034 ms
Prefix reduction         ≈   112 ms
Other internal work      ≈    20 ms
```

Thus, bucket accumulation accounts for approximately 88%–89% of the measured
B-G2 WNAF execution time.

Prefix reduction contributes roughly 10%.

The remaining digit-processing and recombination costs are comparatively small.

Therefore, the internal WNAF analysis narrows the current candidate further:

```text
Groth16 Prover
      ↓
MSM
      ↓
B-G2
      ↓
WNAF
      ↓
Bucket Accumulation
```

---

## 7. Why Prefix Reduction Is Not the Primary Target

The standalone prefix microbenchmark shows that G2 prefix processing is
substantially slower than G1 prefix processing.

However, the internal WNAF trace shows that prefix reduction represents only a
minor fraction of total B-G2 execution time.

Therefore:

```text
A relatively expensive sub-operation
        ≠
The largest total runtime opportunity
```

Optimizing prefix reduction alone would therefore not be expected to provide
the largest possible improvement to B-G2 under the current workload.

This negative result is important because it prevents the optimization direction
from being selected solely from an isolated G2/G1 ratio.

---

## 8. Parallel Scaling

The thread-scaling experiment at `N = 100,000` shows:

| Threads | Prove (ms) | MSM (ms) | Prove Speedup | MSM Speedup |
| ------: | ---------: | -------: | ------------: | ----------: |
|       1 |   3003.423 | 2753.481 |         1.00× |       1.00× |
|       2 |   1539.759 | 1388.973 |         1.95× |       1.98× |
|       4 |    895.387 |  790.277 |         3.35× |       3.48× |
|       8 |    598.051 |  510.853 |         5.02× |       5.39× |
|      12 |    604.115 |  511.573 |         4.97× |       5.38× |
|      16 |    533.568 |  445.080 |         5.63× |       6.19× |
|      20 |    501.156 |  414.052 |         5.99× |       6.65× |

The results demonstrate substantial parallel benefit, but the scaling is far
from ideal linear scaling.

For 20 threads:

```text
Prove speedup ≈ 6.0×
MSM speedup   ≈ 6.65×
```

rather than the theoretical maximum of 20×.

This indicates that parallel execution introduces additional constraints
beyond pure arithmetic throughput.

Possible contributing factors include:

* scheduling overhead;
* memory behavior;
* workload distribution;
* synchronization;
* implementation structure;
* arithmetic cost.

These factors should be considered separately from the arithmetic-level cost
of G2 operations.

---

## 9. Current Bottleneck Conclusion

The current evidence supports the following hierarchy:

```text
System Level
    Groth16 Prover
          ↓
Major Component
    MSM
          ↓
Major Individual MSM
    B-G2
          ↓
Internal Algorithm
    WNAF
          ↓
Dominant Internal Stage
    Bucket Accumulation
```

Therefore, the current primary optimization candidate is:

> **G2 WNAF bucket accumulation**

This is a localization result based on multiple complementary measurements.
It is not yet evidence that a particular optimization technique will improve
the overall system.

---

## 10. Research Status

The baseline and bottleneck-localization phase is currently complete.

The evidence chain has progressed from:

```text
"Prover is expensive"
```

to:

```text
"MSM dominates prover computation"
```

to:

```text
"B-G2 is the largest MSM component"
```

to:

```text
"G2 WNAF bucket accumulation dominates B-G2 execution"
```

No optimization has yet been introduced into the baseline implementation.

This is intentional: the optimization direction should be selected only after
the implementation-level cause of bucket-accumulation cost has been further
understood.

The current baseline therefore serves as the reference point for the next
research phase.

---

## 11. Next Research Questions

The next stage should investigate the causes of the bucket-accumulation cost.

The main questions are:

```text
What operations dominate each bucket update?
        ↓
How is bucket memory accessed?
        ↓
Is the current bucket layout cache-efficient?
        ↓
How is bucket accumulation parallelized?
        ↓
Can accumulation be reorganized or reduced?
        ↓
What trade-offs appear in memory, computation, and parallelism?
```

Any proposed optimization should be evaluated at two levels:

```text
Local Level
G2 / WNAF / Bucket Microbenchmark

        ↓

System Level
End-to-End Groth16 Prover
```

An optimization should not be considered successful based solely on a local
microbenchmark improvement.

---

## 12. Related Data

Raw measurements:

```text
experiments/raw/microbench/msm_g1.csv
experiments/raw/microbench/msm_g2.csv
experiments/raw/microbench/group_add.csv
experiments/raw/microbench/prefix.csv
experiments/raw/msm_trace/msm_breakdown.csv
experiments/raw/msm_trace/trace_n100000_1t.txt
experiments/raw/thread_scaling/prove_scaling.csv
```

Processed results:

```text
results/tables/msm.csv
results/tables/group_add_microbench.csv
results/tables/prefix_microbench.csv
results/tables/microbench_ratio_by_bucket.csv
results/tables/msm_breakdown_summary.csv
results/tables/msm_speedup.csv
results/tables/prover_scaling.csv
```
