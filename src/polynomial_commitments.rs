use std::marker::PhantomData;

use crate::{
    field::{Field, FieldElement},
    polynomials::Polynomial,
};
use num_bigint::BigUint;

use blstrs::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
// use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use group::{Group, ff::Field as FieldT, prime::PrimeCurveAffine};

pub struct GlobalParameters {
    pub gs: Vec<G1Projective>,
    hs: Vec<G2Projective>
}

impl GlobalParameters {
    fn new(gs: Vec<G1Projective>, hs: Vec<G2Projective>) -> Self {
        GlobalParameters { gs, hs }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    IncorrectDegree
}

pub trait PolynomialCommitment {
    fn setup(
        &self,
        // This is something like "max degree"
        d: usize,
    ) -> GlobalParameters;
    /// Should be $f(\tau) \cdot G \in \mathbb G$
    fn commit(&self, polynomial: &Polynomial, global_parameters: &GlobalParameters) -> Result<G1Projective, Error>;
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
        let gs = vec![G1Projective::generator(); d];
        let hs = vec![G2Projective::generator(); d];

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

    fn commit(&self, polynomial: &Polynomial, global_parameters: &GlobalParameters) -> Result<G1Projective, Error> {
        if polynomial.0.len() != global_parameters.gs.len() {
            return Err(Error::IncorrectDegree)
        }
        // For $f_0 .. f_d$ we need to calculate $f_i \times H_i$ where H is the global parameters. We can just use this to do it in an optimized way
       Ok(G1Projective::multi_exp(&global_parameters.gs, &polynomial.0))
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

    let gp = polynomial_committer.setup(5);
}

#[test]
fn commits() {
    let polynomial = Polynomial::new_from_bytes(&[1; 25]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let max_degree = 25;
    let generator = FieldElement::new(BigUint::from(1_u32), field.clone());

    let global_parameters = polynomial_committer.setup(max_degree);

    let commitment = polynomial_committer.commit(&polynomial, &global_parameters);

    // assert_eq!(commitment, G1Projective::generator());
}

#[test]
fn errs_on_incorrect_polynomial_degree() {
    let small_polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let large_polynomial = Polynomial::new_from_bytes(&[1; 420]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let max_degree = 25;
    let global_parameters = polynomial_committer.setup(max_degree);

    let too_small_commitment = polynomial_committer.commit(&small_polynomial, &global_parameters);
    let too_large_commitment = polynomial_committer.commit(&large_polynomial, &global_parameters);
    assert_eq!(too_small_commitment, Err(Error::IncorrectDegree));
    assert_eq!(too_large_commitment, Err(Error::IncorrectDegree));
}

#[test]
fn adjust() {
    let mut small_polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let mut large_polynomial = Polynomial::new_from_bytes(&[1; 420]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let max_degree = 25;
    let global_parameters = polynomial_committer.setup(max_degree);


    let too_small_polynomial_then_adjusted = small_polynomial.adjust_to_degree(max_degree);
    let too_large_polynomial_then_adjusted = large_polynomial.adjust_to_degree(max_degree);

    println!("too large polynomial adjusted was {:?}", too_large_polynomial_then_adjusted.0.len());

    let too_small_commitment = polynomial_committer.commit(too_small_polynomial_then_adjusted, &global_parameters);
    let too_large_commitment = polynomial_committer.commit(too_large_polynomial_then_adjusted, &global_parameters);

    assert!(too_small_commitment.is_ok());
    assert!(too_large_commitment.is_ok());
}


#[test]
fn adjusts_polynomial_of_different_size_to_correct_degree() {
    let mut polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);

    let field = Field(BigUint::from(41_u32));
    let polynomial_committer = GenericPolynomialCommitment::new(field.clone());

    let max_degree = 25;
    let generator = FieldElement::new(BigUint::from(1_u32), field.clone());

    let global_parameters = polynomial_committer.setup(max_degree);

    // Get degree of polynomial commitment, and pad accordingly
    let polynomial_padded = polynomial.adjust_to_degree(max_degree);

    println!("polynomial newly padded is {:?} versus expected degree of {:?}", polynomial_padded.0.len(), max_degree);

    let commitment = polynomial_committer.commit(&polynomial, &global_parameters);
    
    assert!(commitment.is_ok());
}
