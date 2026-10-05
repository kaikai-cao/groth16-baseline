# MSM Microbenchmark and Cost Attribution Analysis

## 1. Purpose

This document summarizes the MSM-focused performance analysis performed on
the Groth16 baseline.

The analysis follows a staged approach:

```text
Groth16 Prover
      ↓
Fine-Grained MSM Breakdown
      ↓
Standalone MSM Benchmark
      ↓
G1/G2 Group-Operation Benchmark
      ↓
WNAF Prefix Benchmark
      ↓
WNAF Internal Trace
      ↓
Cost Attribution
```

The purpose is not only to identify that MSM is expensive, but to determine
which part of the MSM implementation accounts for the observed cost.

---

## 2. Standalone MSM Experimental Setup

The standalone MSM microbenchmark evaluates variable-base MSM over:

* BN254 G1
* BN254 G2

The tested input sizes are:

```text
2^10  = 1,024
2^12  = 4,096
2^14  = 16,384
2^16  = 65,536
```

The formal thread configurations are:

```text
1 thread
20 threads
```

The processed analysis uses the latest five formal measurements for each
configuration and reports the median.

The standalone MSM results are stored in:

```text
results/tables/msm.csv
```

---

## 3. Standalone MSM Results

The current measurements are:

| Group | Threads |      N | Median (ms) |
| ----- | ------: | -----: | ----------: |
| G1    |       1 |  1,024 |       7.108 |
| G1    |       1 |  4,096 |      22.051 |
| G1    |       1 | 16,384 |      76.592 |
| G1    |       1 | 65,536 |     262.892 |
| G1    |      20 |  1,024 |       2.811 |
| G1    |      20 |  4,096 |       6.066 |
| G1    |      20 | 16,384 |      14.106 |
| G1    |      20 | 65,536 |      42.584 |
| G2    |       1 |  1,024 |      22.398 |
| G2    |       1 |  4,096 |      72.043 |
| G2    |       1 | 16,384 |     234.917 |
| G2    |       1 | 65,536 |     800.915 |
| G2    |      20 |  1,024 |       5.542 |
| G2    |      20 |  4,096 |      14.296 |
| G2    |      20 | 16,384 |      36.776 |
| G2    |      20 | 65,536 |     109.493 |

G2 MSM is consistently more expensive than G1 MSM under the tested
configurations.

At 20 threads, the measured G2/G1 ratio is approximately:

```text
N = 1,024    → 1.97×
N = 4,096    → 2.36×
N = 16,384   → 2.61×
N = 65,536   → 2.57×
```
The ratio increases substantially from the smallest workload and remains around
2.6× at the two largest tested sizes. This indicates that the higher cost of G2
MSM remains significant under parallel execution, although the ratio does not
increase monotonically across all tested sizes.

## 4. Parallel MSM Behavior

The measured 1-thread to 20-thread speedups are:

|      N | G1 Speedup | G2 Speedup |
| -----: | ---------: | ---------: |
|  1,024 |      2.53× |      4.04× |
|  4,096 |      3.64× |      5.04× |
| 16,384 |      5.43× |      6.39× |
| 65,536 |      6.17× |      7.31× |

The results show that parallel execution becomes increasingly effective as the
MSM workload becomes larger.

Both G1 and G2 exhibit stronger speedup at larger input sizes, indicating that
parallel execution overhead is relatively more significant for small MSM
workloads.

However, the measured speedups remain far below the ideal 20× scaling
corresponding to 20 threads. The results therefore demonstrate substantial
parallel benefit, but also clear non-linear scaling under the tested
implementation and hardware.

This indicates that parallel execution is affected by factors beyond the
amount of arithmetic work, potentially including:

* parallel scheduling overhead;
* memory behavior;
* workload distribution;
* synchronization;
* implementation structure;
* arithmetic cost.

The standalone benchmark therefore establishes that MSM is both expensive and
substantially parallelizable, but does not by itself explain the source of the
remaining cost.

---

## 5. Connection to the Groth16 Prover

The Groth16 prover was instrumented to measure five concrete MSM operations:

```text
MSM C-H
MSM C-L
MSM A
MSM B-G1
MSM B-G2
```

The formal experiment covers:

```text
N = 1,000
N = 10,000
N = 100,000
N = 1,000,000
```

with:

```text
1 thread
20 threads
```

The resulting MSM measurements are stored in:

```text
experiments/raw/msm_trace/msm_breakdown.csv
```

The processed results are stored in:

