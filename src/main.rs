mod field;
mod polynomial_commitments;
mod polynomials;

use field::Field;
use num_bigint::BigUint;
use polynomial_commitments::{GenericPolynomialCommitment, PolynomialCommitment};

use crate::{field::FieldElement, polynomials::Polynomial};

fn main() {
    let g1 = Field(BigUint::from(41_u32));

    let t = g1.rand();

    let max_degree = 20;

    let polynomial_commitment = GenericPolynomialCommitment::new(g1.clone());

    // Some generator
    // TODO: try replacing with reference to field
    let generator = FieldElement::new(BigUint::from(1_u32), g1);

    let gp = polynomial_commitment.setup(t, max_degree, generator);

    println!("Global parameters are {:?}", gp);

    let my_data = "Justin".as_bytes();

    let polynomial = Polynomial::new(my_data);

    let commitment = polynomial_commitment.commit(polynomial, &gp);
    println!("commitment is {:?}", commitment);
}
