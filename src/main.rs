mod benchmark;

use std::fs::OpenOptions;
use std::io::Write;

mod circuit;
mod groth16;

use ark_bn254::Fr;

use circuit::SquarePlusOne;
use groth16::{prepare_vk, prove, setup, verify};

use benchmark::{run_once, run_prove_only};

fn run_correctness() {
    let mut rng = ark_std::test_rng();

    println!("=== Groth16 Correctness Test ===");

    let circuit = SquarePlusOne::<Fr> {
        x: Some(Fr::from(2u64)),
        y: Some(Fr::from(5u64)),
    };

    let (pk, vk) = {
        let setup_circuit = SquarePlusOne::<Fr> { x: None, y: None };

        let pk = setup(setup_circuit, &mut rng).expect("Groth16 setup failed");

        let vk = pk.vk.clone();

        (pk, vk)
    };

    println!("Setup: OK");

    let proof = prove(circuit, &pk, &mut rng).expect("Groth16 proving failed");

    println!("Prove: OK");

    let pvk = prepare_vk(&vk);

    let verified = verify(&pvk, &proof, &[Fr::from(5u64)]).expect("Groth16 verification failed");

    println!("Verify: {verified}");

    assert!(verified);
}

fn run_benchmark(num_constraints: usize, warmups: usize, runs: usize) {
    let path = "experiments/raw/msm_trace/msm_breakdown.csv";

    std::fs::create_dir_all("experiments/raw/msm_trace")
        .expect("failed to create MSM trace directory");

    let file_exists_and_nonempty = std::fs::metadata(path)
        .map(|m| m.len() > 0)
        .unwrap_or(false);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("failed to open MSM breakdown CSV");

    if !file_exists_and_nonempty {
        writeln!(
            file,
            "N,threads,run,setup_ms,witness_ms,prepare_vk_ms,\
verify_ms,proof_bytes,msm_c_h_ms,msm_c_l_ms,msm_a_ms,msm_b_g1_ms,\
msm_b_g2_ms,msm_total_ms"
        )
        .expect("failed to write CSV header");
    }

    let threads = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".to_string());

    println!(
        "N,threads,run,setup_ms,witness_ms,prepare_vk_ms,prove_ms,\
verify_ms,proof_bytes,msm_c_h_ms,msm_c_l_ms,msm_a_ms,msm_b_g1_ms,\
msm_b_g2_ms,msm_total_ms"
    );

    // Warm-up runs are excluded from the formal dataset.
    for _ in 0..warmups {
        run_once(num_constraints).expect("warm-up benchmark failed");
    }

    // Formal runs.
    for run in 1..=runs {
        let result = run_once(num_constraints).expect("benchmark failed");

        let msm_total_ms = result.msm_c_h_ms
            + result.msm_c_l_ms
            + result.msm_a_ms
            + result.msm_b_g1_ms
            + result.msm_b_g2_ms;

        let line = format!(
            "{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            result.constraints,
            threads,
            run,
            result.setup_ms,
            result.witness_ms,
            result.prepare_vk_ms,
            result.prove_ms,
            result.verify_ms,
            result.proof_bytes,
            result.msm_c_h_ms,
            result.msm_c_l_ms,
            result.msm_a_ms,
            result.msm_b_g1_ms,
            result.msm_b_g2_ms,
            msm_total_ms,
        );

        println!("{line}");

        writeln!(file, "{line}").expect("failed to write MSM benchmark result");
    }
}

fn run_prove_scaling(num_constraints: usize, warmups: usize, runs: usize) {
    let path = "experiments/raw/thread_scaling/prove_scaling.csv";

    std::fs::create_dir_all("experiments/raw/thread_scaling")
        .expect("failed to create thread scaling directory");

    let file_exists_and_nonempty = std::fs::metadata(path)
        .map(|m| m.len() > 0)
        .unwrap_or(false);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("failed to open thread scaling CSV");

    if !file_exists_and_nonempty {
        writeln!(
            file,
            "N,threads,run,setup_ms,witness_ms,prepare_vk_ms,\
prove_ms,verify_ms,proof_bytes,msm_c_h_ms,msm_c_l_ms,\
msm_a_ms,msm_b_g1_ms,msm_b_g2_ms,msm_total_ms"
        )
        .expect("failed to write CSV header");
    }

    let threads = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".to_string());

    println!(
        "N,threads,run,setup_ms,witness_ms,prepare_vk_ms,\
prove_ms,verify_ms,proof_bytes,msm_c_h_ms,msm_c_l_ms,\
msm_a_ms,msm_b_g1_ms,msm_b_g2_ms,msm_total_ms"
    );

    let results =
        run_prove_only(num_constraints, warmups, runs).expect("prove-only benchmark failed");

    for (i, result) in results.iter().enumerate() {
        let msm_total_ms = result.msm_c_h_ms
            + result.msm_c_l_ms
            + result.msm_a_ms
            + result.msm_b_g1_ms
            + result.msm_b_g2_ms;

        let line = format!(
            "{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            result.constraints,
            threads,
            i + 1,
            result.setup_ms,
            result.witness_ms,
            result.prepare_vk_ms,
            result.prove_ms,
            result.verify_ms,
            result.proof_bytes,
            result.msm_c_h_ms,
            result.msm_c_l_ms,
            result.msm_a_ms,
            result.msm_b_g1_ms,
            result.msm_b_g2_ms,
            msm_total_ms,
        );

        println!("{line}");

        writeln!(file, "{line}").expect("failed to write thread scaling result");
    }
}

fn ensure_clean_benchmark_environment() {
    let forbidden = ["MSM_TRACE", "MSM_FORCE_WINDOW"];

    for name in forbidden {
        if std::env::var_os(name).is_some() {
            panic!(
                "Benchmark environment is not clean: {} is set. \
                 Unset it before running formal benchmarks.",
                name
            );
        }
    }
}

fn main() {
    ensure_clean_benchmark_environment();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("bench") => {
            let n = args
                .get(2)
                .expect("missing N")
                .parse::<usize>()
                .expect("invalid N");

            let warmups = args
                .get(3)
                .expect("missing warmups")
                .parse::<usize>()
                .expect("invalid warmups");

            let runs = args
                .get(4)
                .expect("missing runs")
                .parse::<usize>()
                .expect("invalid runs");

            run_benchmark(n, warmups, runs);
        }

        Some("prove-bench") => {
            let n = args
                .get(2)
                .expect("missing N")
                .parse::<usize>()
                .expect("invalid N");

            let warmups = args
                .get(3)
                .expect("missing warmups")
                .parse::<usize>()
                .expect("invalid warmups");

            let runs = args
                .get(4)
                .expect("missing runs")
                .parse::<usize>()
                .expect("invalid runs");

            run_prove_scaling(n, warmups, runs);
        }

        _ => run_correctness(),
    }
}
