use std::time::Instant;

use ark_bn254::Fr;
use ark_serialize::CanonicalSerialize;
use ark_std::test_rng;

use crate::circuit::{RepeatedSquareCircuit, generate_repeated_square_witness};
use crate::groth16::{prepare_vk, prove, setup, verify};

pub struct BenchmarkResult {
    pub constraints: usize,
    pub setup_ms: f64,
    pub witness_ms: f64,
    pub prepare_vk_ms: f64,
    pub prove_ms: f64,
    pub verify_ms: f64,
    pub proof_bytes: usize,
}

pub fn run_once(num_constraints: usize) -> Result<BenchmarkResult, String> {
    let setup_circuit = RepeatedSquareCircuit::<Fr> {
        num_constraints,
        x: None,
        intermediate: None,
        output: None,
    };

    let mut setup_rng = test_rng();

    let start = Instant::now();

    let pk = setup(setup_circuit, &mut setup_rng).map_err(|e| format!("setup failed: {e:?}"))?;

    let setup_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();

    let pvk = prepare_vk(&pk.vk);

    let prepare_vk_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();

    let x = Fr::from(2u64);

    let (intermediate, output) = generate_repeated_square_witness(x, num_constraints);

    let witness_ms = start.elapsed().as_secs_f64() * 1000.0;

    let prove_circuit = RepeatedSquareCircuit::<Fr> {
        num_constraints,
        x: Some(x),
        intermediate: Some(intermediate),
        output: Some(output),
    };

    let mut prove_rng = test_rng();

    let start = Instant::now();

    let proof =
        prove(prove_circuit, &pk, &mut prove_rng).map_err(|e| format!("prove failed: {e:?}"))?;

    let prove_ms = start.elapsed().as_secs_f64() * 1000.0;

    let public_inputs = [output];

    let start = Instant::now();

    let verified =
        verify(&pvk, &proof, &public_inputs).map_err(|e| format!("verify failed: {e:?}"))?;

    let verify_ms = start.elapsed().as_secs_f64() * 1000.0;

    if !verified {
        return Err("verification returned false".to_string());
    }

    let mut bytes = Vec::new();

    proof
        .serialize_compressed(&mut bytes)
        .map_err(|e| format!("proof serialization failed: {e:?}"))?;

    Ok(BenchmarkResult {
        constraints: num_constraints,
        setup_ms,
        witness_ms,
        prepare_vk_ms,
        prove_ms,
        verify_ms,
        proof_bytes: bytes.len(),
    })
}
