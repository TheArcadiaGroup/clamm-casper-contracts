#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate alloc;
use super::test_math_session::*;
use casper_types::{runtime_args, RuntimeArgs, U256};
use std::ops::{Shl, Sub};

use crate::math::test_math_session::exec_test_math;

pub fn q128() -> U256 {
    U256::from_str_radix("100000000000000000000000000000000", 16).unwrap()
}

#[test]
#[should_panic]
fn muldiv_reverts_if_denominator_zero() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(5),
            "2" => U256::from(0)
        },
        5_u8,
    );
}

#[test]
#[should_panic]
fn muldiv_reverts_if_denominator_zero_numerator_overflow() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => q128(),
            "1" => q128(),
            "2" => U256::zero()
        },
        U256::zero(),
    );
}

#[test]
#[should_panic]
fn muldiv_reverts_if_output_overflow() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => q128(),
            "1" => q128(),
            "2" => U256::one()
        },
        U256::zero(),
    );
}

#[test]
fn muldiv_all_max_inputs() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => U256::MAX,
            "1" => U256::MAX,
            "2" => U256::MAX
        },
        U256::MAX,
    );
}

#[test]
fn muldiv_accurate_without_phantom_overflow() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(50) * q128() / 100,
            "2" => U256::from(150) * q128() / 100
        },
        q128() / 3,
    );
}

#[test]
fn muldiv_accurate_with_phantom_overflow() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(35) * q128(),
            "2" => U256::from(8) * q128()
        },
        q128() * U256::from(4375) / 1000,
    );
}

#[test]
fn muldiv_accurate_with_phantom_overflow_repeating_decimal() {
    exec_test_math(
        "mul_div",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(1000) * q128(),
            "2" => U256::from(3000) * q128()
        },
        q128() / 3,
    );
}

#[test]
#[should_panic]
fn muldiv_rounding_up_reverts_if_denominator_zero() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(5),
            "2" => U256::from(0)
        },
        5_u8,
    );
}

#[test]
#[should_panic]
fn muldiv_rounding_up_reverts_if_denominator_zero_numerator_overflow() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => q128(),
            "1" => q128(),
            "2" => U256::zero()
        },
        U256::zero(),
    );
}

#[test]
#[should_panic]
fn muldiv_rounding_up_reverts_if_output_overflow() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => q128(),
            "1" => q128(),
            "2" => U256::one()
        },
        U256::zero(),
    );
}

#[test]
#[should_panic]
fn muldiv_rounding_up_reverts_overflow_all_max_inputs() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => U256::MAX,
            "1" => U256::MAX,
            "2" => U256::MAX.sub(U256::one())
        },
        U256::MAX,
    );
}

#[test]
#[should_panic]
fn muldiv_rounding_up_reverts_overflow_256bits_after_rounding_up() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => U256::from_dec_str("535006138814359").unwrap(),
            "1" => U256::from_dec_str("432862656469423142931042426214547535783388063929571229938474969").unwrap(),
            "2" => U256::from(2)
        },
        U256::MAX,
    );
}

#[test]
#[should_panic]
fn muldiv_rounding_up_reverts_overflow_256bits_after_rounding_up_2() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => U256::from_dec_str("115792089237316195423570985008687907853269984659341747863450311749907997002549").unwrap(),
            "1" => U256::from_dec_str("115792089237316195423570985008687907853269984659341747863450311749907997002550").unwrap(),
            "2" => U256::from_dec_str("115792089237316195423570985008687907853269984653042931687443039491902864365164").unwrap()
        },
        U256::MAX,
    );
}

#[test]
fn muldiv_rounding_up_all_max_inputs() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => U256::MAX,
            "1" => U256::MAX,
            "2" => U256::MAX
        },
        U256::MAX,
    );
}

#[test]
fn muldiv_rounding_up_accurate_without_phantom_overflow() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(50) * q128() / 100,
            "2" => U256::from(150) * q128() / 100
        },
        q128() / 3 + U256::one(),
    );
}

#[test]
fn muldiv_rounding_up_accurate_with_phantom_overflow() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(35) * q128(),
            "2" => U256::from(8) * q128()
        },
        q128() * U256::from(4375) / 1000,
    );
}

#[test]
fn muldiv_rounding_up_accurate_with_phantom_overflow_repeating_decimal() {
    exec_test_math(
        "mul_div_rounding_up",
        runtime_args! {
            "0" => q128(),
            "1" => U256::from(1000) * q128(),
            "2" => U256::from(3000) * q128()
        },
        q128() / 3 + U256::one(),
    );
}
