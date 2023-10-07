use blstrs::Scalar;
use num_traits::pow;
use rand::RngCore;

use core::ops::Div;
use group::ff::Field;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Polynomial(pub Vec<Scalar>);

impl Polynomial {
    pub fn new(scalars: &[Scalar]) -> Self {
        Polynomial(scalars.to_vec())
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let scalars: Vec<Scalar> = bytes
            .into_iter()
            .map(|d| Scalar::from(d.clone() as u64))
            .collect();
        Polynomial(scalars.to_vec())
    }

    pub fn evaluate(&self, point: Scalar) -> Scalar {
        let mut total = Scalar::from(0_u64);
        for (i, coefficient) in self.0.iter().enumerate() {
            total += pow(point, i) * coefficient
        }
        total
    }

    // Adjust a polynomial by padding with randomness to a given degree, or if too large, truncate it to the degree
    pub fn adjust_to_degree(&mut self, d: usize) -> &mut Self {
        // Polynomial degree is too small, will padd
        if self.0.len() < d {
            let difference = d - self.0.len();
            let mut unfilled = vec![];
            rand::thread_rng();
            let mut rng = rand::thread_rng();
            for _ in 0..difference {
                unfilled.push(rng.next_u64())
            }

            let new_randoms: Vec<Scalar> = unfilled
                .into_iter()
                .map(|i| Scalar::from(i as u64))
                .collect();
            self.0.extend(new_randoms.iter());
            self
        } else if self.0.len() > d {
            // Polynomial degree is too big; truncate it
            // TODO: it should probably be more representative over the full polynomial than just keep the first `d`
            self.0.truncate(d);
            self
        } else {
            // If it's the right degree, no change
            self
        }
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty() || self.0.iter().all(|coeff| coeff.is_zero().into())
    }

    fn leading_coefficient(&self) -> Option<Scalar> {
        self.0.last().copied()
    }

    fn to_string(&self) -> String {
        let mut result = String::new();
        for (i, coeff) in self.0.iter().enumerate() {
            if i > 0 {
                result.push_str(" + ");
            }
            result.push_str(&format!("{}x^{}", coeff, i));
        }
        result
    }
}

// Division implementation from Arkworks
// TODO: Needs test
impl Div for Polynomial {
    type Output = Self;
    fn div(self, divisor: Self) -> Self::Output {
        if self.is_zero() {
            Polynomial::new(&[Scalar::from(0)])
        } else if divisor.is_zero() {
            panic!("Dividing by zero polynomial")
        } else if self.0.len() < divisor.0.len() {
            Polynomial::new(&[Scalar::from(0)])
        } else {
            // Now we know that self.degree() >= divisor.degree();
            let mut quotient =
                Polynomial::new(&vec![Scalar::ZERO; self.0.len() - divisor.0.len() + 1]);
            let mut remainder: Polynomial = self.clone().into();
            // Can unwrap here because we know self is not zero.
            let divisor_leading_inv = divisor.leading_coefficient().unwrap().invert().unwrap();
            while !remainder.is_zero() && remainder.0.len() >= divisor.0.len() {
                let cur_q_coeff = remainder.leading_coefficient().unwrap() * divisor_leading_inv;
                let cur_q_degree = remainder.0.len() - divisor.0.len();
                quotient.0[cur_q_degree] = cur_q_coeff;

                for (i, div_coeff) in divisor.0.iter().enumerate() {
                    remainder.0[cur_q_degree + i] -= &(cur_q_coeff * div_coeff);
                }
                while let Some(true) = remainder.0.last().map(|c| c.is_zero().into()) {
                    remainder.0.pop();
                }
            }
            quotient
        }
    }
}

#[test]
fn basic_evaluation() {
    let poly = Polynomial::new_from_bytes(&[1, 2, 3]);

    let point = Scalar::from(5_u64);
    assert_eq!(poly.evaluate(point), Scalar::from(86_u64));
}

#[test]
fn evaluation_with_leading_coefficient() {
    let poly = Polynomial::new_from_bytes(&[2, 4, 3]);
    let point = Scalar::from(6_u64);
    assert_eq!(poly.evaluate(point), Scalar::from(134_u64));
}

#[test]
fn divides_polynomials() {
    //  2x^2+5x+3
    let dividend = Polynomial::new(&vec![Scalar::from(2), Scalar::from(5), Scalar::from(3)]);
    // x + 1
    let divisor = Polynomial::new(&vec![Scalar::from(1), Scalar::from(1)]);
    // 2x+3
    let ans: Polynomial = Polynomial::new(&[Scalar::from(2), Scalar::from(3)]);
    assert_eq!(dividend / divisor, ans)
}