```text
results/tables/msm_breakdown_summary.csv
results/tables/msm_speedup.csv
```

---

## 6. MSM Contribution to Prover Time

For the single-thread measurements, the measured MSM operations account for:

```text
N = 1,000       → 92.6% of Prove Time
N = 10,000      → 92.4%
N = 100,000     → 91.7%
N = 1,000,000   → 90.1%
```

Under 20-thread execution, MSM remains a major component:

```text
N = 1,000       → 70.8%
N = 10,000      → 80.1%
N = 100,000     → 82.6%
N = 1,000,000   → 82.3%
```

Therefore, under the tested implementation and workload, MSM is not a minor
auxiliary operation. It is a dominant computational component of the Groth16
prover.

This conclusion is based on direct measurement of the MSM calls rather than
on inference from coarse-grained prover stages.

---

## 7. B-G2 as the Largest Individual MSM Component

Among the five measured MSM operations, B-G2 is consistently the largest
individual component.

Its share of total measured MSM time is approximately:

```text
1 thread:
N = 1,000       → 43.6%
N = 10,000      → 40.7%
N = 100,000     → 41.9%
N = 1,000,000   → 43.4%

20 threads:
N = 1,000       → 32.2%
N = 10,000      → 36.6%
N = 100,000     → 38.9%
N = 1,000,000   → 40.1%
```

B-G2 therefore represents the most important individual MSM operation in the
current prover implementation.

However, identifying B-G2 as the largest MSM component does not yet explain
why B-G2 is expensive. A lower-level analysis is required.

---

## 8. Group-Operation Analysis

The group-operation microbenchmark evaluates G1 and G2 updates under identical
patterns.

The tested bucket counts are:

```text
256
2,048
8,192
32,768
```

The benchmark includes:

* addition;
* subtraction;
* mixed update behavior.

The measured mixed-update G2/G1 ratios are approximately:

| Bucket Count | G2 / G1 |
| -----------: | ------: |
|          256 |   3.29× |
|        2,048 |   3.29× |
|        8,192 |   3.27× |
|       32,768 |   3.03× |

These results show that an individual G2 group update is substantially more
expensive than the corresponding G1 update under the tested conditions.

This provides a plausible low-level explanation for part of the observed
G2-over-G1 MSM cost.

However, group-update cost alone cannot fully explain MSM runtime because the
MSM implementation also includes digit generation, bucket management, prefix
reduction, recombination, and parallel-runtime effects.

---

## 9. WNAF Bucket Mapping

The current vendorized `ark-ec` implementation uses a WNAF-based MSM path.

For the tested circuit sizes, the selected WNAF window determines the bucket
count approximately as follows:

| Circuit Size | WNAF Window | Bucket Count |
| -----------: | ----------: | -----------: |
|        1,000 |           8 |          256 |
|       10,000 |          11 |        2,048 |
|      100,000 |          13 |        8,192 |
|    1,000,000 |          15 |       32,768 |

This mapping connects the end-to-end Groth16 workload sizes with the
bucket-count settings used in the primitive-level benchmarks.

---

## 10. WNAF Prefix Analysis

The prefix microbenchmark evaluates the prefix-scan stage of WNAF bucket
reduction for both G1 and G2.

The measured G2/G1 prefix ratios are:

| Bucket Count | G2 / G1 |
| -----------: | ------: |
|          256 |   3.97× |
|        2,048 |   3.59× |
|        8,192 |   3.65× |
|       32,768 |   2.93× |

The G2 prefix operation is therefore also substantially more expensive than G1
under the tested conditions.

However, a larger G2/G1 ratio does not automatically make prefix processing the
dominant source of total G2 MSM time.

The total cost contribution must be determined from the internal MSM trace.

---

## 11. WNAF Internal Trace

The vendorized `ark-ec` implementation was instrumented to expose internal
timing for the WNAF MSM path.

A representative diagnostic experiment was performed with:

```text
N = 100,000
threads = 1
```

The trace records internal execution components including:

```text
Digit processing
Bucket accumulation
Prefix reduction
Window processing
Recombination
```

The representative B-G2 measurements show approximately:

```text
B-G2 MSM                     ≈ 1,165 ms
Bucket accumulation          ≈ 1,034 ms
Prefix reduction             ≈   112 ms
Other internal work          ≈   20 ms
```

The precise values vary slightly between runs, but the distribution is stable.

Bucket accumulation therefore accounts for approximately:

```text
88%–89%
```

