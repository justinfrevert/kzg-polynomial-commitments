mod field;
mod polynomial_commitments;
mod polynomials;

use num_bigint::BigUint;
use polynomial_commitments::{GenericPolynomialCommitment, PolynomialCommitment};

use crate::{field::FieldElement, polynomials::Polynomial};

fn main() {

    let max_degree = 20;

    let mut polynomial_commitment = GenericPolynomialCommitment::new();

    // Some generator
    // TODO: try replacing with reference to field

    let gp = polynomial_commitment.setup(max_degree);

    let my_data = "Justin".as_bytes();

    Polynomial::new_from_bytes(my_data);

    // let commitment = polynomial_commitment.commit(polynomial, &gp);
    // println!("commitment is {:?}", commitment);
}
