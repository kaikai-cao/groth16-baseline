mod circuit;
mod groth16;

use groth16::{prove, setup, verify};

fn main() {
    let mut rng = ark_std::test_rng();

    println!("=== Groth16 Correctness Test ===");

    let (pk, vk) = setup(&mut rng).expect("Groth16 setup failed");

    println!("Setup: OK");

    let proof = prove(&pk, &mut rng).expect("Groth16 proving failed");

    println!("Prove: OK");

    let verified = verify(&vk, &proof).expect("Groth16 verification failed");

    println!("Verify: {verified}");

    assert!(verified, "Groth16 proof verification failed");
}
