use std::hint::black_box;
use std::time::Instant;

use ark_bn254::{G1Projective, G2Projective};
use ark_std::{UniformRand, Zero};

const BUCKET_COUNT: usize = 8192;
const REPEATS: usize = 100;
const WARMUPS: usize = 2;
const RUNS: usize = 7;

fn make_g1_buckets() -> Vec<G1Projective> {
    let mut rng = ark_std::test_rng();

    (0..BUCKET_COUNT)
        .map(|_| G1Projective::rand(&mut rng))
        .collect()
}

fn make_g2_buckets() -> Vec<G2Projective> {
    let mut rng = ark_std::test_rng();

    (0..BUCKET_COUNT)
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

        let _=black_box(res);
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

        let _ =black_box(res);
    }

    start.elapsed().as_secs_f64() * 1000.0
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}

fn main() {
    println!("=== Groth16 G1/G2 Prefix Sum Microbenchmark ===");
    println!("bucket_count={BUCKET_COUNT}");
    println!("repeats={REPEATS}");
    println!("warmups={WARMUPS}");
    println!("runs={RUNS}");
    println!();

    println!("Generating bucket inputs...");

    let g1_buckets = make_g1_buckets();
    let g2_buckets = make_g2_buckets();

    println!("Input generation complete.");
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

        println!(
            "run={run} g1={g1:.3}ms g2={g2:.3}ms ratio={:.3}x",
            g2 / g1
        );
    }

    let g1 = median(&mut g1_times);
    let g2 = median(&mut g2_times);

    let total_adds = BUCKET_COUNT * REPEATS * 2;

    println!();
    println!("=== Median Results ===");
    println!();

    println!(
        "G1 prefix : {g1:.3} ms"
    );

    println!(
        "G2 prefix : {g2:.3} ms"
    );

    println!(
        "G2/G1     : {:.3}x",
        g2 / g1
    );

    println!(
        "Total group additions per run: {total_adds}"
    );

    println!(
        "Per addition: G1={:.3} ns, G2={:.3} ns",
        g1 * 1_000_000.0 / total_adds as f64,
        g2 * 1_000_000.0 / total_adds as f64,
    );
}