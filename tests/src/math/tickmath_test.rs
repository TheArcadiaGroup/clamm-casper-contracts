#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate alloc;
use std::ops::Shl;

use super::test_math_session::*;
use crate::math::test_math_session::exec_test_math;
use crate::utils::*;
use casper_types::{runtime_args, RuntimeArgs, U256, U512};
use math::tickmath::{self, max_sqrt_ratio, min_sqrt_ratio};

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = 887272;

#[test]
#[should_panic = "User(15008)"]
fn test_get_sqrt_ratio_at_tick_throw_too_low() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MIN_TICK - 1
        },
        U256::zero(),
    );
}

#[test]
#[should_panic = "User(15008)"]
fn test_get_sqrt_ratio_at_tick_throw_too_high() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MAX_TICK + 1
        },
        U256::zero(),
    );
}

#[test]
fn test_get_sqrt_ratio_at_tick_min_tick() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MIN_TICK
        },
        U256::from(4295128739_u128),
    );
}

#[test]
fn test_get_sqrt_ratio_at_tick_min_tick_plus_one() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MIN_TICK + 1
        },
        U256::from(4295343490_u128),
    );
}

#[test]
fn test_get_sqrt_ratio_at_tick_max_tick_minus_one() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MAX_TICK - 1
        },
        U256::from_dec_str("1461373636630004318706518188784493106690254656249").unwrap(),
    );
}

#[test]
fn test_get_sqrt_ratio_at_tick_min_tick_ratio_is_less_than_js_implementation() {
    let ret: U256 = get_math_lib_with_tc(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MIN_TICK
        },
    );
    assert!(ret < encode_price_sqrt(1, 1_u128 << 127));
}

#[test]
fn test_get_sqrt_ratio_at_tick_min_tick_ratio_is_greater_than_js_implementation() {
    let ret: U256 = get_math_lib_with_tc(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MAX_TICK
        },
    );
    assert!(ret > encode_price_sqrt(1_u128 << 127, 1));
}

#[test]
fn test_get_sqrt_ratio_at_tick_max_tick() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MAX_TICK
        },
        U256::from_dec_str("1461446703485210103287273052203988822378723970342").unwrap(),
    );
}

#[test]
fn test_get_sqrt_ratio_at_tick_tick() {
    let _abs_ticks = [
        50, 100, 250, 500, 1_000, 2_500, 3_000, 4_000, 5_000, 50_000, 150_000, 250_000, 500_000,
        738_203,
    ];
    let _tc = &mut setup();
    for _abs_tick in _abs_ticks {
        // TODO: complete test
        // let js_result =
    }
}

#[test]
fn test_get_sqrt_ratio_at_tick_equal_min_tick() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MIN_TICK
        },
        tickmath::min_sqrt_ratio(),
    );
}

#[test]
fn test_get_sqrt_ratio_at_tick_equal_max_tick() {
    exec_test_math(
        "get_sqrt_ratio_at_tick",
        runtime_args! {
            "0" => MAX_TICK
        },
        tickmath::max_sqrt_ratio(),
    );
}

#[test]
#[should_panic = "User(15009)"]
fn test_get_tick_at_sqrt_ratio_throw_too_low() {
    exec_test_math(
        "get_tick_at_sqrt_ratio",
        runtime_args! {
            "0" => min_sqrt_ratio() - U256::one()
        },
        1,
    );
}

#[test]
#[should_panic = "User(15009)"]
fn test_get_tick_at_sqrt_ratio_throw_too_high() {
    exec_test_math(
        "get_tick_at_sqrt_ratio",
        runtime_args! {
            "0" => max_sqrt_ratio()
        },
        1_i32,
    );
}

#[test]
fn test_get_tick_at_sqrt_ratio_of_min_tick() {
    exec_test_math(
        "get_tick_at_sqrt_ratio",
        runtime_args! {
            "0" => min_sqrt_ratio()
        },
        MIN_TICK,
    );
}

#[test]
fn test_get_tick_at_sqrt_ratio_of_min_tick_plus_one() {
    exec_test_math(
        "get_tick_at_sqrt_ratio",
        runtime_args! {
            "0" => U256::from(4295343490_u64)
        },
        MIN_TICK + 1,
    );
}

#[test]
fn test_get_tick_at_sqrt_ratio_of_max_tick_minus_one() {
    exec_test_math(
        "get_tick_at_sqrt_ratio",
        runtime_args! {
            "0" => U256::from_dec_str("1461373636630004318706518188784493106690254656249").unwrap()
        },
        MAX_TICK - 1,
    );
}

#[test]
fn test_get_tick_at_sqrt_ratio_closest_to_max_tick() {
    exec_test_math(
        "get_tick_at_sqrt_ratio",
        runtime_args! {
            "0" => max_sqrt_ratio() - U256::one()
        },
        MAX_TICK - 1,
    );
}
