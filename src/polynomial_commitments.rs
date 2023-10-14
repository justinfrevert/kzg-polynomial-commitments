use crate::{field::Field, polynomials::Polynomial};
use group::Curve;
use num_bigint::BigUint;

use blstrs::{G1Affine, G1Projective, G2Projective, Scalar, pairing, G2Affine};
use group::prime::PrimeCurveAffine;
use group::{ff::Field as FieldT, Group};

#[derive(Clone, Debug)]
pub struct GlobalParameters {
    pub gs: Vec<G1Projective>,
    hs: Vec<G2Projective>,
}

impl GlobalParameters {
    fn new(gs: Vec<G1Projective>, hs: Vec<G2Projective>) -> Self {
        GlobalParameters { gs, hs }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    // Tried to use a polynomial of an inappropriate degree
    IncorrectDegree,
    // Setup not complete; tried to use commitment scheme prior to setup
    SetupIncomplete,
}

pub trait PolynomialCommitment {
    fn setup(
        &mut self,
        // This is something like "max degree"
        d: usize,
    ) -> GlobalParameters;
    /// Should be $f(\tau) \cdot G \in \mathbb G$
    fn commit(&self, polynomial: &Polynomial) -> Result<G1Projective, Error>;
    fn open();
    fn verify();
    fn create_witness(&self, polynomial: Polynomial, point: Scalar) -> (G1Projective, Scalar);
    fn verify_evaluation(&self, committed_polynomial: G1Projective, point: Scalar, evaluation: Scalar, witness: G1Projective) -> bool;
}

#[derive(Debug)]
pub struct GenericPolynomialCommitment {
    global_parameters: Option<GlobalParameters>,
}

impl GenericPolynomialCommitment {
    // This might seem useless for now. I am keeping it, as I might want to come back later for more initialization values
    pub fn new() -> Self {
        GenericPolynomialCommitment {
            global_parameters: None,
        }
    }
}

impl PolynomialCommitment for GenericPolynomialCommitment {
    // A trusted setup procedure which can generate global parameters for the application 
    fn setup(
        &mut self,
        // This is something like "max degree"
        d: usize,
    ) -> GlobalParameters {
        let gs = vec![G1Projective::generator(); d];
        let hs = vec![G2Projective::generator(); d];

        let tau = Scalar::random(rand::thread_rng());

        let new_gs = gs.iter().map(|g| g * tau).collect();
        let new_hs = hs.iter().map(|h| h * tau).collect();

        let global_parameters = GlobalParameters::new(new_gs, new_hs);
        self.global_parameters = Some(global_parameters.clone());
        global_parameters
    }

    // Generate the commitment to the polynomial
    fn commit(&self, polynomial: &Polynomial) -> Result<G1Projective, Error> {
        if self.global_parameters.is_none() {
            return Err(Error::SetupIncomplete);
        }

        let global_parameters = &self.global_parameters.as_ref().unwrap();
        if polynomial.0.len() != global_parameters.gs.len() {
            return Err(Error::IncorrectDegree);
        }
        // For $f_0 .. f_d$ we need to calculate $f_i \times H_i$ where H is the global parameters. We can just use this to do it in an optimized way
        Ok(G1Projective::multi_exp(
            &global_parameters.gs,
            &polynomial.0,
        ))
    }
    fn open() {}
    fn verify() {}
    // Create the witness and evaluation used for later verifying the evaluation
    // φ(x)−φ(i) / (x−i)
    fn create_witness(&self, polynomial: Polynomial, point: Scalar) -> (G1Projective, Scalar) {
        // The evaulation: φ(i). TODO: Does it need to be mod p?
        let evaluation = polynomial.evaluate(point);

        // Dividend φ(x)−φ(i). We retain the highest degree coefficients(φ(x)) and get −φ(i) by subtracting it by the lowest degree coefficient
        let mut dividend = polynomial.clone();
        dividend.0[0] -= &evaluation;
        let divisor = Polynomial::new(&[-point, Scalar::ONE]);

        let mut witness_polynomial = dividend / divisor;

        witness_polynomial.adjust_to_degree(self.global_parameters.as_ref().unwrap().gs.len());

        // The witness desired is a commitment to the witness polynomial
        let witness = self.commit(&witness_polynomial).unwrap();

        (witness, evaluation)
    }

