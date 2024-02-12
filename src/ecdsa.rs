pub use crate::elliptic_curve::{EllipticCurve, Point};
pub use crate::finite_field::FiniteField;
use num_bigint::{BigUint, RandBigInt};
use sha2::{Digest, Sha256};
use rand::rngs::OsRng;
use num_traits::{Num, Zero};

// ECDSA Key Pair
pub struct EcdsaKeyPair {
    pub private_key: BigUint,
    pub public_key: Point,
}

// ECDSA Signature
#[derive(Debug)]
pub struct EcdsaSignature {
    pub r: BigUint,
    pub s: BigUint,
}

impl EcdsaKeyPair {
    // Efficient key generation, minimizing cloning
    pub fn generate(curve: &EllipticCurve) -> Self {
        let mut rng = OsRng;
        let private_key = rng.gen_biguint_below(&curve.p);
        let public_key = curve.scalar_mul(&curve.g, private_key.clone())
                             .expect("Scalar multiplication failed");

        EcdsaKeyPair { private_key, public_key }
    }
}

impl EcdsaSignature {
    // Refactored signature function to improve clarity and error handling
    pub fn sign(curve: &EllipticCurve, message: &[u8], private_key: &BigUint) -> Result<Self, String> {
        let hash = hash_message(message);
        let mut rng = OsRng;
        let k = generate_nonzero_random(&mut rng, &curve.p);

        let r_point = curve.scalar_mul(&curve.g, k.clone())
                          .map_err(|e| e.to_string())?;

        if let Point::Coor(x, _) = r_point {
            let r_field = FiniteField::new(x.get_value().clone(), curve.p.clone());
            let private_key_field = FiniteField::new(private_key.clone(), curve.p.clone());
            let hash_field = FiniteField::new(hash, curve.p.clone());
            
            let s_field = calculate_s_field(&hash_field, &r_field, &private_key_field, &k, &curve.p)?;
            println!("r: {:?}, s: {:?}", r_field.get_value(), s_field.get_value());
            Ok(EcdsaSignature { r: r_field.get_value().clone(), s: s_field.get_value().clone() })
        } else {
            Err("Invalid r_point generated".to_string())
        }
    }

    // Verification function with improved error handling
    pub fn verify(curve: &EllipticCurve, message: &[u8], public_key: &Point, signature: &EcdsaSignature) -> Result<bool, String> {
        let hash = hash_message(message);
        let hash_field = FiniteField::new(hash, curve.p.clone());

        let signature_s_field = FiniteField::new(signature.s.clone(), curve.p.clone());
        let one_field = FiniteField::new(BigUint::from(1u32), curve.p.clone());

        let w = one_field.div(&signature_s_field)
                         .map_err(|e| e.to_string())?;

        let u1 = hash_field.mul(&w)?;
        let u2 = FiniteField::new(signature.r.clone(), curve.p.clone()).mul(&w)?;

        let u1_point = curve.scalar_mul(&curve.g, u1.get_value().clone())?;
        let u2_point = curve.scalar_mul(public_key, u2.get_value().clone())?;

        let p = curve.add(&u1_point, &u2_point)
                    .map_err(|e| e.to_string())?;
        
        match p {
            Point::Coor(x, _) => Ok(x == FiniteField::new(signature.r.clone(), curve.p.clone())),
            _ => Err("Invalid point generated in verification".to_string()),
        }
    }
}

// Helper functions
fn hash_message(message: &[u8]) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash_result = hasher.finalize();
    BigUint::from_bytes_be(&hash_result)
}

fn generate_nonzero_random(rng: &mut OsRng, p: &BigUint) -> BigUint {
    loop {
        let k = rng.gen_biguint_below(p);
        if k != BigUint::zero() {
            return k;
        }
    }
}

fn calculate_s_field(hash_field: &FiniteField, r_field: &FiniteField, private_key_field: &FiniteField, k: &BigUint, p: &BigUint) -> Result<FiniteField, String> {
    let k_field = FiniteField::new(k.clone(), p.clone());
    hash_field.add(&r_field.mul(private_key_field)?)
             .and_then(|num| num.div(&k_field))
             .map_err(|e| e.to_string())
}


// Test cases for EcdsaKeyPair and EcdsaSignature
#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigUint;
    use crate::ecdsa;

    #[test]
    fn test_sign_normal_operation() {
        
        let p = BigUint::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
        let a = BigUint::from(0u32); // For secp256k1, a is 0
        let b = BigUint::from(7u32); // For secp256k1, b is 7
        let g_x = BigUint::from_str_radix("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
        let g_y = BigUint::from_str_radix("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();

        let curve = EllipticCurve {
            a: FiniteField::new(a, p.clone()),
            b: FiniteField::new(b, p.clone()),
            p: p.clone(),
            g: Point::Coor(
                FiniteField::new(g_x, p.clone()),
                FiniteField::new(g_y, p.clone())
            ),
        };

        let key_pair = EcdsaKeyPair::generate(&curve);
        let message = "test message".as_bytes();

        let signature_result = EcdsaSignature::sign(&curve, message, &key_pair.private_key);

        assert!(signature_result.is_ok(), "Failed to sign message");
        let signature = signature_result.unwrap();

        // Optionally, you can add more checks here, e.g., on the structure of the signature
    }

    #[test]
    fn test_verify() {
        // Setup the elliptic curve parameters (for example, using secp256k1)
        // You'll need to define these parameters based on your implementation
        let p = BigUint::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
        let a = BigUint::from(0u32); // For secp256k1, a is 0
        let b = BigUint::from(7u32); // For secp256k1, b is 7
        let g_x = BigUint::from_str_radix("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
        let g_y = BigUint::from_str_radix("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();
        let curve = EllipticCurve {
            a: FiniteField::new(a, p.clone()),
            b: FiniteField::new(b, p.clone()),
            p: p.clone(),
            g: Point::Coor(
                FiniteField::new(g_x, p.clone()),
                FiniteField::new(g_y, p.clone())
            ),
        };

        // Generate a key pair
        let key_pair = EcdsaKeyPair::generate(&curve);

        // Define a message
        let message = "Hello, world".as_bytes();

        // Sign the message
        let signature = EcdsaSignature::sign(&curve, message, &key_pair.private_key).unwrap();

        // Verify the signature
        let is_valid = EcdsaSignature::verify(&curve, message, &key_pair.public_key, &signature).unwrap();

        // Assert that the signature is valid
        assert!(is_valid, "The signature should be valid.");
    }
}