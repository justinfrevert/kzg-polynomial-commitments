use crate::{
    field::{Field, FieldElement},
    polynomials::Polynomial,
};
use num_bigint::BigUint;

pub trait PolynomialCommitment {
    fn setup(
        &self,
        // tau is a secret value, ideally computed trustlessly, and must be forgotten
        tau: FieldElement,
        // This is something like "max degree"
        d: u32,
        generator: FieldElement,
    ) -> Vec<FieldElement>;
    /// Should be $f(\tau) \cdot G \in \mathbb G$
    fn commit(&self, polynomial: Polynomial, global_parameters: &[FieldElement]) -> FieldElement;
    fn open();
    fn verify();
    fn create_witness();
    fn verify_evaluation();
}

pub struct GenericPolynomialCommitment {
    // Its g_1 field
    g1: Field,
}

impl GenericPolynomialCommitment {
    pub fn new(g1: Field) -> Self {
        GenericPolynomialCommitment { g1 }
    }
}

impl PolynomialCommitment for GenericPolynomialCommitment {
    fn setup(
        &self,
        tau: FieldElement,
        // g1: Field,
        // This is something like "max degree"
        d: u32,
        generator: FieldElement,
    ) -> Vec<FieldElement> {
        // Tau cannot be larger than field
        assert!(tau.value < self.g1.0);

        // let h_1 = BigUint::from(tau.value.clone() * generator.clone());
        let h_1 = tau.clone() * generator.clone();
        let mut global_parameters = vec![h_1];
        // We have to continue this above calculation for all of the /h values for the rest of /d

        let field = self.g1.clone();
        for i in 2..=d {
            let i_as_field = FieldElement::new(BigUint::from(i), field.clone());
            let h_i = tau.clone().pow(i_as_field) * generator.clone();
            global_parameters.push(h_i);
        }

        global_parameters
    }

    fn commit(&self, polynomial: Polynomial, global_parameters: &[FieldElement]) -> FieldElement {
        let mut result = FieldElement::new(BigUint::from(0_u32), self.g1.clone());

        // For $f_0 .. f_d$ we need to calculate $f_i \times H_i$ where H is the global parameters
        // polynomial.0.iter().zip(global_parameters.iter()).for_each(
        polynomial.0.iter().zip(global_parameters.iter()).for_each(
            |(coefficient, global_parameter)| {
                // Constrained to smaller size through modding
                let coefficient_modded = coefficient % self.g1.clone().0;
                let coefficient = coefficient_modded.try_into().unwrap();

                let coefficient_as_field_element = FieldElement::new(coefficient, self.g1.clone());
                result += coefficient_as_field_element * global_parameter.clone();
            },
        );

        result
    }
    fn open() {}
    fn verify() {}
    fn create_witness() {}
    fn verify_evaluation() {}
}

#[test]
fn commits() {
    let polynomial = Polynomial::new(&vec![1, 2, 3]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let t = FieldElement::new(BigUint::from(20_u32), field.clone());

    let max_degree = 25;
    let generator = FieldElement::new(BigUint::from(1_u32), field.clone());

    let global_parameters = polynomial_committer.setup(t, max_degree, generator);

    let commitment = polynomial_committer.commit(polynomial, &global_parameters);

    assert_eq!(commitment, FieldElement::new(BigUint::from(5_u32), field));
}
