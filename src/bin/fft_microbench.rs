use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;

use ark_bn254::Fr;
use ark_ff::UniformRand;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::test_rng;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n = args
        .get(1)
        .expect("missing N")
        .parse::<usize>()
        .expect("invalid N");

    let warmups = args
        .get(2)
        .expect("missing warmups")
        .parse::<usize>()
        .expect("invalid warmups");

    let runs = args
        .get(3)
        .expect("missing runs")
        .parse::<usize>()
        .expect("invalid runs");

    let domain =
        GeneralEvaluationDomain::<Fr>::new(n).expect("failed to construct evaluation domain");

    let domain_size = domain.size();

    println!("Requested N = {}", n);
    println!("Actual domain size = {}", domain_size);

    // ------------------------------------------------------------
    // Generate deterministic input data.
    // This is outside the timed region.
    // ------------------------------------------------------------
    let mut rng = test_rng();

    let coefficients: Vec<Fr> = (0..domain_size).map(|_| Fr::rand(&mut rng)).collect();

    // ------------------------------------------------------------
    // Correctness check: FFT followed by IFFT should recover input.
    // ------------------------------------------------------------
    let mut check = coefficients.clone();

    domain.fft_in_place(&mut check);
    domain.ifft_in_place(&mut check);

    assert_eq!(
        check, coefficients,
        "FFT/IFFT round-trip correctness check failed"
    );

    // ------------------------------------------------------------
    // Prepare input for inverse FFT.
    // We do one forward FFT outside the timed region.
    // ------------------------------------------------------------
    let mut evaluations = coefficients.clone();
    domain.fft_in_place(&mut evaluations);

    // ------------------------------------------------------------
    // Output file
    // ------------------------------------------------------------
    let path = "experiments/raw/microbench/fft.csv";

    std::fs::create_dir_all("experiments/raw/microbench")
        .expect("failed to create output directory");

    let file_exists_and_nonempty = std::fs::metadata(path)
        .map(|m| m.len() > 0)
        .unwrap_or(false);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("failed to open FFT CSV");

    if !file_exists_and_nonempty {
        writeln!(file, "N,domain_size,threads,run,forward_ms,inverse_ms")
            .expect("failed to write FFT CSV header");
    }

    let threads = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".to_string());

    println!("N,domain_size,threads,run,forward_ms,inverse_ms");

    // ------------------------------------------------------------
    // Warm-up
    // ------------------------------------------------------------
    for _ in 0..warmups {
        let mut data = coefficients.clone();
        domain.fft_in_place(&mut data);

        let mut data = evaluations.clone();
        domain.ifft_in_place(&mut data);
    }

    // ------------------------------------------------------------
    // Formal benchmark
    // ------------------------------------------------------------
    for run in 1..=runs {
        // -------------------------
        // Forward FFT
        // -------------------------
        let mut forward_data = coefficients.clone();

        let start = Instant::now();

        domain.fft_in_place(&mut forward_data);

        let forward_ms = start.elapsed().as_secs_f64() * 1000.0;

        // -------------------------
        // Inverse FFT
        // -------------------------
        let mut inverse_data = evaluations.clone();

        let start = Instant::now();

        domain.ifft_in_place(&mut inverse_data);

        let inverse_ms = start.elapsed().as_secs_f64() * 1000.0;

        // -------------------------
        // Sanity check
        // -------------------------
        assert_eq!(forward_data, evaluations, "forward FFT result mismatch");

        assert_eq!(inverse_data, coefficients, "inverse FFT result mismatch");

        let line = format!(
            "{},{},{},{},{:.3},{:.3}",
            n, domain_size, threads, run, forward_ms, inverse_ms,
        );

        println!("{}", line);

        writeln!(file, "{}", line).expect("failed to write FFT benchmark result");
    }
}
