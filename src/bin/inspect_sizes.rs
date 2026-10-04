#[path = "../circuit.rs"]
mod circuit;

#[path = "../groth16.rs"]
mod groth16;

use ark_bn254::Fr;
use ark_std::test_rng;

use circuit::RepeatedSquareCircuit;
use groth16::setup;

fn inspect(n: usize) {
    println!("================================");
    println!("Inspecting circuit size: {n}");

    let circuit = RepeatedSquareCircuit::<Fr> {
        num_constraints: n,
        x: None,
        intermediate: None,
        output: None,
    };

    let mut rng = test_rng();

    let pk = setup(circuit, &mut rng).expect("Groth16 setup failed");

    println!();
    println!("Proving-key query lengths:");

    println!("a_query.len()    = {}", pk.a_query.len());

    println!("b_g1_query.len() = {}", pk.b_g1_query.len());

    println!("b_g2_query.len() = {}", pk.b_g2_query.len());

    println!("h_query.len()    = {}", pk.h_query.len());

    println!("l_query.len()    = {}", pk.l_query.len());

    println!();

    println!(
        "A/B MSM length = query.len() - 1 = {}",
        pk.a_query.len() - 1
    );
}

fn main() {
    for n in [1_000, 10_000, 100_000, 1_000_000] {
        inspect(n);
    }
}
