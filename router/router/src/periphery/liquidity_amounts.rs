use core::ops::{Div, Shl};

use casper_types::U256;
use math::{fixed_point_96, fullmath};

pub fn to_u128(x: &U256) -> u128 {
    x.as_u128()
}

pub fn get_liquidity_for_amount0(
    sqrt_ratio_a_x96_: &U256,
    sqrt_ratio_b_x96_: &U256,
    amount0: &U256,
) -> u128 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96_ > sqrt_ratio_b_x96_ {
        (sqrt_ratio_b_x96_, sqrt_ratio_a_x96_)
    } else {
        (sqrt_ratio_a_x96_, sqrt_ratio_b_x96_)
    };

    let intermediate =
        fullmath::mul_div(sqrt_ratio_a_x96, sqrt_ratio_b_x96, &fixed_point_96::q96());
    to_u128(&fullmath::mul_div(
        amount0,
        &intermediate,
        &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96),
    ))
}

pub fn get_liquidity_for_amount1(
    sqrt_ratio_a_x96_: &U256,
    sqrt_ratio_b_x96_: &U256,
    amount1: &U256,
) -> u128 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96_ > sqrt_ratio_b_x96_ {
        (sqrt_ratio_b_x96_, sqrt_ratio_a_x96_)
    } else {
        (sqrt_ratio_a_x96_, sqrt_ratio_b_x96_)
    };

    to_u128(&fullmath::mul_div(
        &amount1,
        &fixed_point_96::q96(),
        &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96),
    ))
}

pub fn get_liquidity_for_amounts(
    sqrt_ratio_x96_: &U256,
    sqrt_ratio_a_x96_: &U256,
    sqrt_ratio_b_x96_: &U256,
    amount0: &U256,
    amount1: &U256,
) -> u128 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96_ > sqrt_ratio_b_x96_ {
        (sqrt_ratio_b_x96_, sqrt_ratio_a_x96_)
    } else {
        (sqrt_ratio_a_x96_, sqrt_ratio_b_x96_)
    };

    if sqrt_ratio_x96_ <= sqrt_ratio_a_x96 {
        get_liquidity_for_amount0(sqrt_ratio_a_x96, sqrt_ratio_b_x96, amount0)
    } else if sqrt_ratio_x96_ < sqrt_ratio_b_x96 {
        let liquidity0 = get_liquidity_for_amount0(sqrt_ratio_x96_, sqrt_ratio_b_x96, amount0);
        let liquidity1 = get_liquidity_for_amount1(sqrt_ratio_a_x96, sqrt_ratio_x96_, amount1);
        if liquidity0 < liquidity1 {
            liquidity0
        } else {
            liquidity1
        }
    } else {
        get_liquidity_for_amount1(sqrt_ratio_a_x96, sqrt_ratio_b_x96, amount1)
    }
}

pub fn get_amount0_for_liquidity(
    sqrt_ratio_a_x96_: &U256,
    sqrt_ratio_b_x96_: &U256,
    liquidity: u128,
) -> U256 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96_ > sqrt_ratio_b_x96_ {
        (sqrt_ratio_b_x96_, sqrt_ratio_a_x96_)
    } else {
        (sqrt_ratio_a_x96_, sqrt_ratio_b_x96_)
    };

    fullmath::mul_div(
        &U256::from(liquidity).shl(fixed_point_96::RESOLUTION),
        &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96),
        sqrt_ratio_b_x96,
    )
    .div(sqrt_ratio_a_x96)
}

pub fn get_amount1_for_liquidity(
    sqrt_ratio_a_x96_: &U256,
    sqrt_ratio_b_x96_: &U256,
    liquidity: u128,
) -> U256 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96_ > sqrt_ratio_b_x96_ {
        (sqrt_ratio_b_x96_, sqrt_ratio_a_x96_)
    } else {
        (sqrt_ratio_a_x96_, sqrt_ratio_b_x96_)
    };

    fullmath::mul_div(
        &U256::from(liquidity),
        &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96),
        &fixed_point_96::q96(),
    )
}

pub fn get_amounts_for_liquidity(
    sqrt_ratio_x96_: &U256,
    sqrt_ratio_a_x96_: &U256,
    sqrt_ratio_b_x96_: &U256,
    liquidity: u128,
) -> (U256, U256) {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96_ > sqrt_ratio_b_x96_ {
        (sqrt_ratio_b_x96_, sqrt_ratio_a_x96_)
    } else {
        (sqrt_ratio_a_x96_, sqrt_ratio_b_x96_)
    };

    if sqrt_ratio_x96_ <= sqrt_ratio_a_x96 {
        (
            get_amount0_for_liquidity(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity),
            0.into(),
        )
    } else if sqrt_ratio_x96_ < sqrt_ratio_b_x96 {
        let amount0 = get_amount0_for_liquidity(sqrt_ratio_x96_, sqrt_ratio_b_x96, liquidity);
        let amount1 = get_amount1_for_liquidity(sqrt_ratio_a_x96, sqrt_ratio_x96_, liquidity);
        (amount0, amount1)
    } else {
        (
            0.into(),
            get_amount1_for_liquidity(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity),
        )
    }
}
