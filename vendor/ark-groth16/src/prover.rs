use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use crate::{r1cs_to_qap::R1CSToQAP, Groth16, Proof, ProvingKey, VerifyingKey};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup, VariableBaseMSM};
use ark_ff::{Field, PrimeField, UniformRand, Zero};
use ark_poly::GeneralEvaluationDomain;
use ark_relations::{
    gr1cs::{
        ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, Result as R1CSResult,
        SynthesisMode,
    },
    utils::matrix::Matrix,
};
use ark_std::{
    cfg_into_iter, cfg_iter,
    ops::{AddAssign, Mul},
    rand::Rng,
    vec::Vec,
};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

type D<F> = GeneralEvaluationDomain<F>;

/// Timing information for the five MSM operations inside Groth16 proving.
#[derive(Clone, Copy, Debug, Default)]
pub struct MsmTimings {
    /// Time spent in the C-H MSM, in milliseconds.
    pub c_h_ms: f64,

    /// Time spent in the C-L MSM, in milliseconds.
    pub c_l_ms: f64,

    /// Time spent in the A MSM, in milliseconds.
    pub a_ms: f64,

    /// Time spent in the B-G1 MSM, in milliseconds.
    pub b_g1_ms: f64,

    /// Time spent in the B-G2 MSM, in milliseconds.
    pub b_g2_ms: f64,
}

/// Stores the timing information from the most recent proof generation.
static LAST_MSM_TIMINGS: OnceLock<Mutex<Option<MsmTimings>>> = OnceLock::new();

/// Take the timing information from the most recent proof generation.
///
/// The stored value is consumed by this function.
pub fn take_last_msm_timings() -> Option<MsmTimings> {
    LAST_MSM_TIMINGS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take()
}

impl<E: Pairing, QAP: R1CSToQAP> Groth16<E, QAP> {
    /// Create a Groth16 proof using randomness `r` and `s` and
    /// the provided R1CS-to-QAP reduction, using the provided
    /// R1CS constraint matrices.
    #[inline]
    pub fn create_proof_with_reduction_and_matrices(
        pk: &ProvingKey<E>,
        r: E::ScalarField,
        s: E::ScalarField,
        matrices: &[Matrix<E::ScalarField>],
        num_inputs: usize,
        num_constraints: usize,
        full_assignment: &[E::ScalarField],
    ) -> R1CSResult<Proof<E>> {
        let prover_time = start_timer!(|| "Groth16::Prover");

        let witness_map_time = start_timer!(|| "R1CS to QAP witness map");
        let h = QAP::witness_map_from_matrices::<E::ScalarField, D<E::ScalarField>>(
            matrices,
            num_inputs,
            num_constraints,
            full_assignment,
        )?;
        end_timer!(witness_map_time);

        let input_assignment = &full_assignment[1..num_inputs];
        let aux_assignment = &full_assignment[num_inputs..];

        let proof =
            Self::create_proof_with_assignment(pk, r, s, &h, input_assignment, aux_assignment)?;

        end_timer!(prover_time);

        Ok(proof)
    }

