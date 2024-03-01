#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;

use alloc::vec::Vec;
use casper_contract::{self, contract_api::runtime};
use casper_types::bytesrepr::{Bytes, ToBytes};
use casper_types::{runtime_args, Key, RuntimeArgs, U256, U512};
use common::erc20_helpers;
use common::error::require;
use common::utils::{new_purse, u256_to_u512, unwrap_wcspr, wrap_cspr};
use contract_utilities::helpers::{self, get_self_key};
use types::{ExactInputParams, ExactInputSingleParams, ExactOutputParams, ExactOutputSingleParams};

fn try_wrap_cspr() {
    let amount: U512 = runtime::get_named_arg("amount");
    wrap_cspr(get_wcspr(), new_purse(amount), amount);
}

fn get_wcspr() -> Key {
    runtime::get_named_arg("wcspr")
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points: Vec<String> = runtime::get_named_arg("entry_points");
    let datas: Vec<Bytes> = runtime::get_named_arg("datas");
    require(
        entry_points.len() == datas.len(),
        common::error::Error::ErrInvalidLiquiditySessionParams,
    );
    let router: Key = runtime::get_named_arg("router");

    for i in 0..entry_points.len() {
        let entry_point = &entry_points[i];
        let data = &datas[i];
        if entry_point == "exact_input_single" {
            let (
                token_in,
                token_out,
                is_cspr_in,
                fee,
                recipient,
                amount_in,
                amount_out_minimum,
                sqrt_price_limit_x96,
                deadline,
            ): (Key, Key, bool, u32, Key, U256, U256, U256, u64) = helpers::decode_9(data);
            if is_cspr_in {
                try_wrap_cspr();
            }
            erc20_helpers::approve(token_in, router, amount_in);
            runtime::call_versioned_contract::<U256>(
                router.into_hash().unwrap().into(),
                None,
                "exact_input_single",
                runtime_args! {
                    "data" => Bytes::from(ExactInputSingleParams { token_in, token_out, fee, recipient, deadline, amount_in, amount_out_minimum, sqrt_price_limit_x96 }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "exact_input" {
            let (token_in, path, is_cspr, recipient, amount_in, amount_out_minimum, deadline): (
                Key,
                Bytes,
                bool,
                Key,
                U256,
                U256,
                u64,
            ) = helpers::decode_7(data);
            erc20_helpers::approve(token_in, router, amount_in);
            if is_cspr {
                try_wrap_cspr();
            }
            runtime::call_versioned_contract::<U256>(
                router.into_hash().unwrap().into(),
                None,
                "exact_input",
                runtime_args! {
                    "data" => Bytes::from(ExactInputParams { path, recipient, deadline, amount_in, amount_out_minimum }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "exact_output_single" {
            let (
                token_in,
                token_out,
                is_cspr,
                fee,
                recipient,
                amount_out,
                amount_in_maximum,
                sqrt_price_limit_x96,
                deadline,
            ): (Key, Key, bool, u32, Key, U256, U256, U256, u64) = helpers::decode_9(data);
            erc20_helpers::approve(token_in, router, amount_in_maximum);
            if is_cspr {
                try_wrap_cspr();
            }
            runtime::call_versioned_contract::<U256>(
                router.into_hash().unwrap().into(),
                None,
                "exact_output_single",
                runtime_args! {
                    "data" => Bytes::from(ExactOutputSingleParams { token_in, token_out, fee, recipient, deadline, amount_out, amount_in_maximum, sqrt_price_limit_x96 }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "exact_output" {
            let (token_in, path, is_cspr, recipient, amount_out, amount_in_maximum, deadline): (
                Key,
                Bytes,
                bool,
                Key,
                U256,
                U256,
                u64,
            ) = helpers::decode_7(data);
            erc20_helpers::approve(token_in, router, amount_in_maximum);
            if is_cspr {
                try_wrap_cspr();
            }
            runtime::call_versioned_contract::<U256>(
                router.into_hash().unwrap().into(),
                None,
                "exact_output",
                runtime_args! {
                    "data" => Bytes::from(ExactOutputParams { path, recipient, deadline, amount_out, amount_in_maximum }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "unwrap_cspr" {
            let (amount, recipient): (U256, Key) = helpers::decode_2(data);
            let wcspr = get_wcspr();
            let wcspr_balance = erc20_helpers::get_balance(wcspr, get_self_key());
            require(
                wcspr_balance >= amount,
                common::error::Error::ErrInsufficientBalanceWCSPR,
            );
            if wcspr_balance > 0.into() {
                unwrap_wcspr(wcspr, recipient, u256_to_u512(wcspr_balance));
            }
        }
    }
}
