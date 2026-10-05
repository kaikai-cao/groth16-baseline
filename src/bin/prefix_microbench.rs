use std::fs::{OpenOptions, create_dir_all, metadata};
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ark_bn254::{G1Projective, G2Projective};
use ark_std::{UniformRand, Zero};

const DEFAULT_BUCKET_COUNT: usize = 8192;
const REPEATS: usize = 100;
const WARMUPS: usize = 2;
const RUNS: usize = 7;

fn parse_bucket_count() -> usize {
    let bucket_count = std::env::args()
        .nth(1)
        .map(|value| value.parse::<usize>().expect("invalid bucket_count"))
        .unwrap_or(DEFAULT_BUCKET_COUNT);

    assert!(bucket_count > 0, "bucket_count must be > 0");
    bucket_count
}

fn make_g1_buckets(bucket_count: usize) -> Vec<G1Projective> {
    let mut rng = ark_std::test_rng();

    (0..bucket_count)
        .map(|_| G1Projective::rand(&mut rng))
        .collect()
}

fn make_g2_buckets(bucket_count: usize) -> Vec<G2Projective> {
    let mut rng = ark_std::test_rng();

    (0..bucket_count)
        .map(|_| G2Projective::rand(&mut rng))
        .collect()
}

fn bench_g1(buckets: &[G1Projective]) -> f64 {
    let start = Instant::now();

    for _ in 0..REPEATS {
        let mut running_sum = G1Projective::zero();
        let mut res = G1Projective::zero();

        for bucket in buckets.iter().rev() {
            running_sum += bucket;
            res += &running_sum;
        }

        let _ = black_box(res);
    }

    start.elapsed().as_secs_f64() * 1000.0
}

fn bench_g2(buckets: &[G2Projective]) -> f64 {
    let start = Instant::now();

    for _ in 0..REPEATS {
        let mut running_sum = G2Projective::zero();
        let mut res = G2Projective::zero();

        for bucket in buckets.iter().rev() {
            running_sum += bucket;
            res += &running_sum;
        }

        let _ = black_box(res);
    }

    start.elapsed().as_secs_f64() * 1000.0
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}

fn session_id() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX_EPOCH")
        .as_millis()
}

fn raw_csv_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("experiments")
        .join("raw")
        .join("microbench")
        .join("prefix.csv")
}

fn open_csv(path: &PathBuf) -> std::io::Result<std::fs::File> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let exists_and_nonempty = metadata(path).map(|m| m.len() > 0).unwrap_or(false);

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    if !exists_and_nonempty {
        writeln!(
            file,
            "session_id,bucket_count,repeats,total_additions,group,run,total_ms,ns_per_addition"
        )?;
    }

    Ok(file)
}

fn write_result(
    file: &mut std::fs::File,
    session: u128,
    bucket_count: usize,
    group: &str,
    run: usize,
    total_ms: f64,
) -> std::io::Result<()> {
    let total_additions = bucket_count * REPEATS * 2;
    let ns_per_addition = total_ms * 1_000_000.0 / total_additions as f64;

    writeln!(
        file,
        "{session},{bucket_count},{REPEATS},{total_additions},{group},{run},{total_ms:.3},{ns_per_addition:.3}"
    )
}

fn main() {
    let bucket_count = parse_bucket_count();

    println!("=== Groth16 G1/G2 Prefix Sum Microbenchmark ===");
    println!("bucket_count={bucket_count}");
    println!("repeats={REPEATS}");
    println!("warmups={WARMUPS}");
    println!("runs={RUNS}");
    println!();

    println!("Generating bucket inputs...");

    let g1_buckets = make_g1_buckets(bucket_count);
    let g2_buckets = make_g2_buckets(bucket_count);

    println!("Input generation complete.");
    println!();

    let path = raw_csv_path();
    let mut file = open_csv(&path).expect("failed to open prefix CSV");
    let session = session_id();

    println!("Raw CSV: {}", path.display());
    println!("session_id={session}");
    println!();

    println!("Running warmups...");

    for _ in 0..WARMUPS {
        black_box(bench_g1(&g1_buckets));
        black_box(bench_g2(&g2_buckets));
    }

    println!();

    let mut g1_times = Vec::with_capacity(RUNS);
    let mut g2_times = Vec::with_capacity(RUNS);

    for run in 1..=RUNS {
        let g1 = bench_g1(&g1_buckets);
        let g2 = bench_g2(&g2_buckets);

        g1_times.push(g1);
        g2_times.push(g2);

        write_result(&mut file, session, bucket_count, "G1", run, g1)
            .expect("failed to write G1 prefix result");
        write_result(&mut file, session, bucket_count, "G2", run, g2)
            .expect("failed to write G2 prefix result");

        file.flush().expect("failed to flush prefix CSV");

        println!("run={run} g1={g1:.3}ms g2={g2:.3}ms ratio={:.3}x", g2 / g1);
    }

    let g1 = median(&mut g1_times);
    let g2 = median(&mut g2_times);
    let total_additions = bucket_count * REPEATS * 2;

    println!();
    println!("=== Median Results (all 7 formal runs) ===");
    println!();

    println!("G1 prefix : {g1:.3} ms");
    println!("G2 prefix : {g2:.3} ms");
    println!("G2/G1     : {:.3}x", g2 / g1);
    println!("Total group additions per run: {total_additions}");
    println!(
        "Per addition: G1={:.3} ns, G2={:.3} ns",
        g1 * 1_000_000.0 / total_additions as f64,
        g2 * 1_000_000.0 / total_additions as f64,
    );
}
