use crate::polynomials::Polynomial;
use blstrs::{pairing, G1Affine, G1Projective, G2Projective, Scalar};
use group::prime::PrimeCurveAffine;
use group::Curve;
use group::{ff::Field as FieldT, Group};
use rand::Rng;

// Generate global parameters for some group's generator
fn generate_tau_points<T: Group + std::ops::Mul<Scalar, Output = T>>(
    generator: T,
    tau: Scalar,
    length: usize,
) -> Vec<T> {
    let mut generators = Vec::with_capacity(length);
    generators.push(generator);
    let mut generator = generator.clone();

    for _ in 1..length {
        generator = generator * tau;
        generators.push(generator);
    }
    generators
}

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
    fn create_witness(&self, polynomial: Polynomial, point: Scalar) -> (G1Projective, Scalar);
    fn verify_evaluation(
        &self,
        committed_polynomial: G1Projective,
        point: Scalar,
        evaluation: Scalar,
        witness: G1Projective,
    ) -> bool;
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
        let mut rng = rand::thread_rng();
        let tau: u64 = rng.gen();
        let tau = Scalar::from(tau);

        let gs = generate_tau_points(G1Projective::generator(), tau, d);
        let hs = generate_tau_points(G2Projective::generator(), tau, d);

        let global_parameters = GlobalParameters::new(gs, hs);
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

    // Create the witness and evaluation used for later verifying the evaluation
    // φ(x)−φ(i) / (x−i)
    fn create_witness(&self, polynomial: Polynomial, point: Scalar) -> (G1Projective, Scalar) {
        // The evaulation: φ(i). TODO: Does it need to be mod p?
        let evaluation = polynomial.evaluate(point);
        // Dividend φ(x)−φ(i). We retain the highest degree coefficients(φ(x)) and get −φ(i) by subtracting it by the lowest degree coefficient
        let mut witness_polynomial = polynomial.clone();
        witness_polynomial.0[0] -= &evaluation;
        let divisor = Polynomial::new(&[-point, Scalar::ONE]);
        witness_polynomial = witness_polynomial / divisor;

        // A small commit to this new polynomial where we care less about the length
        let witness = G1Projective::multi_exp(
            &self.global_parameters.as_ref().unwrap().gs[..witness_polynomial.0.len()],
            &witness_polynomial.0,
        );

        (witness, evaluation)
    }

    // Determine if the hidden polynomial evaluated at the point did produce the evaluation based on the witness
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
        let evaluation_inverse = g1 * -evaluation;

        // $\frac {C}{g^{\phi(i)}}$
        let left_pairing = committed_polynomial + evaluation_inverse;
        let lhs = pairing(&left_pairing.to_affine(), &g2.to_affine());

        let point_commitment_inverted = g2 * -point;

        // $\frac{g^\alpha}{g^i}$
        let right_side = self.global_parameters.as_ref().unwrap().hs[1] + point_commitment_inverted;
        let rhs = pairing(&witness.to_affine(), &right_side.to_affine());
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

    let polynomial_committer = GenericPolynomialCommitment::new();

    let max_degree = 25;

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
fn creates_and_verifies_witness_polynomial_evaluation() {
    env_logger::init();

    let mut polynomial_committer = GenericPolynomialCommitment::new();
    polynomial_committer.setup(3);

    let polynomial = Polynomial::new_from_bytes(&[1, 2, 3]);
    let point = Scalar::from(5);

    let commitment = polynomial_committer.commit(&polynomial);
    let (witness, evaluation) = polynomial_committer.create_witness(polynomial, point);
    let result =
        polynomial_committer.verify_evaluation(commitment.unwrap(), point, evaluation, witness);

    assert!(result);
}

#[test]
fn intuition_1() {
    let a = G1Projective::generator() * Scalar::from(5);
    let b = G2Projective::generator() * Scalar::from(6);
    let c = G2Projective::generator() * Scalar::from(5 * 6);

    let pairing_a = pairing(&a.into(), &b.into());
    let pairing_b = pairing(&G1Affine::generator(), &c.into());

    assert!(pairing_a == pairing_b);
}

#[test]
fn intuition_2() {
    let a = G1Projective::generator() * Scalar::from(5);
    let b = G1Projective::generator() * Scalar::from(6);

    // Some additive homomorphic property on the commitment
    let pairing_a = pairing(
        &(a + G1Projective::generator()).to_affine(),
        &G2Projective::generator().to_affine(),
    );
    let pairing_b = pairing(&b.to_affine(), &G2Projective::generator().to_affine());

    assert!(pairing_a == pairing_b);
}

#[test]
fn intuition_committed_polynomial_evaluation_basic() {
    // 39 == x^3 -4x^2 +3x -1
    // Only the point being evaluated raised to the degree of each coeefficient
    let x3 = G1Projective::generator() * Scalar::from(5_u64.pow(3));
    let x2 = G1Projective::generator() * Scalar::from(5_u64.pow(2));
    let x = G1Projective::generator() * Scalar::from(5);

    // commitment to the evaluation of the above
    let evaluation_commit = G1Projective::generator() * Scalar::from(39);

    let lhs = evaluation_commit;

    let rhs = x3 * Scalar::from(1)
        + x2 * -Scalar::from(4)
        + x * Scalar::from(3)
        + G1Projective::generator() * -Scalar::from(1);

    assert_eq!(lhs, rhs);
}