of the measured B-G2 WNAF execution time, while prefix reduction contributes
roughly:

```text
10%
```

The remaining time is comparatively small.

---

## 12. Current Cost Attribution

Combining the experiments gives the following evidence chain:

```text
Groth16 Prover
      │
      └── MSM
            │
            └── B-G2
                  │
                  └── WNAF MSM
                        │
                        ├── Bucket accumulation  ← dominant
                        ├── Prefix reduction
                        ├── Digit processing
                        └── Recombination
```

At the same time, the group-operation benchmark shows that G2 updates are
approximately 3× the cost of G1 updates under comparable tested patterns.

This suggests the following interpretation:

```text
G2 arithmetic is intrinsically more expensive
                ↓
G2 bucket accumulation performs a large number
of these group updates
                ↓
Bucket accumulation dominates B-G2 execution
                ↓
B-G2 becomes the largest individual MSM component
                ↓
MSM becomes a dominant component of Groth16 proving
```

This is stronger evidence than simply observing that “MSM is expensive”,
because each layer of the explanation is supported by a different experiment.

---

## 13. Important Negative Result

The WNAF prefix experiment is particularly useful because it rules out an
initially plausible but incomplete explanation.

Although G2 prefix processing is substantially slower than G1 prefix processing
in isolation, the internal trace shows that prefix reduction occupies only a
minor fraction of total B-G2 execution time.

Therefore:

```text
High G2/G1 ratio
        ≠
Primary total-cost bottleneck
```

The current evidence indicates that reducing prefix cost alone is unlikely to
produce the largest possible improvement to B-G2 under the tested workload.

The primary candidate remains bucket accumulation.

---

## 14. Parallelism Observation

The single-thread WNAF trace shows a strong correspondence between the relative
cost of G1 and G2 bucket processing and the standalone group-operation results.

In contrast, the relative G2/G1 behavior changes under 20-thread execution.

This indicates that parallel execution introduces additional effects beyond
individual elliptic-curve operation cost.

Therefore, future optimization should distinguish between:

```text
Arithmetic-level cost
```

and:

```text
Systems-level cost
```

The current results do not justify assuming that an optimization that improves
one group operation by a given percentage will provide the same percentage
improvement to the parallel MSM or to the complete Groth16 prover.

---

## 15. Current Research Conclusion

The current MSM investigation has progressed from a system-level observation to
an implementation-level candidate.

The evidence supports the following conclusions:

1. MSM is a dominant computational component of the Groth16 prover under the
   tested conditions.

2. B-G2 is consistently the largest individual MSM operation.

3. G2 group updates are substantially more expensive than G1 updates under the
   tested microbenchmark conditions.

4. The WNAF internal trace shows that B-G2 execution is dominated by bucket
   accumulation rather than prefix reduction.

5. The current evidence therefore identifies G2 WNAF bucket accumulation as the
   primary candidate for subsequent optimization investigation.

This is a localization result, not yet an optimization result.

No optimization has been introduced into the baseline at this stage.

---

## 16. Next Research Stage

The next stage should investigate why G2 WNAF bucket accumulation is expensive
and whether its cost can be reduced without introducing unacceptable tradeoffs.

Potential questions include:

```text
What arithmetic operation dominates each bucket update?
        ↓
Can bucket updates be reorganized?
        ↓
Can memory access be improved?
        ↓
Can bucket accumulation be parallelized differently?
        ↓
Can data layout reduce overhead?
        ↓
Can the WNAF window or bucket strategy be improved?
```

Any proposed optimization should first be evaluated at the local MSM or bucket
level and then validated with end-to-end Groth16 measurements.

The baseline measurements in this document serve as the reference point for
that future comparison.

---

## 17. Reproducibility

The following data sources support the current analysis:

```text
experiments/raw/microbench/msm_g1.csv
experiments/raw/microbench/msm_g2.csv
experiments/raw/microbench/group_add.csv
experiments/raw/microbench/prefix.csv
experiments/raw/msm_trace/msm_breakdown.csv
experiments/raw/msm_trace/trace_n100000_1t.txt
```

Processed results include:

```text
results/tables/msm.csv
results/tables/group_add_microbench.csv
results/tables/prefix_microbench.csv
results/tables/microbench_ratio_by_bucket.csv
results/tables/msm_breakdown_summary.csv
results/tables/msm_speedup.csv
```

Figures are generated from the processed datasets and stored under:

```text
results/figures/
```

Raw data should remain unchanged so that all derived results can be
recomputed later.
