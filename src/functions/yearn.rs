
use primitive_types::{U256};
use crate::vyper::*;
use crate::maths::*;

pub const yearn_calc_supply: FixedPointFunction = FixedPointFunction {
    fun: yearn_calc_supply_fun,
    x_bounds: FixedPointBounds { radix: 10, places: 18, min: 0.0, max: 2.0  },
    y_bounds: FixedPointBounds { radix: 10, places: 18, min: 0.0, max: 100.0 },
    num_points: FixedPointNumPoints { default: 100, min: 5, max: 10000 },
};

fn yearn_calc_supply_fun(vb_prod: U256) -> U256 {
    let debug: bool  = false;


    let MAX_POW_REL_ERR: U256 = u256d("100");
    let PRECISION = u256d("1000000000000000000");

    let AMP    = u256d("450000000000000000000");
    let mut d  = u256d("449000000000000000000");

    // let vb_sum: U256 = u256d("10901945277009618639966");
    // let mut s: U256  = u256d("2514337702656951993513");

    // add_liquidity 1
    let vb_sum: U256 = u256d("5314420781261619946859");
    let mut s: U256  = u256d("2511236098261249777670");

    // // add_liquidity 2
    // let vb_sum: U256 = u256d("10000027134684780493830");
    // let mut s: U256  = u256d("2511654134961604379116");

    // add_liquidity 3
    // let vb_sum: U256 = u256d("10000043909621138586861");
    // let mut s: U256  = u256d("2512767327443788939269");

    let mut l  = AMP * vb_sum;
    let mut r  = vb_prod;

    for _ in 0..256 {
        // println!("l sp/loop: {l}");
        // println!("s sp/loop: {s}");
        // println!("r sp/loop: {r}");
        // println!("d sp/loop: {d}");
        let mut sp = unsafe_div(unsafe_sub(l, unsafe_mul(s,r)), d);
        // println!("sp sp/loop: {sp}");
        for i in 0..8 {
            // println!("r r/before {r}");
            r = unsafe_div(unsafe_mul(r, sp), s);
            // println!("r r/after {r}");
        }
        let mut delta = if sp >= s { unsafe_sub(sp, s) } else { unsafe_sub(s, sp)};
        if unsafe_div(unsafe_mul(delta, PRECISION), s) <= MAX_POW_REL_ERR {
            if debug { println!("vb_prod {vb_prod} , r {r}") }
            return r; // Just returning r unlike the Vyper function from the Yearn Stableswap pool
        }
        s = sp;
    }
    panic!("did not converge");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yearn_calc_supply() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(yearn_calc_supply_fun(U256::from_dec_str("3530246247551768").unwrap()),    U256::from(0));
        assert_eq!(yearn_calc_supply_fun(U256::from_dec_str("1100000001490116096").unwrap()), U256::from(666));
        Ok(())
    }

}