    // Determine if the hidden polynomial evaluated at the point did produce the evaluation based on the witness
    // a(poly) b(point) c(evaluation)
    // $e(\frac {C}{g^{\phi(i)}}, {g}) = e(w_i, \frac{g^\alpha}{g^i})$
    fn verify_evaluation(
        &self,
        committed_polynomial: G1Projective,
        point: Scalar,
        evaluation: Scalar,
        witness: G1Projective,
    ) -> bool {
        let g1 = G1Projective::generator();
        let g2 = G2Projective::generator();

        let inner = committed_polynomial - g1 * evaluation;

        let lhs = pairing(&inner.to_affine(), &g2.to_affine());

        let inner = g2 - &g2 * point;
        let rhs = pairing(&witness.to_affine(), &inner.to_affine());

        lhs == rhs
    }

}

#[test]
fn setup() {
    let mut polynomial_committer = GenericPolynomialCommitment::new();
    let gp = polynomial_committer.setup(5);
}

#[test]
fn errs_on_incorrect_polynomial_degree() {
    let small_polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let large_polynomial = Polynomial::new_from_bytes(&[1; 420]);

    let mut polynomial_committer = GenericPolynomialCommitment::new();

    let max_degree = 25;
    polynomial_committer.setup(max_degree);

    let too_small_commitment = polynomial_committer.commit(&small_polynomial);
    let too_large_commitment = polynomial_committer.commit(&large_polynomial);
    assert_eq!(too_small_commitment, Err(Error::IncorrectDegree));
    assert_eq!(too_large_commitment, Err(Error::IncorrectDegree));
}

#[test]
fn adjusts_polynomial_of_different_size_to_correct_degree() {
    let mut small_polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let mut large_polynomial = Polynomial::new_from_bytes(&[1; 420]);

    let field = Field(BigUint::from(41_u32));
    let mut polynomial_committer = GenericPolynomialCommitment::new();

    let max_degree = 25;
    let global_parameters = polynomial_committer.setup(max_degree);

    let too_small_polynomial_then_adjusted = small_polynomial.adjust_to_degree(max_degree);
    let too_large_polynomial_then_adjusted = large_polynomial.adjust_to_degree(max_degree);

    let too_small_commitment = polynomial_committer.commit(too_small_polynomial_then_adjusted);
    let too_large_commitment = polynomial_committer.commit(too_large_polynomial_then_adjusted);

    assert!(too_small_commitment.is_ok());
    assert!(too_large_commitment.is_ok());
}

#[test]
fn polynomial_commitment() {
    use crate::*;

    let mut polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let mut polynomial_committer = GenericPolynomialCommitment::new();
    let max_degree = 25;

    polynomial_committer.setup(max_degree);

    // Get degree of polynomial commitment, and pad accordingly
    polynomial.adjust_to_degree(max_degree);

    let commitment = polynomial_committer.commit(&polynomial);

    assert!(commitment.is_ok());
}

#[test]
fn creates_witness_polynomial() {
    env_logger::init();

    let mut polynomial_committer = GenericPolynomialCommitment::new();
    polynomial_committer.setup(3);

    let polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let point = Scalar::from(5);

    let commitment = polynomial_committer.commit(&polynomial);

    let (witness, evaluation) = polynomial_committer.create_witness(polynomial, point);

    let result = polynomial_committer.verify_evaluation(commitment.unwrap(), point, evaluation, witness);
    println!("Verification Result: {}", result);
    assert!(result);
}