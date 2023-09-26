use blstrs::Scalar;
use num_traits::pow;

pub struct Polynomial(pub Vec<Scalar>);

impl Polynomial {
    fn new(scalars: &[Scalar]) -> Self {
        Polynomial(
            scalars.to_vec()
        )
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let scalars: Vec<Scalar> = bytes.into_iter().map(|d| Scalar::from(d.clone() as u64)).collect();
        Polynomial(
            scalars.to_vec()
        )
    }

    fn evaluate(&self, point: Scalar) -> Scalar {
        let mut total = Scalar::from(0_u64);
        for (i, coefficient) in self.0.iter().enumerate() {
            total += pow(point, i) * coefficient
        }
        total
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
