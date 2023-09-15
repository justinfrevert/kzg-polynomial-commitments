use std::ops::{Add, Sub, Neg, Mul, Div};

#[derive(Clone, PartialEq, Eq)]
struct Field(u64);

impl Field {
    fn find_optimal_root_of_unity(&self) -> Option<u64> {
        // Iterate backwards to find the greatest matching field element first, we are interested in the greatest one
        for i in (0..self.0 - 1).rev() {
            println!("Checking self.0 - 1 {:?}  %  {:?} == 0", self.0 - 1, i);
            // We are looking for the greatest number which the p of the multiplicative group can divide
            if (self.0 - 1) % i == 0 {
                println!("Got a match");
                return Some(i)
            }
        }
        return None
    }
}

struct FieldElement {
    value: u64,
    field: Field
}


impl FieldElement {
    fn pow(self, rhs: Self) -> u64 {
        assert!(self.field == rhs.field);
        self.value.pow(rhs.value.try_into().expect("Could not fit exponent into u32")) % self.field.0
    }
}

impl Add for FieldElement {
    type Output = u64;
    fn add(self, rhs: Self) -> Self::Output {
        (self.value + rhs.value) % self.field.0
    }
}

impl Sub for FieldElement {
    type Output = u64;
    fn sub(self, rhs: Self) -> Self::Output {
        self.value + -rhs
    }
}

impl Neg for FieldElement {
    type Output = u64;

    fn neg(self) -> Self::Output {
        (self.field.0  - self.value) % self.field.0
    }
}

impl Mul for FieldElement {
    type Output = u64;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.value  * rhs.value) % self.field.0
    }
}

impl Div for FieldElement {
    type Output = u64;

    fn div(self, rhs: Self) -> Self::Output {
        (self.value / rhs.value) % self.field.0
    }
}


fn main() {
    let field = Field(41);

    let generator = 1;

    // let secret = 

    field.find_optimal_root_of_unity();

}

#[test]
fn adds() {
    let field = Field(101);

    let field_element_lower = FieldElement {
        value: 100,
        field: field.clone()
    };

    let field_element_higher = FieldElement {
        value: 2,
        field
    };

    assert_eq!(field_element_lower + field_element_higher, 1);
}

#[test]
fn subtracts() {
    let field = Field(101);

    let field_element_lower = FieldElement {
        value: 3,
        field: field.clone()
    };

    let field_element_higher = FieldElement {
        value: 5,
        field
    };

    assert_eq!(field_element_lower - field_element_higher, 99);
}

#[test]
fn multiplies() {
    let field = Field(101);

    let field_element_lower = FieldElement {
        value: 103,
        field: field.clone()
    };

    let field_element_higher = FieldElement {
        value: 1,
        field
    };

    assert_eq!(field_element_lower * field_element_higher, 2);
}

#[test]
fn exponentiates() {
    let field = Field(101);

    let field_element_lower = FieldElement {
        value: 2,
        field: field.clone()
    };

    let field_element_higher = FieldElement {
        value: 7,
        field
    };

    assert_eq!(field_element_lower.pow(field_element_higher), 27);
}

#[test]
fn example_field() {
    let field = Field(41);

        let field_element_lower = FieldElement {
        value: 1,
        field: field.clone()
    };

    let field_element_higher = FieldElement {
        value: 40,
        field
    };

    assert_eq!(field_element_lower + field_element_higher,  0);

}

#[test]
fn low_field() {
    let field = Field(13);

        let field_element_lower = FieldElement {
        value: 3,
        field: field.clone()
    };

    let field_element_higher = FieldElement {
        value: 10,
        field
    };

    assert_eq!(field_element_lower + field_element_higher,  0);

}


#[test]
fn find_optimal_root_of_unity() {
    
        let field = Field(41);
        assert_eq!(field.find_optimal_root_of_unity(), Some(8));

}