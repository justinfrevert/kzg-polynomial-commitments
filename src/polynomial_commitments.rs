use crate::{
    field::{Field, FieldElement},
    polynomials::Polynomial,
};
use num_bigint::BigUint;

use blstrs::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
// use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use group::{Group, ff::Field as FieldT};

pub struct GlobalParameters {
    pub gs: Vec<G1Projective>,
    hs: Vec<G2Projective>
}

impl GlobalParameters {
    fn new(gs: Vec<G1Projective>, hs: Vec<G2Projective>) -> Self {
        GlobalParameters { gs, hs }
    }
}

pub trait PolynomialCommitment {
    fn setup(
        &self,
        // This is something like "max degree"
        d: usize,
    ) -> GlobalParameters;
    /// Should be $f(\tau) \cdot G \in \mathbb G$
    fn commit(&self, polynomial: Polynomial, global_parameters: &GlobalParameters) -> G1Projective;
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
        // This is something like "max degree"
        d: usize,
    ) -> GlobalParameters {
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

        GlobalParameters::new(new_gs, new_hs)
    }

    fn commit(&self, polynomial: Polynomial, global_parameters: &GlobalParameters) -> G1Projective {
        // For $f_0 .. f_d$ we need to calculate $f_i \times H_i$ where H is the global parameters. We can just use this to do it in an optimized way
        G1Projective::multi_exp(&global_parameters.gs, &polynomial.0)
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
    let polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let max_degree = 25;
    let generator = FieldElement::new(BigUint::from(1_u32), field.clone());

    let global_parameters = polynomial_committer.setup(max_degree);

    // let commitment = polynomial_committer.commit(polynomial, &global_parameters);

    // assert_eq!(commitment, FieldElement::new(BigUint::from(5_u32), field));
}
