use ark_ff::Field;
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_relations::lc;

/// Minimal correctness-test circuit:
/// x^2 + 1 = y
pub struct SquarePlusOne<F: Field> {
    pub x: Option<F>,
    pub y: Option<F>,
}

impl<F: Field> ConstraintSynthesizer<F> for SquarePlusOne<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let x = cs.new_witness_variable(|| self.x.ok_or(SynthesisError::AssignmentMissing))?;

        let y = cs.new_input_variable(|| self.y.ok_or(SynthesisError::AssignmentMissing))?;

        // x * x = y - 1
        cs.enforce_r1cs_constraint(
            || lc!() + x,
            || lc!() + x,
            || lc!() + y - ark_relations::gr1cs::Variable::One,
        )?;

        Ok(())
    }
}

/// Scaling circuit:
///
/// z_1 = x^2
/// z_2 = z_1^2
/// ...
/// z_N = z_{N-1}^2
///
/// The final output z_N is public.
pub struct RepeatedSquareCircuit<F: Field> {
    pub num_constraints: usize,
    pub x: Option<F>,
    pub intermediate: Option<Vec<F>>,
    pub output: Option<F>,
}

impl<F: Field> ConstraintSynthesizer<F> for RepeatedSquareCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        assert!(self.num_constraints > 0);

        let mut current_var =
            cs.new_witness_variable(|| self.x.ok_or(SynthesisError::AssignmentMissing))?;

        for i in 0..self.num_constraints {
            let next_var = if i + 1 == self.num_constraints {
                cs.new_input_variable(|| self.output.ok_or(SynthesisError::AssignmentMissing))?
            } else {
                let value = self
                    .intermediate
                    .as_ref()
                    .and_then(|values| values.get(i))
                    .cloned();

                cs.new_witness_variable(|| value.ok_or(SynthesisError::AssignmentMissing))?
            };

            // current^2 = next
            cs.enforce_r1cs_constraint(
                || lc!() + current_var,
                || lc!() + current_var,
                || lc!() + next_var,
            )?;

            current_var = next_var;
        }

        Ok(())
    }
}

/// Generate the witness values required by RepeatedSquareCircuit.
///
/// Returns:
/// - intermediate values z_1 ... z_{N-1}
/// - final public output z_N
pub fn generate_repeated_square_witness<F: Field>(x: F, num_constraints: usize) -> (Vec<F>, F) {
    assert!(num_constraints > 0);

    let mut current = x;
    let mut intermediate = Vec::with_capacity(num_constraints - 1);

    for i in 0..num_constraints {
        let next = current.square();

        if i + 1 == num_constraints {
            return (intermediate, next);
        }

        intermediate.push(next);
        current = next;
    }

    unreachable!()
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

        assert!(cs.is_satisfied().unwrap());
    }

    #[test]
    fn test_repeated_square_structure() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        let num_constraints = 5;

        let circuit = RepeatedSquareCircuit {
            num_constraints,
            x: Some(Fr::from(2u64)),
            intermediate: Some(vec![
                Fr::from(4u64),
                Fr::from(16u64),
                Fr::from(256u64),
                Fr::from(65536u64),
            ]),
            output: Some(Fr::from(65536u64).square()),
        };

        circuit.generate_constraints(cs.clone()).unwrap();

        assert_eq!(cs.num_constraints(), num_constraints);

        assert!(cs.is_satisfied().unwrap());
    }
}
