mod circuit;

use ark_bn254::Fr;
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystem};

use circuit::SquarePlusOne;

fn main() {
    let cs = ConstraintSystem::<Fr>::new_ref();

    let circuit = SquarePlusOne {
        x: Some(Fr::from(2u64)),
        y: Some(Fr::from(5u64)),
    };

    circuit
        .generate_constraints(cs.clone())
        .expect("failed to generate constraints");

    println!("=== Groth16 Baseline: Circuit Structure ===");
    println!("Constraints: {}", cs.num_constraints());
    println!("Instance variables: {}", cs.num_instance_variables());
    println!("Witness variables: {}", cs.num_witness_variables());
}
