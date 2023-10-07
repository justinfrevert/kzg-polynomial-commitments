use std::ops::AddAssign;
use std::ops::{Add, Div, Mul, Neg, Sub};

use num_bigint::BigUint;
use rand::thread_rng;
use rand::Rng;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field(pub BigUint);

impl Field {
    // Get random field element from this field
    pub fn rand(&self) -> FieldElement {
        let mut rng = thread_rng();
        let random_field_element_number = rng.gen_range(0..self.0.clone().try_into().unwrap());

        // Return the random field element
        FieldElement {
            value: BigUint::from(random_field_element_number as u32),
            field: self.clone(),
        }
    }

    // fn find_optimal_root_of_unity(&self) -> Option<u64> {
    //     // Iterate backwards to find the greatest matching field element first, we are interested in the greatest one
    //     for i in (0..self.0 - 1).rev() {
    //         // We are looking for the greatest number which the p of the multiplicative group can divide
    //         if (self.0 - 1) % i == 0 {
    //             return Some(i);
    //         }
    //     }
    //     return None;
    // }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldElement {
    pub value: BigUint,
    field: Field,
}

impl FieldElement {
    pub fn new(value: BigUint, field: Field) -> Self {
        FieldElement { value, field }
    }
}

impl FieldElement {
    pub fn pow(self, rhs: Self) -> FieldElement {
        assert!(self.field == rhs.field);
        let result = self.value.pow(
            rhs.value
                .try_into()
                .expect("Could not fit exponent into u32"),
        ) % self.clone().field.0;

        FieldElement::new(result, self.field)
    }
}

impl Add for FieldElement {
    type Output = FieldElement;
    fn add(self, rhs: Self) -> Self::Output {
        let ans = (self.value + rhs.value) % self.field.clone().0;
        FieldElement::new(ans, self.field)
    }
}

impl AddAssign for FieldElement {
    fn add_assign(&mut self, rhs: Self) {
        // let result = (self.value + rhs.value) % self.field.0;
        self.value = (self.clone().value + rhs.value) % self.clone().field.0;
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(self) -> Self::Output {
        let result = (self.clone().field.0 - self.clone().value) % self.clone().field.clone().0;
        // let result = (self.field.0 - self.value) % self.field.0;
        FieldElement::new(result, self.clone().field)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: Self) -> Self::Output {
        let result = (self.clone().value * rhs.value) % self.clone().field.0;
        FieldElement::new(result, self.field)
    }
}

impl Div for FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: Self) -> Self::Output {
        // (self.value / rhs.value) % self.field.0
        let result = (self.value / rhs.value) % self.field.clone().0;
        FieldElement::new(result, self.field)
    }
}

#[test]
fn adds() {
    let field = Field(BigUint::from(101_u32));

    let field_element_lower = FieldElement {
        value: BigUint::from(100_u32),
        field: field.clone(),
    };

    let field_element_higher = FieldElement {
        value: BigUint::from(2_u32),
        field: field.clone(),
    };

    assert_eq!(
        field_element_lower + field_element_higher,
        FieldElement::new(BigUint::from(1_u32), field)
    );
}

#[test]
fn subtracts() {
    let field = Field(BigUint::from(101_u32));

    let field_element_lower = FieldElement {
        value: BigUint::from(3_u32),
        field: field.clone(),
    };

    let field_element_higher = FieldElement {
        value: BigUint::from(5_u32),
        field: field.clone(),
    };

    assert_eq!(
        field_element_lower - field_element_higher,
        FieldElement::new(BigUint::from(99_u32), field)
    );
}

#[test]
fn multiplies() {
    let field = Field(BigUint::from(101_u32));

    let field_element_lower = FieldElement {
        value: BigUint::from(103_u32),
        field: field.clone(),
    };

    let field_element_higher = FieldElement {
        value: BigUint::from(1_u32),
        field: field.clone(),
    };

    assert_eq!(
        field_element_lower * field_element_higher,
        FieldElement {
            value: BigUint::from(2_u32),
            field
        }
    );
}

#[test]
fn exponentiates() {
    let field = Field(BigUint::from(101_u32));

    let field_element_lower = FieldElement {
        value: BigUint::from(2_u32),
        field: field.clone(),
    };

    let field_element_higher = FieldElement {
        value: BigUint::from(7_u32),
        field: field.clone(),
    };

    assert_eq!(
        field_element_lower.pow(field_element_higher),
        FieldElement::new(BigUint::from(27_u32), field)
    );
}

#[test]
fn example_field() {
    let field = Field(BigUint::from(41_u32));

    let field_element_lower = FieldElement {
        value: BigUint::from(1_u32),
        field: field.clone(),
    };

    let field_element_higher = FieldElement {
        value: BigUint::from(40_u32),
        field: field.clone(),
    };

    assert_eq!(
        field_element_lower + field_element_higher,
        FieldElement::new(BigUint::from(0_u32), field)
    );
}

#[test]
fn low_field() {
    let field = Field(BigUint::from(13_u32));

    let field_element_lower = FieldElement {
        value: BigUint::from(3_u32),
        field: field.clone(),
    };

    let field_element_higher = FieldElement {
        value: BigUint::from(10_u32),
        field: field.clone(),
    };

    assert_eq!(
        field_element_lower + field_element_higher,
        FieldElement::new(BigUint::from(0_u32), field)
    );
}

// #[test]
// fn find_optimal_root_of_unity() {
//     let field = Field(41);
//     assert_eq!(field.find_optimal_root_of_unity(), Some(8));
// }
