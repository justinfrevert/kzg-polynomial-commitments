use blstrs::Scalar;
use num_traits::pow;
use rand::RngCore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Polynomial(pub Vec<Scalar>);

impl Polynomial {
    fn new(scalars: &[Scalar]) -> Self {
        Polynomial(scalars.to_vec())
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let scalars: Vec<Scalar> = bytes
            .into_iter()
            .map(|d| Scalar::from(d.clone() as u64))
            .collect();
        Polynomial(scalars.to_vec())
    }

    fn evaluate(&self, point: Scalar) -> Scalar {
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
