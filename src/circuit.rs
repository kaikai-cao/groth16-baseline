use ark_ff::Field;
use ark_relations::{
    gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
    lc,
};

pub struct SquarePlusOne<F: Field> {
    pub x: Option<F>,
    pub y: Option<F>,
}

impl<F: Field> ConstraintSynthesizer<F> for SquarePlusOne<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let x = cs.new_witness_variable(|| self.x.ok_or(SynthesisError::AssignmentMissing))?;

        let y = cs.new_input_variable(|| self.y.ok_or(SynthesisError::AssignmentMissing))?;

        cs.enforce_r1cs_constraint(
            || lc!() + x,
            || lc!() + x,
            || lc!() + y - lc!() + ark_relations::gr1cs::Variable::One,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::gr1cs::ConstraintSystem;

    #[test]
    fn test_square_plus_one() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        let circuit = SquarePlusOne {
            x: Some(Fr::from(2u64)),
            y: Some(Fr::from(5u64)),
        };

        circuit.generate_constraints(cs.clone()).unwrap();

        assert_eq!(cs.num_constraints(), 1);
        assert_eq!(cs.num_instance_variables(), 2);
        assert_eq!(cs.num_witness_variables(), 1);
    }
}
