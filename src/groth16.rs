use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, prepare_verifying_key};
use ark_relations::gr1cs::SynthesisError;
use ark_std::rand::Rng;

use crate::circuit::SquarePlusOne;

pub fn setup<R: Rng>(
    rng: &mut R,
) -> Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>), SynthesisError> {
    // Setup only needs the circuit structure.
    // The actual witness assignment is not needed here.
    let circuit = SquarePlusOne::<Fr> { x: None, y: None };

    let pk = Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, rng)?;

    let vk = pk.vk.clone();

    Ok((pk, vk))
}

pub fn prove<R: Rng>(pk: &ProvingKey<Bn254>, rng: &mut R) -> Result<Proof<Bn254>, SynthesisError> {
    // Proving requires the actual witness.
    let circuit = SquarePlusOne::<Fr> {
        x: Some(Fr::from(2u64)),
        y: Some(Fr::from(5u64)),
    };

    Groth16::<Bn254>::create_random_proof_with_reduction(circuit, pk, rng)
}

pub fn verify(vk: &VerifyingKey<Bn254>, proof: &Proof<Bn254>) -> Result<bool, SynthesisError> {
    let pvk = prepare_verifying_key(vk);

    let public_inputs = [Fr::from(5u64)];

    Groth16::<Bn254>::verify_proof(&pvk, proof, &public_inputs)
}
