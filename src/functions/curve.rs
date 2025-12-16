use primitive_types::{U256};
use crate::vyper::*;
use crate::maths::*;

pub const curve_get_D: FixedPointFunction = FixedPointFunction {
    name: "curve_get_D",
    fun: curve_get_D_fun,
    x_bounds: FixedPointBounds { radix: 10, places: 18, min: 0.0, max: 1.0  },
    y_bounds: FixedPointBounds { radix: 10, places: 18, min: 0.0, max: 1.0 },
    num_points: FixedPointNumPoints { default: 5000, min: 100, max: 10000 },
};

fn curve_get_D_fun(x_n: U256) -> U256 {
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