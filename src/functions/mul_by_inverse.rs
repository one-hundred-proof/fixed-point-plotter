use primitive_types::{U256, U512};
use crate::maths::*;

pub fn x_mul_inverse_fun(x: U256) -> U256 {
    if x > u256d("500000000000000000000000000000000000") {
        panic!("error");
    }

    if x > u256f(0) {
        mul(x, div(U256::from(10u128.pow(18)), x))
    } else {
        u256f(0)
    }
}

fn mul(x: U256, y: U256) -> U256 {
    x * y / U256::from(10u128.pow(18))
}

fn div(x: U256, y: U256) -> U256 {
    x * U256::from(10u128.pow(18))/ y
}

/*
 * Uses 512-bit arithmetic in intermediate calculations to retain precision
 */
fn full_mul_div(x: U256, y: U256, z: U256) -> U256 {
    let x_512: U512 = U512::from(x);
    let y_512: U512 = U512::from(y);
    let z_512: U512 = U512::from(z);
    return U256::from_little_endian( &((x_512 * y_512) / z_512).to_little_endian()[0..32]);
}

pub const x_mul_inverse: FixedPointFunction = FixedPointFunction {
    fun: x_mul_inverse_fun,
    x_bounds: FixedPointBounds { radix: 10, places: 18, min: 0.0, max: 1.0  },
    y_bounds: FixedPointBounds { radix: 10, places: 18, min: 0.0, max: 1e18 },
    num_points: FixedPointNumPoints { default: 5000, min: 100, max: 10000 },
};

// /*
//  *   The function to be plotted
//  */
// pub fn loss_of_rewards(x: U256) -> U256 {
//     if x < U256::pow(U256::from(2), U256::from(150)) {
//         let growth_inside = to_X128(0.5);
//         let last_liquidity = U256::from(1);
//         let new_liquidity = x * last_liquidity + 1;
//         let last_growth_inside_x128 = U256::from(0);

//         let last_growth_adjustment =
//             full_mul_div(growth_inside - last_growth_inside_x128, last_liquidity, new_liquidity);
//         let last_growth_inside_x128_1 = growth_inside - last_growth_adjustment;


//         return (growth_inside - last_growth_inside_x128) * last_liquidity - (growth_inside - last_growth_inside_x128_1) * new_liquidity;
//     } else {
//         return U256::from(0);
//     }

// }

// /*
//  *  Converts a floating point number to Uniswap's X128 format (Q128.128)
//  */
// pub fn to_X128(x: f64) -> U256 {
//     f64_to_u256(x,2,128)
// }

// /*
//  *  Converts a floating point number to Uniswap's X128 format (Q64.96)
//  */
// pub fn to_X96(x: f64) -> U256 {
//     f64_to_u256(x,2,96)
// }



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_math_div() -> Result<(), Box<dyn std::error::Error>> {
        let max_256_str = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        let max_256 = u256d(max_256_str);
        assert_eq!(full_mul_div(max_256, max_256, max_256), max_256);

        let x: U256 = U256::pow(U256::from(2), U256::from(129)) + 30;
        let y: U256 = U256::pow(U256::from(2), U256::from(135)) + 456;
        let z: U256 = U256::pow(U256::from(2), U256::from(50)) - 12345;

        assert_eq!(full_mul_div(x,y,z), u256d("26328072917427972477888272069236239697031062744045588567990936851"));

        Ok(())
    }
}