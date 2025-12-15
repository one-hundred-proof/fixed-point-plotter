use primitive_types::{U256, U512};

use crate::maths::*;


/*
 *   The function to be plotted
 */
pub fn loss_of_rewards(x: U256) -> U256 {
    if x < U256::pow(U256::from(2), U256::from(150)) {
        let growth_inside = to_X128(0.5);
        let last_liquidity = U256::from(1);
        let new_liquidity = x * last_liquidity + 1;
        let last_growth_inside_x128 = U256::from(0);

        let last_growth_adjustment =
            full_mul_div(growth_inside - last_growth_inside_x128, last_liquidity, new_liquidity);
        let last_growth_inside_x128_1 = growth_inside - last_growth_adjustment;


        return (growth_inside - last_growth_inside_x128) * last_liquidity - (growth_inside - last_growth_inside_x128_1) * new_liquidity;
    } else {
        return U256::from(0);
    }

}

/*
 *  Converts a floating point number to Uniswap's X128 format (Q128.128)
 */
pub fn to_X128(x: f64) -> U256 {
    f64_to_u256(x,2,128)
}

/*
 *  Converts a floating point number to Uniswap's X128 format (Q64.96)
 */
pub fn to_X96(x: f64) -> U256 {
    f64_to_u256(x,2,96)
}


pub fn x_mul_inverse(x: U256) -> U256 {
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


fn u256d(s: &str) -> U256 {
    return U256::from_dec_str(s).unwrap();
}

fn u256f<T>(x: T) -> U256 where U256: From<T> {
    return U256::from(x);
}

fn unsafe_div(a: U256, b: U256) -> U256 {
    if b == U256::from(0) { return b } ;
    return a / b;
}

fn unsafe_sub(a: U256, b: U256) -> U256 {
    return a.overflowing_sub(b).0;
}

fn unsafe_add(a: U256, b: U256) -> U256 {
    return a.overflowing_add(b).0;
}

fn unsafe_mul(a: U256, b: U256) -> U256 {
    return a.overflowing_mul(b).0;
}

pub fn yearn_calc_supply(vb_prod: U256) -> U256 {
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
    if debug { println!("Did not converge. vb_prod = {vb_prod} , r = {r}") }
    return u256d("666"); // special value
}

pub fn curve_get_D(x_n: U256) -> U256 {
    if x_n == u256d("0") || x_n >= u256d("340282366920938463463374607431768211455") { return x_n; }
    let _amp = u256d("30000");
    let A_PRECISION = u256d("100");

    let mut _xp = [ f64_to_u256(1.0, 10,20), u256d("10000000") ].to_vec();
    _xp.push(x_n);

    // println!("{:?}", _xp);

    let N_COINS = U256::from(_xp.len());
    let mut S = u256d("0");
    for &x in &_xp {
        S += x;
    }
    if S == u256d("0") { return S; }

    let mut D = S;
    let Ann = _amp * N_COINS;

    for i in 0..256 {
        let mut D_P = D;
        for &x in &_xp {
            D_P = D_P * D / x;
        }
        D_P /= U256::pow(N_COINS, N_COINS);
        let Dprev = D;

        // println!("D {:?}", D);
        // println!("D_P {:?}", D_P);

        // println!("num    {:?}",  (unsafe_div(Ann * S, A_PRECISION) + D_P * N_COINS) * D);
        // println!("denom  {:?}",


        D = (
            (unsafe_div(Ann * S, A_PRECISION) + D_P * N_COINS) * D
            /
            (
                unsafe_div((Ann - A_PRECISION) * D, A_PRECISION) +
                unsafe_add(N_COINS, u256d("1")) * D_P
            )
        );

        if D > Dprev {
            if D - Dprev <= u256d("1") {
                // println!("D {:?}", D);
                return D;
            }
        } else {
            if Dprev - D <= u256d("1") {
                // println!("D {:?}", D);
                return D;
            }
        }
    }

    return u256d("100000000000000000"); // special error value
}

fn curve_get_y_D(D: U256) -> U256 {
    let AMP = u256d("30000");
    let A_PRECISION = u256d("100");

    let mut xp = [ u256d("1000000000000000000"), u256d("10000"), u256d("100000000000000000000") ].to_vec();
    let i = 0;

    let mut S_ = u256d("0");
    let _x = u256d("0");
    let mut y_prev = u256d("0");
    let mut c = D;
    let N_COINS = u256f(xp.len());
    let Ann = AMP * N_COINS;


    for _i in 0..xp.len() {
        let mut _x;
        if _i != i {
            _x = xp[i];
        } else {
            continue;
        }
        S_ += _x;
    }
    c = c * D * A_PRECISION / (Ann * N_COINS);
    let b = S_ + D * A_PRECISION / Ann;
    let mut y = D;

    for _i in 0..256 {
        y_prev = y;
        y = (y * y + c) / (u256f(2) * y + b - D);
        if y > y_prev {
            if y - y_prev <= u256f(1) { return y; }
        } else {
            if y_prev - y <= u256f(1) { return y; }
        }
    }

    return u256d("100000000000000000"); // 1e17

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_math_div() -> Result<(), Box<dyn std::error::Error>> {
        let max_256_str = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        let max_256 = U256::from_str_radix(max_256_str, 10)?;
        assert_eq!(full_mul_div(max_256, max_256, max_256), max_256);

        let x: U256 = U256::pow(U256::from(2), U256::from(129)) + 30;
        let y: U256 = U256::pow(U256::from(2), U256::from(135)) + 456;
        let z: U256 = U256::pow(U256::from(2), U256::from(50)) - 12345;

        assert_eq!(full_mul_div(x,y,z), U256::from_str_radix("26328072917427972477888272069236239697031062744045588567990936851", 10)?);

        Ok(())
    }

    // #[test]
    // fn test_plot_fun() -> Result<(), Box<dyn std::error::Error>> {

    //     assert_eq!(plot_fun(U256::from_str_radix("10", 10)?),                                U256::from(3));
    //     assert_eq!(plot_fun(U256::from_str_radix("100", 10)?),                               U256::from(80));
    //     assert_eq!(plot_fun(U256::from_str_radix("1000", 10)?),                              U256::from(256));
    //     assert_eq!(plot_fun(U256::from_str_radix("10000", 10)?),                             U256::from(2778));
    //     assert_eq!(plot_fun(U256::from_str_radix("100000", 10)?),                            U256::from(62648));
    //     assert_eq!(plot_fun(U256::from_str_radix("1000000", 10)?),                           U256::from(329744));

    //     assert_eq!(plot_fun(U256::pow(U256::from(2), U256::from(127))),
    //                U256::from_str_radix("170141183460469231731687303715884105727", 10)?);


    //     Ok(())
    // }

    #[test]
    fn test_yearn_calc_supply() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(yearn_calc_supply(U256::from_dec_str("3530246247551768").unwrap()),    U256::from(0));
        assert_eq!(yearn_calc_supply(U256::from_dec_str("1100000001490116096").unwrap()), U256::from(666));
        Ok(())
    }

}
