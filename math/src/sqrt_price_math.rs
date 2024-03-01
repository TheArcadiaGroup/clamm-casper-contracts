use core::ops::{Add, Shl};

use casper_types::U256;
use common::{console, error::require};

use crate::{fixed_point_96, fullmath, safe_cast};
use types::i256::I256;

pub fn get_next_sqrt_price_from_amount0_rounding_up(
    sqrt_p_x96: &U256,
    liquidity: u128,
    amount: &U256,
    add: bool,
) -> U256 {
    if amount == &U256::zero() {
        return *sqrt_p_x96;
    }

    let numerator1 = &U256::from(liquidity).shl(fixed_point_96::RESOLUTION);
    if add {
        let product = amount.overflowing_mul(*sqrt_p_x96).0;
        if product / amount == *sqrt_p_x96 {
            let denominator = &numerator1.overflowing_add(product).0;
            if denominator >= numerator1 {
                console::log("hehe");
                return safe_cast::to_u160(fullmath::mul_div_rounding_up(
                    numerator1,
                    sqrt_p_x96,
                    denominator,
                ));
            }
        }
        safe_cast::to_u160(fullmath::unsafe_div_rounding_up(
            numerator1,
            &(numerator1 / sqrt_p_x96 + amount),
        ))
    } else {
        let product = amount.overflowing_mul(*sqrt_p_x96).0;
        require(
            product / amount == *sqrt_p_x96 && numerator1 > &product,
            common::error::Error::SqrtRatioInternalError,
        );
        let denominator = numerator1 - product;
        safe_cast::to_u160(fullmath::mul_div_rounding_up(
            numerator1,
            sqrt_p_x96,
            &denominator,
        ))
    }
}

pub fn get_next_sqrt_price_from_amount1_rounding_down(
    sqrt_p_x96: &U256,
    liquidity: u128,
    amount: &U256,
    add: bool,
) -> U256 {
    let max_u160 = U256::one().shl(160) - U256::one();
    if add {
        let quotient = if amount <= &max_u160 {
            amount.shl(fixed_point_96::RESOLUTION) / U256::from(liquidity)
        } else {
            fullmath::mul_div(amount, &fixed_point_96::q96(), &liquidity.into())
        };
        safe_cast::to_u160(sqrt_p_x96.add(quotient))
    } else {
        let quotient = if amount <= &max_u160 {
            fullmath::unsafe_div_rounding_up(
                &amount.shl(fixed_point_96::RESOLUTION),
                &liquidity.into(),
            )
        } else {
            fullmath::mul_div_rounding_up(amount, &fixed_point_96::q96(), &U256::from(liquidity))
        };
        require(
            sqrt_p_x96 > &quotient,
            common::error::Error::SqrtRatioInternalError,
        );
        safe_cast::to_u160(sqrt_p_x96 - quotient)
    }
}

pub fn get_next_sqrt_price_from_input(
    sqrt_p_x96: &U256,
    liquidity: u128,
    amount_in: &U256,
    zero_for_one: bool,
) -> U256 {
    require(
        sqrt_p_x96 > &U256::zero(),
        common::error::Error::SqrtRatioInternalError,
    );
    require(liquidity > 0, common::error::Error::SqrtRatioInternalError);
    if zero_for_one {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_p_x96, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_p_x96, liquidity, amount_in, true)
    }
}

pub fn get_next_sqrt_price_from_output(
    sqrt_p_x96: &U256,
    liquidity: u128,
    amount_out: &U256,
    zero_for_one: bool,
) -> U256 {
    require(
        sqrt_p_x96 > &U256::zero(),
        common::error::Error::SqrtRatioInternalError,
    );
    require(liquidity > 0, common::error::Error::SqrtRatioInternalError);
    if zero_for_one {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_p_x96, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_p_x96, liquidity, amount_out, false)
    }
}

pub fn get_amount0_delta(
    sqrt_ratio_0_x96: &U256,
    sqrt_ratio_1_x96: &U256,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_0_x96 > sqrt_ratio_1_x96 {
        (sqrt_ratio_1_x96, sqrt_ratio_0_x96)
    } else {
        (sqrt_ratio_0_x96, sqrt_ratio_1_x96)
    };

    let numerator1 = &U256::from(liquidity).shl(fixed_point_96::RESOLUTION);
    let numerator2 = &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96);
    require(
        sqrt_ratio_a_x96 > &U256::zero(),
        common::error::Error::SqrtRatioInternalError,
    );
    if round_up {
        fullmath::unsafe_div_rounding_up(
            &fullmath::mul_div_rounding_up(numerator1, numerator2, sqrt_ratio_b_x96),
            sqrt_ratio_a_x96,
        )
    } else {
        fullmath::mul_div(numerator1, numerator2, sqrt_ratio_b_x96) / sqrt_ratio_a_x96
    }
}

pub fn get_amount1_delta(
    sqrt_ratio_0_x96: &U256,
    sqrt_ratio_1_x96: &U256,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_0_x96 > sqrt_ratio_1_x96 {
        (sqrt_ratio_1_x96, sqrt_ratio_0_x96)
    } else {
        (sqrt_ratio_0_x96, sqrt_ratio_1_x96)
    };

    if round_up {
        fullmath::mul_div_rounding_up(
            &liquidity.into(),
            &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96),
            &fixed_point_96::q96(),
        )
    } else {
        fullmath::mul_div(
            &liquidity.into(),
            &(sqrt_ratio_b_x96 - sqrt_ratio_a_x96),
            &fixed_point_96::q96(),
        )
    }
}

pub fn get_amount0_delta_2(
    sqrt_ratio_0_x96: &U256,
    sqrt_ratio_1_x96: &U256,
    liquidity: i128,
) -> I256 {
    if liquidity < 0 {
        -I256::from(get_amount0_delta(
            sqrt_ratio_0_x96,
            sqrt_ratio_1_x96,
            liquidity.unsigned_abs(),
            false,
        ))
    } else {
        I256::from(get_amount0_delta(
            sqrt_ratio_0_x96,
            sqrt_ratio_1_x96,
            liquidity as u128,
            true,
        ))
    }
}

pub fn get_amount1_delta_2(
    sqrt_ratio_0_x96: &U256,
    sqrt_ratio_1_x96: &U256,
    liquidity: i128,
) -> I256 {
    if liquidity < 0 {
        -I256::from(get_amount1_delta(
            sqrt_ratio_0_x96,
            sqrt_ratio_1_x96,
            liquidity.unsigned_abs(),
            false,
        ))
    } else {
        I256::from(get_amount1_delta(
            sqrt_ratio_0_x96,
            sqrt_ratio_1_x96,
            liquidity as u128,
            true,
        ))
    }
}
