# Experimental Environment

## 1. Project Information

- Project: groth16-baseline
- Purpose: Reproducible Groth16 zkSNARK performance baseline
- Experiment Stage: Day 8 - Baseline Benchmark
- Baseline Date: 2026-10-02

## 2. Operating System

- OS: Windows 11 Home China Insider Preview
- Version: 25H2
- OS Build: 26220.9568
- Architecture: x86_64

## 3. Hardware

- CPU: Intel(R) Core(TM) i7-14700
- Physical Cores: 20
- Logical Processors: 28
- RAM: 32 GB

## 4. Rust Environment

- Rust Version: 1.98.1
- Cargo Version: 1.98.1
- Rust Host: x86_64-pc-windows-msvc
- Rust Toolchain: 1.98.1-x86_64-pc-windows-msvc
- Rust Target: x86_64-pc-windows-msvc

## 5. Compiler Environment

- C++ Toolchain: MSVC
- MSVC Linker: Working
- Visual Studio / Build Tools: Visual Studio 2026 Insiders

## 6. Arkworks Environment

- ark-groth16: 0.6.0
- ark-relations: 0.6.0
- ark-ff: 0.6.0
- ark-ec: 0.6.0
- ark-snark: 0.6.0
- Curve: BN254

## 7. Build Configuration

- Build Profile: release
- Optimization: Rust release profile
- Parallel Threads: 20
- Parallel Runtime: Rayon

## 8. Benchmark Configuration

- Circuit Sizes: 1,000 / 10,000 / 100,000 / 1,000,000 constraints
- Warm-up Runs: 2
- Measurement Runs: 5
- Statistical Method: Median
- Timing Method: std::time::Instant
- Proof Size: Compressed serialization length
- Peak Memory: Peak Working Set measured by an external PowerShell monitor
- RNG: ark_std::test_rng()

## 9. Notes

This benchmark measures the current Groth16 implementation under a fixed
hardware and software environment.

Setup time measures Groth16 parameter generation only.

Witness generation is measured separately and is not included in Prove Time.

Prepare VK is measured separately from the final proof verification.

Prove Time measures Groth16 proof generation only.

Verify Time measures the final Groth16 proof verification operation.

Proof size is measured using compressed proof serialization.

Peak memory is a single-run Peak Working Set measurement and is therefore
not a five-run median.

The 1,000,000-constraint smoke-test result is retained as auxiliary data and
is not used in the formal five-run timing baseline.

Because the operating system is a Windows Insider Preview build, the exact
OS build number is recorded for reproducibility.

Any environment change that may affect performance should be recorded and
committed to version control.