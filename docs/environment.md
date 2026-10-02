# Experimental Environment

## 1. Project Information

- Project: groth16-baseline
- Purpose: Reproducible Groth16 zkSNARK baseline
- Experiment Stage: Day 8 - Baseline Setup

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
- MSVC Linker: Available
- Visual Studio / Build Tools: Visual Studio 2026 Insiders

## 6. Arkworks Environment

- ark-groth16:
- ark-relations:
- ark-ff:
- ark-ec:
- ark-snark:

## 7. Build Configuration

- Build Profile: dev / release
- Optimization Level:
- Parallel Threads:

## 8. Benchmark Configuration

- Warm-up Runs:
- Measurement Runs:
- Statistical Method:
- Timing Method:

## 9. Reproducibility Notes

This experiment is conducted on Windows 11 Home China Insider Preview,
version 25H2, OS build 26220.9568.

The Rust toolchain is pinned to Rust 1.98.1 with the
x86_64-pc-windows-msvc target.

Hardware, operating system, Rust toolchain, compiler configuration,
cryptographic library versions, build profile, and parallelism settings
should remain fixed during baseline and comparative experiments whenever
possible.

Any environment change that may affect performance should be recorded
and committed to version control.

Because this system uses a Windows Insider Preview build, the exact OS
build number is recorded explicitly for reproducibility.