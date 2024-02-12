use num_bigint::BigUint;
use num_traits::Num;
use crate::finite_field::FiniteField;
use crate::elliptic_curve::{EllipticCurve, Point};

pub struct Secp256k1 {
    pub elliptic_curve: EllipticCurve,
}

impl Secp256k1 {
    pub fn new() -> Self {
        let p = BigUint::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
        let a = FiniteField::new(BigUint::from(0u32), p.clone());
        let b = FiniteField::new(BigUint::from(7u32), p.clone());
        let g = Point::Coor(FiniteField::new(BigUint::from_str_radix("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap(), p.clone()),FiniteField::new(BigUint::from_str_radix("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap(), p.clone()),
    );

        Secp256k1 {
            elliptic_curve: EllipticCurve { a, b, p ,g},
        }
    }

    pub fn generate_public_key(&self, private_key: BigUint) -> Result<Point, &'static str> {
        self.elliptic_curve.scalar_mul(&self.elliptic_curve.g, private_key)
    }
}