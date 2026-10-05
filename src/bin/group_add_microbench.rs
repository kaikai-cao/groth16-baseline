use std::fs::{OpenOptions, create_dir_all, metadata};
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ark_bn254::{G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::VariableBaseMSM;
use ark_std::UniformRand;

const DEFAULT_BUCKET_COUNT: usize = 8192;
const N_UPDATES: usize = 500_000;
const WARMUPS: usize = 2;
const RUNS: usize = 7;

/// Generate a fixed bucket access pattern.
///
/// G1/G2 use exactly the same bucket indices and signs; only the group type
/// changes. `bucket_count` is passed explicitly so the benchmark can match
/// the bucket count used by a particular MSM configuration.
fn make_pattern(bucket_count: usize) -> (Vec<usize>, Vec<bool>) {
    let mut indices = Vec::with_capacity(N_UPDATES);
    let mut signs = Vec::with_capacity(N_UPDATES);

    let mut state: u64 = 0x1234_5678_9abc_def0;

    for _ in 0..N_UPDATES {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);

        let index = (state as usize) % bucket_count;
        let positive = (state >> 63) == 0;

        indices.push(index);
        signs.push(positive);
    }

    (indices, signs)
}

fn make_g1_bases() -> Vec<G1Affine> {
    let mut rng = ark_std::test_rng();

    (0..N_UPDATES).map(|_| G1Affine::rand(&mut rng)).collect()
}

fn make_g2_bases() -> Vec<G2Affine> {
    let mut rng = ark_std::test_rng();

    (0..N_UPDATES).map(|_| G2Affine::rand(&mut rng)).collect()
}

/// Only execute bucket += base.
fn g1_add_only(bases: &[G1Affine], indices: &[usize], bucket_count: usize) -> f64 {
    let mut buckets = vec![<G1Projective as VariableBaseMSM>::ZERO_BUCKET; bucket_count];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] += &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// Only execute bucket -= base.
fn g1_sub_only(bases: &[G1Affine], indices: &[usize], bucket_count: usize) -> f64 {
    let mut buckets = vec![<G1Projective as VariableBaseMSM>::ZERO_BUCKET; bucket_count];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] -= &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// Execute += / -= according to the fixed sign pattern.
