use crate::{
    field::{Field, FieldElement},
    polynomials::Polynomial,
};
use num_bigint::BigUint;

use blstrs::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use group::{Group, ff::Field as FieldT};

pub trait PolynomialCommitment {
    fn setup(
        &self,
        // tau is a secret value, ideally computed trustlessly, and must be forgotten
        tau: FieldElement,
        // This is something like "max degree"
        d: usize,
    ) -> (Vec<G1Projective>, Vec<G2Projective>);
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
        d: usize,
    ) -> (Vec<G1Projective>, Vec<G2Projective>) {
        let mut gs = vec![G1Projective::generator(); d];
        let mut hs = vec![G2Projective::generator(); d];

        // Modulus of the base field used in the bls12_381 scheme
        // From https://github.com/zkcrypto/bls12_381/blob/7de7b9d9c509b9973b35a3241b74bbbea95e700a/src/fp.rs#L70
        let p = {
            let large_integer_str = "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787";
            BigUint::parse_bytes(large_integer_str.as_bytes(), 10).unwrap()
        };

        let tau = Scalar::random(rand::thread_rng());

        let new_gs = gs.iter().map(|g| g * tau).collect();
        let new_hs = hs.iter().map(|h| h * tau).collect();

        (new_gs, new_hs)
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
fn setup() {
    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field);
}

#[test]
fn commits() {
    let polynomial = Polynomial::new(&vec![1, 2, 3]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let t = FieldElement::new(BigUint::from(20_u32), field.clone());

    let max_degree = 25;
    let generator = FieldElement::new(BigUint::from(1_u32), field.clone());

    let global_parameters = polynomial_committer.setup(t, max_degree);

    // let commitment = polynomial_committer.commit(polynomial, &global_parameters);

    // assert_eq!(commitment, FieldElement::new(BigUint::from(5_u32), field));
}
