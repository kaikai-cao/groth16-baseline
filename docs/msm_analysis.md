# MSM Microbenchmark Analysis

## 1. Experimental Setup

The MSM microbenchmark evaluates BN254 G1 and G2 variable-base MSM under
1-thread and 20-thread configurations.

The tested input sizes are:

- 2^10
- 2^12
- 2^14
- 2^16

For each configuration, five formal runs are used and the median execution
time is reported.

## 2. Main Observations

MSM execution time increases substantially with input size for both G1 and G2.
The growth is not simply proportional to the number of input points over the
tested range.

G2 MSM is consistently slower than G1 MSM. Under 20 threads, the G2/G1 time
ratio increases from approximately 2.08x at N=1024 to approximately 2.61x at
N=65536.

Parallel execution becomes increasingly effective as the input size grows.
For G1, the measured speedup from 1 thread to 20 threads increases from
approximately 2.17x at N=1024 to 5.92x at N=65536. For G2, the corresponding
speedup increases from approximately 3.33x to 6.95x.

## 3. Relation to Profiling

The prover profiling identified Compute B in G2 and Compute C as major
components of prover execution time at large circuit sizes. The current MSM
microbenchmark shows that large-scale G2 MSM is computationally expensive and
benefits substantially from parallel execution.

However, the microbenchmark does not prove that MSM alone is the bottleneck of
the Groth16 prover. The Compute B-G2 and Compute C stages contain multiple
operations, so their execution time cannot be directly equated with MSM time.

Therefore, MSM, especially G2 MSM, should currently be regarded as a promising
candidate for further fine-grained investigation rather than a confirmed
system-level bottleneck.

## 4. Research Implication

The next optimization stage should connect the standalone MSM results with
the actual MSM operations executed inside the Groth16 prover. The goal is to
measure their contribution to individual prover stages before selecting an
optimization target.