fn g1_mixed(bases: &[G1Affine], indices: &[usize], signs: &[bool], bucket_count: usize) -> f64 {
    let mut buckets = vec![<G1Projective as VariableBaseMSM>::ZERO_BUCKET; bucket_count];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        if signs[i] {
            buckets[indices[i]] += &bases[i];
        } else {
            buckets[indices[i]] -= &bases[i];
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// G2 += base.
fn g2_add_only(bases: &[G2Affine], indices: &[usize], bucket_count: usize) -> f64 {
    let mut buckets = vec![<G2Projective as VariableBaseMSM>::ZERO_BUCKET; bucket_count];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] += &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// G2 -= base.
fn g2_sub_only(bases: &[G2Affine], indices: &[usize], bucket_count: usize) -> f64 {
    let mut buckets = vec![<G2Projective as VariableBaseMSM>::ZERO_BUCKET; bucket_count];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] -= &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// Execute += / -= according to the fixed sign pattern.
fn g2_mixed(bases: &[G2Affine], indices: &[usize], signs: &[bool], bucket_count: usize) -> f64 {
    let mut buckets = vec![<G2Projective as VariableBaseMSM>::ZERO_BUCKET; bucket_count];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        if signs[i] {
            buckets[indices[i]] += &bases[i];
        } else {
            buckets[indices[i]] -= &bases[i];
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
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

fn parse_bucket_count() -> usize {
    let bucket_count = std::env::args()
        .nth(1)
        .map(|value| value.parse::<usize>().expect("invalid bucket_count"))
        .unwrap_or(DEFAULT_BUCKET_COUNT);

    assert!(bucket_count > 0, "bucket_count must be > 0");
    bucket_count
}

fn raw_csv_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("experiments")
        .join("raw")
        .join("microbench")
        .join("group_add.csv")
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
            "session_id,bucket_count,updates,group,operation,run,total_ms,ns_per_update"
        )?;
    }

    Ok(file)
}

fn write_result(
    file: &mut std::fs::File,
    session: u128,
    bucket_count: usize,
    group: &str,
    operation: &str,
    run: usize,
    total_ms: f64,
) -> std::io::Result<()> {
    let ns_per_update = total_ms * 1_000_000.0 / N_UPDATES as f64;

    writeln!(
        file,
        "{session},{bucket_count},{N_UPDATES},{group},{operation},{run},{total_ms:.3},{ns_per_update:.3}"
    )
}

fn main() {
    let bucket_count = parse_bucket_count();

    println!("=== Groth16 G1/G2 Bucket Operation Microbenchmark ===");
    println!("bucket_count={bucket_count}");
    println!("updates={N_UPDATES}");
    println!("warmups={WARMUPS}");
    println!("runs={RUNS}");
    println!();

    println!("Generating benchmark inputs...");

    let (indices, signs) = make_pattern(bucket_count);

    let g1_bases = make_g1_bases();
    let g2_bases = make_g2_bases();

    println!("Input generation complete.");
    println!();

    let path = raw_csv_path();
    let mut file = open_csv(&path).expect("failed to open group-add CSV");
    let session = session_id();

    println!("Raw CSV: {}", path.display());
    println!("session_id={session}");
    println!();

    println!("Running warmups...");

    for _ in 0..WARMUPS {
        black_box(g1_add_only(&g1_bases, &indices, bucket_count));
        black_box(g2_add_only(&g2_bases, &indices, bucket_count));

        black_box(g1_sub_only(&g1_bases, &indices, bucket_count));
        black_box(g2_sub_only(&g2_bases, &indices, bucket_count));

        black_box(g1_mixed(&g1_bases, &indices, &signs, bucket_count));
        black_box(g2_mixed(&g2_bases, &indices, &signs, bucket_count));
    }

    let mut g1_add_times = Vec::with_capacity(RUNS);
    let mut g2_add_times = Vec::with_capacity(RUNS);
    let mut g1_sub_times = Vec::with_capacity(RUNS);
    let mut g2_sub_times = Vec::with_capacity(RUNS);
    let mut g1_mixed_times = Vec::with_capacity(RUNS);
    let mut g2_mixed_times = Vec::with_capacity(RUNS);

    for run in 1..=RUNS {
        let g1_add = g1_add_only(&g1_bases, &indices, bucket_count);
        let g2_add = g2_add_only(&g2_bases, &indices, bucket_count);

        let g1_sub = g1_sub_only(&g1_bases, &indices, bucket_count);
        let g2_sub = g2_sub_only(&g2_bases, &indices, bucket_count);

        let g1_mixed = g1_mixed(&g1_bases, &indices, &signs, bucket_count);
        let g2_mixed = g2_mixed(&g2_bases, &indices, &signs, bucket_count);

        g1_add_times.push(g1_add);
        g2_add_times.push(g2_add);
        g1_sub_times.push(g1_sub);
        g2_sub_times.push(g2_sub);
        g1_mixed_times.push(g1_mixed);
        g2_mixed_times.push(g2_mixed);

        for (group, operation, time) in [
            ("G1", "add", g1_add),
            ("G2", "add", g2_add),
            ("G1", "sub", g1_sub),
            ("G2", "sub", g2_sub),
            ("G1", "mixed", g1_mixed),
            ("G2", "mixed", g2_mixed),
        ] {
            write_result(
                &mut file,
                session,
                bucket_count,
                group,
                operation,
                run,
                time,
            )
            .expect("failed to write group-add result");
        }

        file.flush().expect("failed to flush group-add CSV");

        println!(
            "run={run} \
             add_g1={g1_add:.3}ms add_g2={g2_add:.3}ms \
             sub_g1={g1_sub:.3}ms sub_g2={g2_sub:.3}ms \
             mixed_g1={g1_mixed:.3}ms mixed_g2={g2_mixed:.3}ms"
        );
    }

    let g1_add = median(&mut g1_add_times);
    let g2_add = median(&mut g2_add_times);
    let g1_sub = median(&mut g1_sub_times);
    let g2_sub = median(&mut g2_sub_times);
    let g1_mixed = median(&mut g1_mixed_times);
    let g2_mixed = median(&mut g2_mixed_times);

    println!();
    println!("=== Median Results (all 7 formal runs) ===");
    println!();

    println!(
        "ADD   : G1={g1_add:.3} ms, G2={g2_add:.3} ms, G2/G1={:.3}x",
        g2_add / g1_add
    );

    println!(
        "SUB   : G1={g1_sub:.3} ms, G2={g2_sub:.3} ms, G2/G1={:.3}x",
        g2_sub / g1_sub
    );

    println!(
        "MIXED : G1={g1_mixed:.3} ms, G2={g2_mixed:.3} ms, G2/G1={:.3}x",
        g2_mixed / g1_mixed
    );

    println!();
    println!(
        "Average time per update (mixed): \
         G1={:.3} ns, G2={:.3} ns",
        g1_mixed * 1_000_000.0 / N_UPDATES as f64,
        g2_mixed * 1_000_000.0 / N_UPDATES as f64,
    );
}