    #[inline]
    fn create_proof_with_assignment(
        pk: &ProvingKey<E>,
        r: E::ScalarField,
        s: E::ScalarField,
        h: &[E::ScalarField],
        input_assignment: &[E::ScalarField],
        aux_assignment: &[E::ScalarField],
    ) -> R1CSResult<Proof<E>> {
        // ============================================================
        // Compute C
        // ============================================================
        let c_acc_time = start_timer!(|| "Compute C");

        // ----------------------------
        // MSM C-H
        // ----------------------------
        let msm_c_h_timer = start_timer!(|| "MSM C-H");
        let msm_c_h_start = Instant::now();

        let h_assignment = cfg_into_iter!(h)
            .map(|s| s.into_bigint())
            .collect::<Vec<_>>();

        let h_acc =
            E::G1::msm_bigint(&pk.h_query, &h_assignment);

        let msm_c_h_ms = msm_c_h_start.elapsed().as_secs_f64() * 1000.0;
        end_timer!(msm_c_h_timer);

        drop(h_assignment);

        // ----------------------------
        // Prepare auxiliary assignment
        // ----------------------------
        let aux_assignment = cfg_iter!(aux_assignment)
            .map(|s| s.into_bigint())
            .collect::<Vec<_>>();

        // ----------------------------
        // MSM C-L
        // ----------------------------
        let msm_c_l_timer = start_timer!(|| "MSM C-L");
        let msm_c_l_start = Instant::now();

        let l_aux_acc =
            E::G1::msm_bigint(&pk.l_query, &aux_assignment);

        let msm_c_l_ms = msm_c_l_start.elapsed().as_secs_f64() * 1000.0;
        end_timer!(msm_c_l_timer);

        let r_s_delta_g1 = pk.delta_g1 * (r * s);

        end_timer!(c_acc_time);

        // ============================================================
        // Prepare complete assignment
        // ============================================================
        let input_assignment = input_assignment
            .iter()
            .map(|s| s.into_bigint())
            .collect::<Vec<_>>();

        let assignment = [&input_assignment[..], &aux_assignment[..]].concat();

        drop(aux_assignment);

        // ============================================================
        // Compute A
        // ============================================================
        let mut msm_a_ms = 0.0;

        let a_acc_time = start_timer!(|| "Compute A");

        let r_g1 = pk.delta_g1.mul(r);

        let g_a = Self::calculate_coeff(
            r_g1,
            &pk.a_query,
            pk.vk.alpha_g1,
            &assignment,
            "MSM A",
            &mut msm_a_ms,
        );

        let s_g_a = g_a * &s;

        end_timer!(a_acc_time);

        // ============================================================
        // Compute B in G1
        // ============================================================
        let mut msm_b_g1_ms = 0.0;

        let g1_b = if !r.is_zero() {
            let b_g1_acc_time = start_timer!(|| "Compute B in G1");

            let s_g1 = pk.delta_g1.mul(s);

            let g1_b = Self::calculate_coeff(
                s_g1,
                &pk.b_g1_query,
                pk.beta_g1,
                &assignment,
                "MSM B-G1",
                &mut msm_b_g1_ms,
            );

            end_timer!(b_g1_acc_time);

            g1_b
        } else {
            E::G1::zero()
        };

        // ============================================================
        // Compute B in G2
        // ============================================================
        let mut msm_b_g2_ms = 0.0;

        let b_g2_acc_time = start_timer!(|| "Compute B in G2");

        let s_g2 = pk.vk.delta_g2.mul(s);

        let g2_b = Self::calculate_coeff(
            s_g2,
            &pk.b_g2_query,
            pk.vk.beta_g2,
            &assignment,
            "MSM B-G2",
            &mut msm_b_g2_ms,
        );

        let r_g1_b = g1_b * &r;

        drop(assignment);

        end_timer!(b_g2_acc_time);

        // ============================================================
        // Finish C
        // ============================================================
        let c_time = start_timer!(|| "Finish C");

        let mut g_c = s_g_a;
        g_c += &r_g1_b;
        g_c -= &r_s_delta_g1;
        g_c += &l_aux_acc;
        g_c += &h_acc;

        end_timer!(c_time);

        // ============================================================
        // Save MSM timing results
        // ============================================================
        let timings = MsmTimings {
            c_h_ms: msm_c_h_ms,
            c_l_ms: msm_c_l_ms,
            a_ms: msm_a_ms,
            b_g1_ms: msm_b_g1_ms,
            b_g2_ms: msm_b_g2_ms,
        };

        *LAST_MSM_TIMINGS
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(timings);

        Ok(Proof {
            a: g_a.into_affine(),
            b: g2_b.into_affine(),
            c: g_c.into_affine(),
        })
    }

    /// Create a Groth16 proof that is zero-knowledge using the provided
    /// R1CS-to-QAP reduction.
    /// This method samples randomness for zero knowledges via `rng`.
    #[inline]
    pub fn create_random_proof_with_reduction<C>(
        circuit: C,
        pk: &ProvingKey<E>,
        rng: &mut impl Rng,
    ) -> R1CSResult<Proof<E>>
    where
        C: ConstraintSynthesizer<E::ScalarField>,
    {
        let r = E::ScalarField::rand(rng);
        let s = E::ScalarField::rand(rng);

        Self::create_proof_with_reduction(circuit, pk, r, s)
    }

    /// Create a Groth16 proof that is *not* zero-knowledge with the provided
    /// R1CS-to-QAP reduction.
    #[inline]
    pub fn create_proof_with_reduction_no_zk<C>(
        circuit: C,
        pk: &ProvingKey<E>,
    ) -> R1CSResult<Proof<E>>
    where
        C: ConstraintSynthesizer<E::ScalarField>,
    {
        Self::create_proof_with_reduction(
            circuit,
            pk,
            E::ScalarField::zero(),
            E::ScalarField::zero(),
        )
    }

