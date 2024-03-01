#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate alloc;
use super::test_math_session::*;
use crate::math::test_math_session::exec_test_math;
use casper_types::{runtime_args, RuntimeArgs, U128, U256};
use math::{self};
use std::ops::{Shl, Sub};
use types::i128::I128;

#[test]
fn test_no_reverts() {
    exec_test_math(
        "add_delta",
        runtime_args! {
            "0" => U128::one(),
            "1" => I128::from(0_i128)
        },
        U128::one(),
    );

    exec_test_math(
        "add_delta",
        runtime_args! {
            "0" => U128::one(),
            "1" => I128::from(-1_i128)
        },
        U128::zero(),
    );

    exec_test_math(
        "add_delta",
        runtime_args! {
            "0" => U128::one(),
            "1" => I128::from(1_i128)
        },
        U128::from(2),
    );
}

#[test]
#[should_panic = "User(15007)"]
fn test_revert_overflow() {
    exec_test_math(
        "add_delta",
        runtime_args! {
            "0" => U128::from(U256::from(2).pow(128.into()).sub(U256::from(15)).as_u128()),
            "1" => I128::from(15_i128)
        },
        U128::from(2),
    );
}

#[test]
#[should_panic = "User(15006)"]
fn test_revert_underflow() {
    exec_test_math(
        "add_delta",
        runtime_args! {
            "0" => U128::from(0),
            "1" => I128::from(-1_i128)
        },
        U128::from(2),
    );
}

#[test]
#[should_panic = "User(15006)"]
fn test_revert_underflow_2() {
    exec_test_math(
        "add_delta",
        runtime_args! {
            "0" => U128::from(3),
            "1" => I128::from(-4_i128)
        },
        U128::from(2),
    );
}
