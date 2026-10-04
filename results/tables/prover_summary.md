# Groth16 Prover Profiling Summary

| N | Prove median (ms) | Trace median (ms) | QAP (ms) | Compute C (ms) | Compute A (ms) | B-G1 (ms) | B-G2 (ms) | Verify (ms) | Proof (B) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1000 | 21.244 | 21.222 | 3.139 | 5.990 | 3.066 | 2.898 | 5.476 | 1.049 | 128 |
| 10000 | 77.647 | 77.214 | 11.895 | 22.599 | 9.071 | 9.154 | 22.408 | 1.410 | 128 |
| 100000 | 450.286 | 449.108 | 76.869 | 120.248 | 52.133 | 51.586 | 138.594 | 1.046 | 128 |
| 1000000 | 4024.170 | 4018.000 | 681.383 | 978.854 | 478.513 | 547.475 | 1284.000 | 1.018 | 128 |

## Methodology

- Each benchmark uses 2 warm-up runs and 5 formal runs.
- Baseline timing is taken from `prover_n*.csv`.
- Internal stage timing is taken from `prover_n*.err`.
- Only the final 5 `Groth16::Prover` trace blocks are used.
- Stage trace is coarse-grained and is not instruction-level profiling.
- Stage time is not automatically interpreted as pure MSM/FFT time.
