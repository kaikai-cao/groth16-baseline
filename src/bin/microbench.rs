use std::{
    fs::{OpenOptions, create_dir_all},
    hint::black_box,
    io::Write,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use ark_bn254::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{CurveGroup, VariableBaseMSM};
use ark_ff::PrimeField;
use ark_std::{UniformRand, test_rng};

const DEFAULT_N: usize = 1 << 10;

fn record_msm_result(group: &str, n: usize, time_ms: f64) -> std::io::Result<()> {
    create_dir_all("experiments/raw/microbench")?;

    let filename = format!("experiments/raw/microbench/msm_{group}.csv");

    let file_exists = std::path::Path::new(&filename).exists();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filename)?;

    if !file_exists {
        writeln!(file, "timestamp_ms,threads,group,N,time_ms")?;
    }

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_millis();

    let threads = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".to_string());

    writeln!(file, "{timestamp_ms},{threads},{group},{n},{time_ms:.3}")?;

    Ok(())
}

fn bench_g1(n: usize) {
    let mut rng = test_rng();

    println!("================================");
    println!("MSM Microbenchmark");
    println!("Group: G1");
    println!("N: {n}");

    // -------------------------------
    // Prepare input
    // -------------------------------

    let bases: Vec<G1Affine> = (0..n)
        .map(|_| G1Projective::rand(&mut rng).into_affine())
        .collect();

    let scalars: Vec<_> = (0..n).map(|_| Fr::rand(&mut rng).into_bigint()).collect();

    assert_eq!(bases.len(), scalars.len());

    // -------------------------------
    // Warm-up
    // -------------------------------

    let warmup = G1Projective::msm_bigint(&bases, &scalars);

    let _ = black_box(warmup);

    // -------------------------------
    // Formal measurement
    // -------------------------------

    let start = Instant::now();

    let result = G1Projective::msm_bigint(&bases, &scalars);

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let _ = black_box(result);

    // -------------------------------
    // Output
    // -------------------------------

    println!("Time: {elapsed_ms:.3} ms");

    record_msm_result("g1", n, elapsed_ms).expect("failed to record G1 MSM result");

    println!("Recorded: experiments/raw/microbench/msm_g1.csv");
}

fn bench_g2(n: usize) {
    let mut rng = test_rng();

    println!("================================");
    println!("MSM Microbenchmark");
    println!("Group: G2");
    println!("N: {n}");

    // -------------------------------
    // Prepare input
    // -------------------------------

    let bases: Vec<G2Affine> = (0..n)
        .map(|_| G2Projective::rand(&mut rng).into_affine())
        .collect();

    let scalars: Vec<_> = (0..n).map(|_| Fr::rand(&mut rng).into_bigint()).collect();

    assert_eq!(bases.len(), scalars.len());

    // -------------------------------
    // Warm-up
    // -------------------------------

    let warmup = G2Projective::msm_bigint(&bases, &scalars);

    let _ = black_box(warmup);

    // -------------------------------
    // Formal measurement
    // -------------------------------

    let start = Instant::now();

    let result = G2Projective::msm_bigint(&bases, &scalars);

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let _ = black_box(result);

    // -------------------------------
    // Output
    // -------------------------------

    println!("Time: {elapsed_ms:.3} ms");

    record_msm_result("g2", n, elapsed_ms).expect("failed to record G2 MSM result");

    println!("Recorded: experiments/raw/microbench/msm_g2.csv");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // ---------------------------------
    // Usage:
    //
    // cargo run --release --bin microbench -- g1
    // cargo run --release --bin microbench -- g2
    //
    // Or explicitly specify N:
    //
    // cargo run --release --bin microbench -- g2 4096
    // ---------------------------------

    let group = args.get(1).map(String::as_str).unwrap_or("g1");

    let n = args
        .get(2)
        .map(|s| s.parse::<usize>().expect("N must be a positive integer"))
        .unwrap_or(DEFAULT_N);

    assert!(n > 0);

    match group {
        "g1" => bench_g1(n),
        "g2" => bench_g2(n),

        _ => {
            eprintln!("Usage: cargo run --release --bin microbench -- [g1|g2] [N]");

            std::process::exit(1);
        }
    }
}
