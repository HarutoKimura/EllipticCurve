use num_bigint::{BigUint};
pub use crate::finite_field::FiniteField;

#[derive(PartialEq, Clone, Debug)]
pub enum Point {
    Coor(FiniteField, FiniteField),
    Identity,
}

#[derive(PartialEq, Clone, Debug)]
pub struct EllipticCurve {
    pub a: FiniteField,
    pub b: FiniteField,
    pub p: BigUint,
    pub g: Point,
}

impl EllipticCurve {
    pub fn add(&self, c: &Point, d: &Point) -> Result<Point, &'static str> {
        if !self.is_on_curve(c)? {
            return Err("Point is not on the curve");
        }
        if !self.is_on_curve(d)? {
            return Err("Point is not on the curve");
        }

        match (c, d) {
            (Point::Identity, _) => Ok(d.clone()),
            (_, Point::Identity) => Ok(c.clone()),
            (Point::Coor(x1, y1), Point::Coor(x2, y2)) =>  {
                if x1 == x2 && y1.add(&y2)? == FiniteField::new(BigUint::from(0u32), self.p.clone()) {
                    return Ok(Point::Identity);
                }
                //  s = (y2 -y1) / (x2 - x1) mod p
                // x3 = s^2 - x1 - x2 mod p
                // y3 = -s(x3 - x1) -x1 mod p
                let slope_num = y2.sub(&y1)?;
                let slope_den = x2.sub(&x1)?;
                let s = slope_num.div(&slope_den)?;

                let x3_y3 = self.compute_x3_y3(&x1, &y1, &x2, &s)?;
                Ok(Point::Coor(x3_y3.0, x3_y3.1))
            }
        }
    }

    pub fn double(&self, c: &Point) -> Result<Point, &'static str> {
        if !self.is_on_curve(c)? {
            return Err("Point is not on the curve");
        }

        match c {
            Point::Identity => Ok(Point::Identity),
            Point::Coor(x1, y1) => {
                // s = (3 * x1^2 + a) / (2 * y1) mod p
                // x3 = s^2 - 2 * x1 mod p
                // y3 = s(x1 - x3) - y1 mod p
            let x_squared = x1.mul(x1)?;
            let three = FiniteField::new(BigUint::from(3u32), self.p.clone());
            let three_times_x_squared = x_squared.mul(&three)?;
            let slope_num = three_times_x_squared.add(&self.a)?;

            let two_y1 = y1.mul(&FiniteField::new(BigUint::from(2u32), self.p.clone()))?;

            let s = slope_num.div(&two_y1)?;

            let x3_y3 = self.compute_x3_y3(x1, y1, x1, &s)?;
            Ok(Point::Coor(x3_y3.0, x3_y3.1))
            }
        }
    }
    // x3 = s^2 - x1 -x2 mod p
    // y3 = s(x1 -x3) -y1 mod p
    fn compute_x3_y3(&self, x1: &FiniteField, y1: &FiniteField, x2: &FiniteField, s: &FiniteField) -> Result<(FiniteField, FiniteField), &'static str> {
        let s_squared = s.mul(&s)?;
        let x1_plus_x2 = x1.add(&x2)?;
        let x3 = s_squared.sub(&x1_plus_x2)?;

        let x1_minus_x3 = x1.sub(&x3)?;
        let s_times_x1_minus_x3 = s.mul(&x1_minus_x3)?;
        let y3 = s_times_x1_minus_x3.sub(&y1)?;

        if !self.is_on_curve(&Point::Coor(x3.clone(), y3.clone()))? {
            return Err("Resulting point is not on the curve");
        }

        Ok((x3, y3))
    }
    
    // add-double algorithm for scalar multiplication - B = d*A
    // index increasing from LSB to MSB\

    // pub fn scalar_mul(&self, p: &Point, s: BigUint) -> Result<Point, &'static str> {
    //         // Convert s to a vector of bits (LSB to MSB)
    //         let bits = s.to_radix_le(2);
        
    //         // Start with the identity point (point at infinity)
    //         let mut res = Point::Identity;
        
    //         // This will hold the doubled value of P in each iteration
    //         let mut temp = p.clone();
        
    //         // Iterate over each bit
    //         for bit in bits {
    //             if bit == 1 {
    //                 // If the bit is 1, add the temp point to res
    //                 res = self.add(&res, &temp)?;
    //             }
    //             // Double the temp point for the next iteration
    //             temp = self.double(&temp)?;
    //         }
        
    //         Ok(res)
    //     }

    // double-add algorithm for scalar multiplication - B =d*A
    // index decreasing from MSB to LSB

    // pub fn scalar_mul(&self, p: &Point, s: BigUint) -> Result<Point, &'static str> {
    //     // Check if the scalar s is zero
    //     if s == BigUint::from(0u32) {
    //         return Ok(Point::Identity); // Return the identity point for scalar 0
    //     }
    
    //     let bits = s.to_radix_le(2);
    //     let mut res = p.clone(); // Start with point P
    //     let mut i = bits.len() - 1;
    
    //     while i > 0 {
    //         i -= 1;
    //         res = self.double(&res)?; // Double the point
    
    //         if bits[i] == 1 {
    //             res = self.add(&res, p)?; // Add P if the current bit is 1
    //         }
    //     }
    
    //     Ok(res)
    // }

    // Recursively compute the scalar multiplication - B = d*A
    pub fn scalar_mul(&self, p: &Point, s: BigUint) -> Result<Point, &'static str> {
        if !self.is_on_curve(&p)? {
            return Err("Point is not on the curve");
        } 
        else if s == BigUint::from(0u32) { // Check if the scalar s is zero
            return Ok(Point::Identity);
        } 
        else if s.clone() == BigUint::from(1u32) {  // Check if the scalar s is one
            return Ok(p.clone());
        } 
        else if s.clone() % BigUint::from(2u32) == BigUint::from(1u32) {
            let scalar_mul_result = self.scalar_mul(p, s - BigUint::from(1u32))?;
            self.add(p, &scalar_mul_result) // addtion when s is odd
        } 
        else { // 
            let double_result = self.double(p)?; // double when s is even
            self.scalar_mul(&double_result, s / BigUint::from(2u32))
        }
    }

    // check wether the point is on the curve or not
    // y^2 = x^3 + ax + b mod p
    pub fn is_on_curve(&self, c: &Point) -> Result<bool, &'static str> {
        match c {
            Point::Identity => Ok(Point::Identity == Point::Identity),
            Point::Coor(x, y) => {
                //y^2 
                let y_squared = y.mul(&y)?;
                //x^3 
                let x_cubed = x.mul(&x)?.mul(&x)?;
    
                let ax = self.a.mul(&x)?;
                // check y^2 = x^3 + ax + b mod p
                let right_side = x_cubed.add(&ax)?.add(&self.b)?;
    
                Ok(y_squared == right_side)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        //y^2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            b: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            p: BigUint::from(17u32),
            g: Point::Coor(FiniteField::new(BigUint::from(5u32), BigUint::from(17u32)), FiniteField::new(BigUint::from(1u32), BigUint::from(17u32))),
        };

        // (5, 1) + (6, 3) = (10, 6)
        let p1 = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p2 = Point::Coor(FiniteField::new(BigUint::from(6u32), curve.p.clone()), FiniteField::new(BigUint::from(3u32), curve.p.clone()));
        let p3 = Point::Coor(FiniteField::new(BigUint::from(10u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        assert_eq!(curve.add(&p1, &p2), Ok(p3));

        // (5, 1) + (3, 1) = (9, 16)
        let p1 = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p2 = Point::Coor(FiniteField::new(BigUint::from(3u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p3 = Point::Coor(FiniteField::new(BigUint::from(9u32), curve.p.clone()), FiniteField::new(BigUint::from(16u32), curve.p.clone()));
        assert_eq!(curve.add(&p1, &p2), Ok(p3));

        // (5, 1) + (9, 16) = (16, 13)
        let p1 = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p2 = Point::Coor(FiniteField::new(BigUint::from(9u32), curve.p.clone()), FiniteField::new(BigUint::from(16u32), curve.p.clone()));
        let p3 = Point::Coor(FiniteField::new(BigUint::from(16u32), curve.p.clone()), FiniteField::new(BigUint::from(13u32), curve.p.clone()));
        assert_eq!(curve.add(&p1, &p2), Ok(p3));

        // (5, 1) + (16, 13) = (0, 6)
        let p1 = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p2 = Point::Coor(FiniteField::new(BigUint::from(16u32), curve.p.clone()), FiniteField::new(BigUint::from(13u32), curve.p.clone()));
        let p3 = Point::Coor(FiniteField::new(BigUint::from(0u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        assert_eq!(curve.add(&p1, &p2), Ok(p3));

        // (5, 1) + (0, 6) = (13, 7)
        let p1 = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p2 = Point::Coor(FiniteField::new(BigUint::from(0u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        let p3 = Point::Coor(FiniteField::new(BigUint::from(13u32), curve.p.clone()), FiniteField::new(BigUint::from(7u32), curve.p.clone()));
        assert_eq!(curve.add(&p1, &p2), Ok(p3));

        // (5, 1) + (13, 7) = (7, 6)
        let p1 = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let p2 = Point::Coor(FiniteField::new(BigUint::from(13u32), curve.p.clone()), FiniteField::new(BigUint::from(7u32), curve.p.clone()));
        let p3 = Point::Coor(FiniteField::new(BigUint::from(7u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        assert_eq!(curve.add(&p1, &p2), Ok(p3));

    }

    #[test]
    fn test_double() {
        //y^ 2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            b: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            p: BigUint::from(17u32),
            g: Point::Coor(FiniteField::new(BigUint::from(5u32), BigUint::from(17u32)), FiniteField::new(BigUint::from(1u32), BigUint::from(17u32))),
        };

        // 2(5, 1) = (6, 3) -> d = 2
        let point = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let double_point = Point::Coor(FiniteField::new(BigUint::from(6u32), curve.p.clone()), FiniteField::new(BigUint::from(3u32), curve.p.clone()));
        let calculated_double_point = curve.double(&point);
        assert_eq!(calculated_double_point, Ok(double_point));

        // 2(6, 3) = (3, 1) -> d = 4
        let point = Point::Coor(FiniteField::new(BigUint::from(6u32), curve.p.clone()), FiniteField::new(BigUint::from(3u32), curve.p.clone()));
        let double_point = Point::Coor(FiniteField::new(BigUint::from(3u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let calculated_double_point = curve.double(&point);
        assert_eq!(calculated_double_point, Ok(double_point));

        // 2(3, 1) = (13, 7) -> d = 8
        let point = Point::Coor(FiniteField::new(BigUint::from(3u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        let double_point = Point::Coor(FiniteField::new(BigUint::from(13u32), curve.p.clone()), FiniteField::new(BigUint::from(7u32), curve.p.clone()));
        let calculated_double_point = curve.double(&point);
        assert_eq!(calculated_double_point, Ok(double_point));

    }

    // necessary to change the value of d and test
    #[test]
    fn test_scalar_mul() {
        // y^2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            b: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            p: BigUint::from(17u32),
            g: Point::Coor(FiniteField::new(BigUint::from(5u32), BigUint::from(17u32)), FiniteField::new(BigUint::from(1u32), BigUint::from(17u32))),
        };
        let point = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));

        // 2(5, 1) = (6, 3)
        let second_point = Point::Coor(FiniteField::new(BigUint::from(6u32), curve.p.clone()), FiniteField::new(BigUint::from(3u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(2u32)), Ok(second_point));

        // 3(5, 1) = (10, 6)
        let second_point = Point::Coor(FiniteField::new(BigUint::from(10u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(3u32)), Ok(second_point));

        // 4(5, 1) = (3, 1)
        let second_point = Point::Coor(FiniteField::new(BigUint::from(3u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(4u32)), Ok(second_point));

        // 5(5, 1) = (9, 16)
        let second_point = Point::Coor(FiniteField::new(BigUint::from(9u32), curve.p.clone()), FiniteField::new(BigUint::from(16u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(5u32)), Ok(second_point));

        // 6(5, 1) = (16, 13) 
        let second_point = Point::Coor(FiniteField::new(BigUint::from(16u32), curve.p.clone()), FiniteField::new(BigUint::from(13u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(6u32)), Ok(second_point));

        // 7(5, 1) = (0, 6)
        let second_point = Point::Coor(FiniteField::new(BigUint::from(0u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(7u32)), Ok(second_point));

        // 8(5, 1) = (13, 7) 
        let second_point = Point::Coor(FiniteField::new(BigUint::from(13u32), curve.p.clone()), FiniteField::new(BigUint::from(7u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(8u32)), Ok(second_point));

        // 9(5, 1) = (7, 6) 
        let second_point = Point::Coor(FiniteField::new(BigUint::from(7u32), curve.p.clone()), FiniteField::new(BigUint::from(6u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(9u32)), Ok(second_point));

        // 10(5, 1) = (7, 11)
        let tenth_point = Point::Coor(FiniteField::new(BigUint::from(7u32), curve.p.clone()), FiniteField::new(BigUint::from(11u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(10u32)), Ok(tenth_point));

        // 18(5, 1) = (5, 16)
        let eighteenth_point = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(16u32), curve.p.clone()));
        assert_eq!(curve.scalar_mul(&point, BigUint::from(18u32)), Ok(eighteenth_point));

        //19(5, 1) = Point::Idnetity
        let nineteenth_point = Point::Identity;
        assert_eq!(curve.scalar_mul(&point, BigUint::from(19u32)), Ok(nineteenth_point));
    }

    #[test]
    fn test_scalar_mul_with_zero(){
        // y^2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            b: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            p: BigUint::from(17u32),
            g: Point::Coor(FiniteField::new(BigUint::from(5u32), BigUint::from(17u32)), FiniteField::new(BigUint::from(1u32), BigUint::from(17u32))),
        };

        let point = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));

        let calculated_scalar_mul_point = curve.scalar_mul(&point, BigUint::from(0u32));

        assert_eq!(calculated_scalar_mul_point, Ok(Point::Identity));
    }

    #[test]
    fn test_is_on_curve() {
        
        let curve = EllipticCurve {
            a: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            b: FiniteField::new(BigUint::from(2u32), BigUint::from(17u32)),
            p: BigUint::from(17u32),
            g: Point::Coor(FiniteField::new(BigUint::from(5u32), BigUint::from(17u32)), FiniteField::new(BigUint::from(1u32), BigUint::from(17u32))),
        };

        let on_curve_point = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(1u32), curve.p.clone()));

        assert!(curve.is_on_curve(&on_curve_point).unwrap());

        let off_curve_point = Point::Coor(FiniteField::new(BigUint::from(5u32), curve.p.clone()), FiniteField::new(BigUint::from(2u32), curve.p.clone()));

        assert!(!curve.is_on_curve(&off_curve_point).unwrap(), "Point is not on the curve");
    }
}