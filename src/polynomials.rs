use num_bigint::BigUint;

pub struct Polynomial(pub Vec<BigUint>);

impl Polynomial {
    pub fn new(data: &[u8]) -> Self {
        Polynomial(
            data.to_vec()
                .into_iter()
                .map(|d| BigUint::from(d))
                .collect(),
        )
    }

    pub fn evaluate(&self, point: &BigUint) -> BigUint {
        let mut total = BigUint::from(0_u32);
        for (i, coefficient) in self.0.iter().enumerate() {
            total += point.pow(i as u32) * coefficient
        }
        total
    }
}

#[test]
fn basic_evaluation() {
    let poly = Polynomial::new(&[1, 2, 3]);

    let point = BigUint::from(5_u32);
    assert_eq!(poly.evaluate(&point), BigUint::from(86_u32));
}

#[test]
fn evaluation_with_leading_coefficient() {
    let poly = Polynomial::new(&[2, 4, 3]);
    let point = BigUint::from(6_u32);
    assert_eq!(poly.evaluate(&point), BigUint::from(134_u32));
}
