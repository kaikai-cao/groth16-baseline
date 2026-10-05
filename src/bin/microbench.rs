use std::{
    fs::{File, OpenOptions, create_dir_all},
    hint::black_box,
    io::{BufRead, BufReader, Write},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use ark_bn254::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{CurveGroup, VariableBaseMSM};
use ark_ff::PrimeField;
use ark_std::{UniformRand, test_rng};

const DEFAULT_N: usize = 1 << 10;

const WARMUPS: usize = 2;
const RUNS: usize = 7;

/// Create a unique identifier for one complete benchmark session.
fn session_id() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_nanos()
}

/// Record one MSM measurement.
///
/// Each invocation records:
///
/// session_id, threads, group, N, run, time_ms
fn record_msm_result(
    session: u128,
    group: &str,
    n: usize,
    run: usize,
    time_ms: f64,
) -> std::io::Result<()> {
    const HEADER: &str = "session_id,threads,group,N,run,time_ms";

    create_dir_all("experiments/raw/microbench")?;

    let filename = format!("experiments/raw/microbench/msm_{group}.csv");

    let path = std::path::Path::new(&filename);

    // ---------------------------------
    // Check existing CSV
    // ---------------------------------

    let file_exists_and_nonempty = std::fs::metadata(path)
        .map(|metadata| metadata.len() > 0)
        .unwrap_or(false);

    if file_exists_and_nonempty {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut header = String::new();
        reader.read_line(&mut header)?;

        let header = header.trim_end_matches(['\r', '\n']);

        if header != HEADER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "invalid MSM CSV header in {}: expected '{}', found '{}'",
                    filename, HEADER, header
                ),
            ));
        }
    }

    // ---------------------------------
    // Open CSV for append
    // ---------------------------------

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    // Write header only for a new or empty file.
    if !file_exists_and_nonempty {
        writeln!(file, "{HEADER}")?;
    }

    let threads = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".to_string());

    writeln!(file, "{session},{threads},{group},{n},{run},{time_ms:.3}")?;

    Ok(())
}

fn bench_g1(n: usize) {
    let mut rng = test_rng();

    println!("================================");
    println!("MSM Microbenchmark");
    println!("Group: G1");
    println!("N: {n}");

    // ---------------------------------
    // Prepare input
    // ---------------------------------

    let bases: Vec<G1Affine> = (0..n)
        .map(|_| G1Projective::rand(&mut rng).into_affine())
        .collect();

    let scalars: Vec<_> = (0..n).map(|_| Fr::rand(&mut rng).into_bigint()).collect();

    assert_eq!(bases.len(), scalars.len());

    // ---------------------------------
    // Benchmark session
    // ---------------------------------

    let session = session_id();

    println!("session_id={session}");
    println!("warmups={WARMUPS}");
    println!("formal_runs={RUNS}");

    // ---------------------------------
    // Warm-up
    // ---------------------------------

    println!("Running warm-ups...");

    for warmup in 1..=WARMUPS {
        let result = G1Projective::msm_bigint(&bases, &scalars);

        let _ = black_box(result);

        println!("warm-up={warmup}");
    }

    // ---------------------------------
    // Formal measurement
    // ---------------------------------

    println!("Running formal measurements...");

    for run in 1..=RUNS {
        let start = Instant::now();

        let result = G1Projective::msm_bigint(&bases, &scalars);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        let _ = black_box(result);

        println!("run={run} time={elapsed_ms:.3} ms");

        record_msm_result(session, "g1", n, run, elapsed_ms)
            .expect("failed to record G1 MSM result");
    }

    println!("Recorded: experiments/raw/microbench/msm_g1.csv");
}

fn bench_g2(n: usize) {
    let mut rng = test_rng();

    println!("================================");
    println!("MSM Microbenchmark");
    println!("Group: G2");
    println!("N: {n}");

    // ---------------------------------
    // Prepare input
    // ---------------------------------

    let bases: Vec<G2Affine> = (0..n)
        .map(|_| G2Projective::rand(&mut rng).into_affine())
        .collect();

    let scalars: Vec<_> = (0..n).map(|_| Fr::rand(&mut rng).into_bigint()).collect();

    assert_eq!(bases.len(), scalars.len());

    // ---------------------------------
    // Benchmark session
    // ---------------------------------

    let session = session_id();

    println!("session_id={session}");
    println!("warmups={WARMUPS}");
    println!("formal_runs={RUNS}");

    // ---------------------------------
    // Warm-up
    // ---------------------------------

    println!("Running warm-ups...");

    for warmup in 1..=WARMUPS {
        let result = G2Projective::msm_bigint(&bases, &scalars);

        let _ = black_box(result);

        println!("warm-up={warmup}");
    }

    // ---------------------------------
    // Formal measurement
    // ---------------------------------

    println!("Running formal measurements...");

    for run in 1..=RUNS {
        let start = Instant::now();

        let result = G2Projective::msm_bigint(&bases, &scalars);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        let _ = black_box(result);

        println!("run={run} time={elapsed_ms:.3} ms");

        record_msm_result(session, "g2", n, run, elapsed_ms)
            .expect("failed to record G2 MSM result");
    }

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

    assert!(n > 0, "N must be greater than 0");

    match group {
        "g1" => bench_g1(n),
        "g2" => bench_g2(n),

        _ => {
            eprintln!("Usage: cargo run --release --bin microbench -- [g1|g2] [N]");

            std::process::exit(1);
        }
    }
}