    /// Create a Groth16 proof using randomness `r` and `s` and the provided
    /// R1CS-to-QAP reduction.
    #[inline]
    pub fn create_proof_with_reduction<C>(
        circuit: C,
        pk: &ProvingKey<E>,
        r: E::ScalarField,
        s: E::ScalarField,
    ) -> R1CSResult<Proof<E>>
    where
        E: Pairing,
        C: ConstraintSynthesizer<E::ScalarField>,
        QAP: R1CSToQAP,
    {
        let prover_time = start_timer!(|| "Groth16::Prover");

        let cs = ConstraintSystem::new_ref();

        // Set the optimization goal
        cs.set_optimization_goal(OptimizationGoal::Constraints);

        cs.set_mode(SynthesisMode::Prove {
            construct_matrices: true,
            generate_lc_assignments: false,
        });

        // ============================================================
        // Constraint synthesis
        // ============================================================
        let synthesis_time = start_timer!(|| "Constraint synthesis");

        circuit.generate_constraints(cs.clone())?;

        end_timer!(synthesis_time);

        // ============================================================
        // Inlining LCs
        // ============================================================
        let lc_time = start_timer!(|| "Inlining LCs");

        cs.finalize();

        end_timer!(lc_time);

        debug_assert!(cs.is_satisfied().unwrap());

        // ============================================================
        // R1CS -> QAP witness map
        // ============================================================
        let witness_map_time = start_timer!(|| "R1CS to QAP witness map");

        let h = QAP::witness_map::<E::ScalarField, D<E::ScalarField>>(cs.clone())?;

        end_timer!(witness_map_time);

        // ============================================================
        // Create proof
        // ============================================================
        let prover = cs.borrow().unwrap();

        let proof = Self::create_proof_with_assignment(
            pk,
            r,
            s,
            &h,
            &prover.instance_assignment().unwrap()[1..],
            &prover.witness_assignment().unwrap(),
        )?;

        end_timer!(prover_time);

        Ok(proof)
    }

    /// Given a Groth16 proof, returns a fresh proof of the same statement. For
    /// a proof π of a statement S, the output of the non-deterministic
    /// procedure `rerandomize_proof(π)` is statistically indistinguishable
    /// from a fresh honest proof of S. For more info, see theorem 3 of [\[BKSV20\]](https://eprint.iacr.org/2020/811)
    pub fn rerandomize_proof(
        vk: &VerifyingKey<E>,
        proof: &Proof<E>,
        rng: &mut impl Rng,
    ) -> Proof<E> {
        // These are our rerandomization factors. They must be nonzero and uniformly
        // sampled.
        let (mut r1, mut r2) = (E::ScalarField::zero(), E::ScalarField::zero());

        while r1.is_zero() || r2.is_zero() {
            r1 = E::ScalarField::rand(rng);
            r2 = E::ScalarField::rand(rng);
        }

        // See figure 1 in the paper referenced above:
        //   A' = (1/r₁)A
        //   B' = r₁B + r₁r₂(δG₂)
        //   C' = C + r₂A

        // We can unwrap() this because r₁ is guaranteed to be nonzero
        let new_a = proof.a.mul(r1.inverse().unwrap());
        let new_b = proof.b.mul(r1) + &vk.delta_g2.mul(r1 * &r2);
        let new_c = proof.c + proof.a.mul(r2).into_affine();

        Proof {
            a: new_a.into_affine(),
            b: new_b.into_affine(),
            c: new_c.into_affine(),
        }
    }

    fn calculate_coeff<G: AffineRepr>(
        initial: G::Group,
        query: &[G],
        vk_param: G,
        assignment: &[<G::ScalarField as PrimeField>::BigInt],
        timer_name: &'static str,
        msm_time_ms: &mut f64,
    ) -> G::Group
    where
        G::Group: VariableBaseMSM<MulBase = G>,
    {
        let el = query[0];

        // ============================================================
        // MSM
        // ============================================================
        let msm_timer = start_timer!(|| timer_name);
        let msm_start = Instant::now();

        let acc =
            G::Group::msm_bigint(&query[1..], assignment);

        *msm_time_ms = msm_start.elapsed().as_secs_f64() * 1000.0;

        end_timer!(msm_timer);

        // ============================================================
        // Remaining group operations
        // ============================================================
        let mut res = initial;

        res.add_assign(&el);
        res += &acc;
        res.add_assign(&vk_param);

        res
    }
}