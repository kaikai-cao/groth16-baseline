mod benchmark;
mod circuit;
mod groth16;

use ark_bn254::Fr;

use benchmark::run_once;
use circuit::SquarePlusOne;
use groth16::{prepare_vk, prove, setup, verify};

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

    let prove_circuit = circuit;

    let proof = prove(prove_circuit, &pk, &mut rng).expect("Groth16 proving failed");

    println!("Prove: OK");

    let pvk = prepare_vk(&vk);

    let verified = verify(&pvk, &proof, &[Fr::from(5u64)]).expect("Groth16 verification failed");

    println!("Verify: {verified}");

    assert!(verified);
}

fn print_benchmark(result: &benchmark::BenchmarkResult, run: usize) {
    println!(
        "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{}",
        result.constraints,
        run,
        result.setup_ms,
        result.witness_ms,
        result.prepare_vk_ms,
        result.prove_ms,
        result.verify_ms,
        result.proof_bytes
    );
}

fn run_benchmark(num_constraints: usize, warmups: usize, runs: usize) {
    println!("N,run,setup_ms,witness_ms,prepare_vk_ms,prove_ms,verify_ms,proof_bytes");

    for _ in 0..warmups {
        run_once(num_constraints).expect("warm-up benchmark failed");
    }

    for run in 1..=runs {
        let result = run_once(num_constraints).expect("benchmark failed");

        print_benchmark(&result, run);
    }
}

fn main() {
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

        _ => run_correctness(),
    }
}
