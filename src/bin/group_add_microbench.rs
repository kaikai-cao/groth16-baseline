use std::hint::black_box;
use std::time::Instant;

use ark_bn254::{G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::VariableBaseMSM;
use ark_std::UniformRand;

const BUCKET_COUNT: usize = 8192;
const N_UPDATES: usize = 500_000;
const WARMUPS: usize = 2;
const RUNS: usize = 7;

/// 生成固定的 bucket 访问模式。
///
/// 目的：G1/G2 使用完全一样的 bucket index 和 sign，
/// 唯一变化就是具体群类型。
fn make_pattern() -> (Vec<usize>, Vec<bool>) {
    let mut indices = Vec::with_capacity(N_UPDATES);
    let mut signs = Vec::with_capacity(N_UPDATES);

    let mut state: u64 = 0x1234_5678_9abc_def0;

    for _ in 0..N_UPDATES {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);

        let index = (state as usize) % BUCKET_COUNT;
        let positive = (state >> 63) == 0;

        indices.push(index);
        signs.push(positive);
    }

    (indices, signs)
}

fn make_g1_bases() -> Vec<G1Affine> {
    let mut rng = ark_std::test_rng();

    (0..N_UPDATES)
        .map(|_| G1Affine::rand(&mut rng))
        .collect()
}

fn make_g2_bases() -> Vec<G2Affine> {
    let mut rng = ark_std::test_rng();

    (0..N_UPDATES)
        .map(|_| G2Affine::rand(&mut rng))
        .collect()
}

/// 只执行 bucket += base
fn g1_add_only(
    bases: &[G1Affine],
    indices: &[usize],
) -> f64 {
    let mut buckets =
        vec![<G1Projective as VariableBaseMSM>::ZERO_BUCKET; BUCKET_COUNT];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] += &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// 只执行 bucket -= base
fn g1_sub_only(
    bases: &[G1Affine],
    indices: &[usize],
) -> f64 {
    let mut buckets =
        vec![<G1Projective as VariableBaseMSM>::ZERO_BUCKET; BUCKET_COUNT];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] -= &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// 按照固定 sign 模式混合执行 += / -=
fn g1_mixed(
    bases: &[G1Affine],
    indices: &[usize],
    signs: &[bool],
) -> f64 {
    let mut buckets =
        vec![<G1Projective as VariableBaseMSM>::ZERO_BUCKET; BUCKET_COUNT];

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

/// G2 += base
fn g2_add_only(
    bases: &[G2Affine],
    indices: &[usize],
) -> f64 {
    let mut buckets =
        vec![<G2Projective as VariableBaseMSM>::ZERO_BUCKET; BUCKET_COUNT];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] += &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// G2 -= base
fn g2_sub_only(
    bases: &[G2Affine],
    indices: &[usize],
) -> f64 {
    let mut buckets =
        vec![<G2Projective as VariableBaseMSM>::ZERO_BUCKET; BUCKET_COUNT];

    let start = Instant::now();

    for i in 0..N_UPDATES {
        buckets[indices[i]] -= &bases[i];
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    black_box(&buckets);

    elapsed
}

/// 按照固定 sign 模式混合执行 += / -=
fn g2_mixed(
    bases: &[G2Affine],
    indices: &[usize],
    signs: &[bool],
) -> f64 {
    let mut buckets =
        vec![<G2Projective as VariableBaseMSM>::ZERO_BUCKET; BUCKET_COUNT];

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

fn main() {
    println!("=== Groth16 G1/G2 Bucket Operation Microbenchmark ===");
    println!("bucket_count={BUCKET_COUNT}");
    println!("updates={N_UPDATES}");
    println!("warmups={WARMUPS}");
    println!("runs={RUNS}");
    println!();

    println!("Generating benchmark inputs...");

    let (indices, signs) = make_pattern();

    let g1_bases = make_g1_bases();
    let g2_bases = make_g2_bases();

    println!("Input generation complete.");
    println!();

    // ---------------------------------------------------------
    // Warmup
    // ---------------------------------------------------------

    println!("Running warmups...");

    for _ in 0..WARMUPS {
        black_box(g1_add_only(&g1_bases, &indices));
        black_box(g2_add_only(&g2_bases, &indices));

        black_box(g1_sub_only(&g1_bases, &indices));
        black_box(g2_sub_only(&g2_bases, &indices));

        black_box(g1_mixed(&g1_bases, &indices, &signs));
        black_box(g2_mixed(&g2_bases, &indices, &signs));
    }

    // ---------------------------------------------------------
    // Formal runs
    // ---------------------------------------------------------

    let mut g1_add_times = Vec::with_capacity(RUNS);
    let mut g2_add_times = Vec::with_capacity(RUNS);

    let mut g1_sub_times = Vec::with_capacity(RUNS);
    let mut g2_sub_times = Vec::with_capacity(RUNS);

    let mut g1_mixed_times = Vec::with_capacity(RUNS);
    let mut g2_mixed_times = Vec::with_capacity(RUNS);

    for run in 1..=RUNS {
        let g1_add = g1_add_only(&g1_bases, &indices);
        let g2_add = g2_add_only(&g2_bases, &indices);

        let g1_sub = g1_sub_only(&g1_bases, &indices);
        let g2_sub = g2_sub_only(&g2_bases, &indices);

        let g1_mixed = g1_mixed(&g1_bases, &indices, &signs);
        let g2_mixed = g2_mixed(&g2_bases, &indices, &signs);

        g1_add_times.push(g1_add);
        g2_add_times.push(g2_add);

        g1_sub_times.push(g1_sub);
        g2_sub_times.push(g2_sub);

        g1_mixed_times.push(g1_mixed);
        g2_mixed_times.push(g2_mixed);

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
    println!("=== Median Results ===");
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