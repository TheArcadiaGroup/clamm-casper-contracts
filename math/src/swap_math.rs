use alloc::format;
use casper_types::U256;
use common::console;

use crate::{fullmath, sqrt_price_math};
use types::i256::I256;

pub fn compute_swap_step(
    sqrt_ratio_current_x96: &U256,
    sqrt_ratio_target_x96: &U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u64,
) -> (U256, U256, U256, U256) {
    let zero_for_one = sqrt_ratio_current_x96 >= sqrt_ratio_target_x96;
    let exact_in = amount_remaining >= I256::from(0);
    let sqrt_ratio_next_x96;
    let mut amount_in = Default::default();
    let mut amount_out = Default::default();
    console::log(&format!(
        "compute_swap_step exact_in {:?}, {:?}",
        exact_in, amount_remaining
    ));
    if exact_in {
        let amount_remaining_less_fee = &fullmath::mul_div(
            &amount_remaining.0.as_u128().into(),
            &(1e6 as u64 - fee_pips).into(),
            &(1e6 as u64).into(),
        );
        amount_in = if zero_for_one {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_target_x96,
                sqrt_ratio_current_x96,
                liquidity,
                true,
            )
        } else {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_current_x96,
                sqrt_ratio_target_x96,
                liquidity,
                true,
            )
        };
        sqrt_ratio_next_x96 = if amount_remaining_less_fee >= &amount_in {
            console::log("compute_swap_step 0");
            *sqrt_ratio_target_x96
        } else {
            console::log("compute_swap_step 1");
            sqrt_price_math::get_next_sqrt_price_from_input(
                sqrt_ratio_current_x96,
                liquidity,
                amount_remaining_less_fee,
                zero_for_one,
            )
        };
    } else {
        amount_out = if zero_for_one {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_target_x96,
                sqrt_ratio_current_x96,
                liquidity,
                false,
            )
        } else {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_current_x96,
                sqrt_ratio_target_x96,
                liquidity,
                false,
            )
        };
        sqrt_ratio_next_x96 = if U256::from(-amount_remaining) >= amount_out {
            *sqrt_ratio_target_x96
        } else {
            sqrt_price_math::get_next_sqrt_price_from_output(
                sqrt_ratio_current_x96,
                liquidity,
                &U256::from(-amount_remaining),
                zero_for_one,
            )
        };
    }
    console::log(&format!("compute_swap_step {:?}", sqrt_ratio_next_x96));
    let max = *sqrt_ratio_target_x96 == sqrt_ratio_next_x96;
    if zero_for_one {
        amount_in = if max && exact_in {
            amount_in
        } else {
            sqrt_price_math::get_amount0_delta(
                &sqrt_ratio_next_x96,
                sqrt_ratio_current_x96,
                liquidity,
                true,
            )
        };
        amount_out = if max && !exact_in {
            amount_out
        } else {
            sqrt_price_math::get_amount1_delta(
                &sqrt_ratio_next_x96,
                sqrt_ratio_current_x96,
                liquidity,
                false,
            )
        };
    } else {
        amount_in = if max && exact_in {
            amount_in
        } else {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_current_x96,
                &sqrt_ratio_next_x96,
                liquidity,
                true,
            )
        };
        amount_out = if max && !exact_in {
            amount_out
        } else {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_current_x96,
                &sqrt_ratio_next_x96,
                liquidity,
                false,
            )
        };
    }

    if !exact_in && amount_out >= U256::from(-amount_remaining) {
        amount_out = U256::from(-amount_remaining);
    }

    let fee_amount = if exact_in && sqrt_ratio_next_x96 != *sqrt_ratio_target_x96 {
        U256::from(amount_remaining) - amount_in
    } else {
        fullmath::mul_div_rounding_up(
            &amount_in,
            &fee_pips.into(),
            &((1e6 as u64) - fee_pips).into(),
        )
    };
    (sqrt_ratio_next_x96, amount_in, amount_out, fee_amount)
}
