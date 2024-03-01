#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate alloc;
use std::ops::Shl;

use super::test_math_session::*;
use casper_types::{runtime_args, RuntimeArgs, U256};

use crate::math::test_math_session::exec_test_math;

#[test]
#[should_panic = "User(15000)"]
fn test_most_significant_bit_0() {
    exec_test_math(
        "most_significant_bit",
        runtime_args! {
            "0" => U256::zero()
        },
        0_u8,
    );
}

#[test]
fn test_most_significant_bit_1() {
    exec_test_math(
        "most_significant_bit",
        runtime_args! {
            "0" => U256::one()
        },
        0_u8,
    );
}

#[test]
fn test_most_significant_bit_2() {
    exec_test_math(
        "most_significant_bit",
        runtime_args! {
            "0" => U256::from(2_u64)
        },
        1_u8,
    );
}

#[test]
fn test_most_significant_bit_all_powers_of_2() {
    let mut i = 0_u64;
    let mut tc = setup();
    while i < 60 {
        let input = U256::one().shl(i);
        test_math_lib(
            &mut tc,
            "most_significant_bit",
            runtime_args! {
                "0" => input
            },
            i as u8,
        );
        i += 1;
    }
}

#[test]
fn test_most_significant_bit_all_powers_of_2_2() {
    let mut i = 100_u64;
    let mut tc = setup();
    while i < 110 {
        let input = U256::one().shl(i);
        test_math_lib(
            &mut tc,
            "most_significant_bit",
            runtime_args! {
                "0" => input
            },
            i as u8,
        );
        i += 1;
    }
}

#[test]
fn test_most_significant_bit_uint256_max() {
    exec_test_math(
        "most_significant_bit",
        runtime_args! {
            "0" => U256::MAX
        },
        255_u8,
    );
}

#[test]
fn test_least_significant_bit_1() {
    exec_test_math(
        "least_significant_bit",
        runtime_args! {
            "0" => U256::one()
        },
        0_u8,
    );
}

#[test]
fn test_least_significant_bit_2() {
    exec_test_math(
        "least_significant_bit",
        runtime_args! {
            "0" => U256::from(2)
        },
        1_u8,
    );
}

#[test]
fn test_least_significant_bi_all_powers_of_2() {
    let mut i = 0_u64;
    let mut tc = setup();
    while i < 60 {
        let input = U256::one().shl(i);
        test_math_lib(
            &mut tc,
            "least_significant_bit",
            runtime_args! {
                "0" => input
            },
            i as u8,
        );
        i += 1;
    }
}

#[test]
fn test_least_significant_bi_all_powers_of_2_2() {
    let mut i = 100_u64;
    let mut tc = setup();
    while i < 110 {
        let input = U256::one().shl(i);
        test_math_lib(
            &mut tc,
            "least_significant_bit",
            runtime_args! {
                "0" => input
            },
            i as u8,
        );
        i += 1;
    }
}

#[test]
fn test_least_significant_bit_uint256_max() {
    exec_test_math(
        "least_significant_bit",
        runtime_args! {
            "0" => U256::MAX
        },
        0_u8,
    );
}
