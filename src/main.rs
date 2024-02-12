mod finite_field;
pub use crate::finite_field::FiniteField;

mod elliptic_curve;
pub use crate::elliptic_curve::EllipticCurve;


mod secp256k1;
use num_bigint::BigUint;
use crate::secp256k1::Secp256k1;
use crate::elliptic_curve::Point;

mod ecdsa;
pub use crate::ecdsa::{EcdsaKeyPair, EcdsaSignature};

fn main() {
    let secp256k1 = Secp256k1::new();
    let private_key = BigUint::from(123456789u64);

    match Secp256k1::generate_public_key(&secp256k1, private_key) {
        Ok(public_key) => {
            match public_key {
                Point::Coor(x, y) => println!("Public key:\n\nx: {:?}\ny: {:?}", x, y),
                Point::Identity => println!("Public key is at the identity point."),
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
