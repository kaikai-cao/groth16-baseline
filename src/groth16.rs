use ark_bn254::{Bn254, Fr};
use ark_groth16::{
    Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
    prepare_verifying_key as ark_prepare_verifying_key,
};
use ark_relations::gr1cs::{ConstraintSynthesizer, SynthesisError};
use ark_std::rand::Rng;

pub fn setup<C: ConstraintSynthesizer<Fr>>(
    circuit: C,
    rng: &mut impl Rng,
) -> Result<ProvingKey<Bn254>, SynthesisError> {
    Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, rng)
}

pub fn prove<C: ConstraintSynthesizer<Fr>>(
    circuit: C,
    pk: &ProvingKey<Bn254>,
    rng: &mut impl Rng,
) -> Result<Proof<Bn254>, SynthesisError> {
    Groth16::<Bn254>::create_random_proof_with_reduction(circuit, pk, rng)
}

pub fn prepare_vk(vk: &VerifyingKey<Bn254>) -> PreparedVerifyingKey<Bn254> {
    ark_prepare_verifying_key(vk)
}

pub fn verify(
    pvk: &PreparedVerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
    public_inputs: &[Fr],
) -> Result<bool, SynthesisError> {
    Groth16::<Bn254>::verify_proof(pvk, proof, public_inputs)
}
