use primitive_types::{U256};

pub fn unsafe_div(a: U256, b: U256) -> U256 {
    if b == U256::from(0) { return b } ;
    return a / b;
}

pub fn unsafe_sub(a: U256, b: U256) -> U256 {
    return a.overflowing_sub(b).0;
}

pub fn unsafe_add(a: U256, b: U256) -> U256 {
    return a.overflowing_add(b).0;
}

pub fn unsafe_mul(a: U256, b: U256) -> U256 {
    return a.overflowing_mul(b).0;
}
