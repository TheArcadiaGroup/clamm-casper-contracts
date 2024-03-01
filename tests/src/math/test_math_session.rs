use std::any::type_name;
use std::fmt::Debug;

use crate::constants;
use crate::utils::{self, alice, bob, call_and_get_any, eve, owner};
use casper_engine_test_support::InMemoryWasmTestBuilder;
use casper_types::bytesrepr::Bytes;
use casper_types::{bytesrepr::FromBytes, runtime_args, CLTyped, Key, RuntimeArgs};
use casper_types::{U128, U256};
use contract_utilities::helpers;
use types::i256::I256;

pub struct TestMathContext {
    test_session: Key,
    builder: InMemoryWasmTestBuilder,
}

pub fn setup() -> TestMathContext {
    let mut builder = utils::initialize_builder();
    utils::fund_accounts(&mut builder, &[owner(), alice(), bob(), eve()]);
    utils::deploy_contract(
        &mut builder,
        owner(),
        constants::TEST_MATH_SESSION,
        runtime_args! {},
    );

    let test_session =
        utils::get_contract_package_hash(&builder, owner(), "test_math_session".to_string());
    TestMathContext {
        test_session,
        builder,
    }
}

pub fn get_math_lib<T: FromBytes + CLTyped + PartialEq + Debug>(
    tc: &mut TestMathContext,
    func: &str,
    args: RuntimeArgs,
) -> T {
    let qualified_type_name: String = type_name::<T>().to_string();
    let name: String = qualified_type_name.split("::").last().unwrap().to_string();
    call_and_get_any(
        &mut tc.builder,
        tc.test_session.into_hash().unwrap().into(),
        func,
        &name,
        tc.test_session,
        args,
    )
}

#[allow(dead_code)]
pub fn get_math_lib_with_tc<T: FromBytes + CLTyped + PartialEq + Debug>(
    func: &str,
    args: RuntimeArgs,
) -> T {
    let tc = &mut setup();
    let qualified_type_name: String = type_name::<T>().to_string();
    let name: String = qualified_type_name.split("::").last().unwrap().to_string();
    call_and_get_any(
        &mut tc.builder,
        tc.test_session.into_hash().unwrap().into(),
        func,
        &name,
        tc.test_session,
        args,
    )
}

#[allow(dead_code)]
pub fn get_swap_math_result(
    sqrt_ratio_current_x96: U256,
    sqrt_ratio_target_x96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u64,
) -> (U256, U256, U256, U256) {
    let tc = &mut setup();
    let ret = call_and_get_any::<Bytes>(
        &mut tc.builder,
        tc.test_session.into_hash().unwrap().into(),
        "compute_swap_step",
        "compute_swap_math",
        tc.test_session,
        runtime_args! {
            "0" => sqrt_ratio_current_x96,
            "1" => sqrt_ratio_target_x96,
            "2" => U128::from(liquidity),
            "3" => amount_remaining,
            "4" => fee_pips,
        },
    );
    helpers::decode_4(&ret)
}

pub fn test_math_lib<T: FromBytes + CLTyped + PartialEq + Debug>(
    tc: &mut TestMathContext,
    func: &str,
    args: RuntimeArgs,
    expected: T,
) {
    let ret = get_math_lib::<T>(tc, func, args.clone());
    if ret != expected {
        println!(
            "get_math_lib::<T>(tc, func, args): {:?}, {:?}",
            ret, expected
        );
    }
    assert!(get_math_lib::<T>(tc, func, args) == expected);
}

pub fn exec_test_math<T: FromBytes + CLTyped + PartialEq + Debug>(
    func: &str,
    args: RuntimeArgs,
    expected: T,
) {
    test_math_lib(&mut setup(), func, args, expected);
}
