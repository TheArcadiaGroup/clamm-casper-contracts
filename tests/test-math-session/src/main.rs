#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use alloc::{string::String, vec};
use casper_contract::{
    self,
    contract_api::{runtime, storage},
};
use casper_types::bytesrepr::FromBytes;
use casper_types::{
    bytesrepr::ToBytes, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
    RuntimeArgs,
};
use casper_types::{U128, U256};
use common::console;
use contract_utilities::helpers::encode_4;
use math::{bitmath, fullmath, liquidity_math, sqrt_price_math, swap_math, tickmath};
use types::i128::I128;
use types::i256::I256;
const TEST_SESSION: &str = "test_math_session";

fn store_result_with_name<T: CLTyped + ToBytes>(result: T, name: String) {
    match runtime::get_key(&name) {
        Some(Key::URef(uref)) => storage::write(uref, result),
        Some(_) => unreachable!(),
        None => {
            let new_uref = storage::new_uref(result);
            runtime::put_key(&name, new_uref.into());
        }
    }
}

#[no_mangle]
extern "C" fn get_any_data() {
    let func: String = runtime::get_named_arg("func");
    let return_type_name: String = runtime::get_named_arg("return_type_name");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;

    if func == "most_significant_bit" {
        let b: u8 = bitmath::most_significant_bit(args.get("0").unwrap().clone().into_t().unwrap());

        store_result_with_name(b, return_type_name);
    } else if func == "least_significant_bit" {
        let b: u8 =
            bitmath::least_significant_bit(args.get("0").unwrap().clone().into_t().unwrap());

        store_result_with_name(b, return_type_name);
    } else if func == "mul_div" {
        let b: U256 = fullmath::mul_div(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            &args.get("2").unwrap().clone().into_t().unwrap(),
        );

        store_result_with_name(b, return_type_name);
    } else if func == "mul_div_rounding_up" {
        let b: U256 = fullmath::mul_div_rounding_up(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            &args.get("2").unwrap().clone().into_t().unwrap(),
        );

        store_result_with_name(b, return_type_name);
    } else if func == "add_delta" {
        let first: U128 = args.get("0").unwrap().clone().into_t().unwrap();
        let second: I128 = args.get("1").unwrap().clone().into_t().unwrap();
        let b: u128 = liquidity_math::add_delta(first.as_u128(), second.into());

        store_result_with_name(U128::from(b), return_type_name);
    } else if func == "get_sqrt_ratio_at_tick" {
        let first: i32 = args.get("0").unwrap().clone().into_t().unwrap();
        let b: U256 = tickmath::get_sqrt_ratio_at_tick(first);

        store_result_with_name(b, return_type_name);
    } else if func == "get_tick_at_sqrt_ratio" {
        let first: U256 = args.get("0").unwrap().clone().into_t().unwrap();
        let b: i32 = tickmath::get_tick_at_sqrt_ratio(first);
        store_result_with_name(b, return_type_name);
    } else if func == "get_next_sqrt_price_from_amount0_rounding_up" {
        let b: U256 = sqrt_price_math::get_next_sqrt_price_from_amount0_rounding_up(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            args.get("1")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            &args.get("2").unwrap().clone().into_t().unwrap(),
            args.get("3").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_next_sqrt_price_from_amount1_rounding_down" {
        let b: U256 = sqrt_price_math::get_next_sqrt_price_from_amount1_rounding_down(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            args.get("1")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            &args.get("2").unwrap().clone().into_t().unwrap(),
            args.get("3").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_next_sqrt_price_from_input" {
        let b: U256 = sqrt_price_math::get_next_sqrt_price_from_input(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            args.get("1")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            &args.get("2").unwrap().clone().into_t().unwrap(),
            args.get("3").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_next_sqrt_price_from_output" {
        let b: U256 = sqrt_price_math::get_next_sqrt_price_from_output(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            args.get("1")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            &args.get("2").unwrap().clone().into_t().unwrap(),
            args.get("3").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_amount0_delta" {
        let b: U256 = sqrt_price_math::get_amount0_delta(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            args.get("2")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            args.get("3").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_amount1_delta" {
        let b: U256 = sqrt_price_math::get_amount1_delta(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            args.get("2")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            args.get("3").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_amount0_delta_2" {
        let b: I256 = sqrt_price_math::get_amount0_delta_2(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            args.get("2").unwrap().clone().into_t::<I128>().unwrap().0,
        );
        store_result_with_name(b, return_type_name);
    } else if func == "get_amount1_delta_2" {
        let b: I256 = sqrt_price_math::get_amount1_delta_2(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            args.get("2").unwrap().clone().into_t::<I128>().unwrap().0,
        );
        store_result_with_name(b, return_type_name);
    } else if func == "compute_swap_step" {
        // args.get("3").unwrap().clone().into_t::<I256>().unwrap();
        console::log("get_any_data math");
        console::log(&format!(
            "args.get.unwrap().clone() {:?}",
            args.get("3").unwrap().clone()
        ));
        let (sqrt_ratio_next_x96, amount_in, amount_out, fee_amount) = swap_math::compute_swap_step(
            &args.get("0").unwrap().clone().into_t().unwrap(),
            &args.get("1").unwrap().clone().into_t().unwrap(),
            args.get("2")
                .unwrap()
                .clone()
                .into_t::<U128>()
                .unwrap()
                .as_u128(),
            args.get("3").unwrap().clone().into_t().unwrap(),
            args.get("4").unwrap().clone().into_t().unwrap(),
        );
        store_result_with_name(
            encode_4(&sqrt_ratio_next_x96, &amount_in, &amount_out, &fee_amount),
            "compute_swap_math".into(),
        );
    }
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    let get_any_data = EntryPoint::new(
        String::from("get_any_data"),
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(get_any_data);

    let (_contract_hash, _version) = storage::new_contract(
        entry_points,
        None,
        Some(format!("{}_package_hash", TEST_SESSION)),
        None,
    );
}
