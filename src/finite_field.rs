use num_bigint::{BigUint};

#[derive(PartialEq, Clone, Debug)]
pub struct FiniteField {
    pub value: BigUint,
    pub p: BigUint,
}

impl FiniteField {
    // since value is an element of Fp, value should be less than p 
    pub fn new(value: BigUint, p:BigUint) -> Self {

        // println!("Creating FiniteField with value: {:?}, p: {:?}", value, p);
    
        assert!(value < p, "Value should be less than p");
        Self {value: value % &p, p}
    }

    pub fn get_value(&self) -> &BigUint {
        &self.value
    }

    // add two elements of Fp
    // (a + b) mod p
    pub fn add(&self, other: &FiniteField) -> Result<FiniteField, &'static str> {
        if self.p != other.p {
            return Err("Operands must be from the same field (p should be the same)");
        }
        Ok(FiniteField {
            value: (&self.value + &other.value) % &self.p,
            p: self.p.clone(),
        })
    }
    
    // subtract two elements of Fp
    // to ensure a - b is positive, add p to a - b
    // (a + p - b) mod p
    pub fn sub(&self, other: &FiniteField) -> Result<FiniteField, &'static str> {
        if self.p != other.p {
            return Err("Operands must be from the same field (p should be the same)");
        }
        Ok(FiniteField {
            value: (&self.value + &self.p - &other.value) % &self.p,
            p: self.p.clone(),
        })
    }

    // multiply two elements of Fp
    // (a * b) mod p
    pub fn mul(&self, other: &FiniteField) -> Result<FiniteField, &'static str> {
        if self.p != other.p {
            return Err("Operands must be from the same field (p should be the same)");
        }
        Ok(FiniteField {
            value: (&self.value * &other.value) % &self.p,
            p: self.p.clone(),
        })
    }

    // divide two elements of Fp
    // by Fermat's little theorem, a^(p-1) = 1 mod p s.t. gcd(a,p) = 1
    // thus, a^(p-2) = a^-1 mod p
    pub fn div(&self, other: &FiniteField) -> Result<FiniteField, &'static str> {
        if self.p != other.p {
            return Err("Operands must be from the same field (p should be the same)");
        }
        if other.value == BigUint::from(0u32) {
            return Err("Cannot divide by zero")
        }
        
        let exponent = &self.p - BigUint::from(2u32);
        Ok(FiniteField {
            value: (&self.value * &other.value.modpow(&exponent, &self.p)) % &self.p,
            p: self.p.clone(),
        })
}
}

// Test cases for FiniteField
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a = FiniteField::new(BigUint::from(2u32), BigUint::from(7u32));
        let b = FiniteField::new(BigUint::from(4u32), BigUint::from(7u32));
        let c = FiniteField::new(BigUint::from(6u32), BigUint::from(7u32));

        assert_eq!(a.add(&b), Ok(c));
    }

    #[test]
    fn test_sub() {
        let a = FiniteField::new(BigUint::from(2u32), BigUint::from(7u32));
        let b = FiniteField::new(BigUint::from(4u32), BigUint::from(7u32));
        let c = FiniteField::new(BigUint::from(5u32), BigUint::from(7u32));

        assert_eq!(a.sub(&b), Ok(c));
    }

    #[test]
    fn test_mul() {
        let a = FiniteField::new(BigUint::from(2u32), BigUint::from(7u32));
        let b = FiniteField::new(BigUint::from(4u32), BigUint::from(7u32));
        let c = FiniteField::new(BigUint::from(1u32), BigUint::from(7u32));

        assert_eq!(a.mul(&b), Ok(c));
    }

    #[test]
    fn test_div() {
        let a = FiniteField::new(BigUint::from(2u32), BigUint::from(7u32));
        let b = FiniteField::new(BigUint::from(4u32), BigUint::from(7u32));
        let c = FiniteField::new(BigUint::from(4u32), BigUint::from(7u32));

        assert_eq!(a.div(&b), Ok(c));
    }
}