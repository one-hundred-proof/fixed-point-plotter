use num_bigint::BigUint;
use num_traits::ToPrimitive; // give us `to_f64`
use primitive_types::{U256};

pub struct FixedPointNumPoints {
    pub default: usize,
    pub min: usize,
    pub max: usize,
}

pub struct FixedPointBounds {
    pub radix: u8,
    pub places: u32,
    pub min: f64,
    pub max: f64
}

pub struct FixedPointFunction {
    pub name: &'static str,
    pub fun: fn(U256) -> U256,
    pub x_bounds: FixedPointBounds,
    pub y_bounds: FixedPointBounds,
    pub num_points: FixedPointNumPoints,
}

// Converts a U256 fixed-point number to f64 with `decimals` fractional digits
pub fn u256_to_f64(value: U256, radix: u8, places: u32) -> f64 {
    let factor: BigUint = BigUint::from(radix).pow(places);

    let buf = value.to_big_endian();
    let big_value = BigUint::from_bytes_be(&buf);
    let int_part = (&big_value / &factor).to_f64().unwrap_or(0.0);

    if let Some(factor_64) = factor.to_f64() {
      let frac_part = (&big_value % &factor).to_f64().unwrap_or(0.0) / factor_64;
      int_part + frac_part
    } else {
       0f64
    }
}

pub fn f64_to_u256(value: f64, radix: u8, places: u32) -> U256 {
    let factor = (radix as f64).powi(places as i32);
    let scaled = value * factor;
    // Clamp negative values to 0 since U256 can't represent them
    if scaled.is_sign_negative() {
        U256::zero()
    } else {
        U256::from(scaled as u128)
    }
}

pub fn u256d(s: &str) -> U256 {
    return U256::from_dec_str(s).unwrap();
}

pub fn u256f<T>(x: T) -> U256 where U256: From<T> {
    return U256::from(x);
}
