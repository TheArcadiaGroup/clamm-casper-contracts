#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;

use alloc::vec::Vec;
use casper_contract::contract_api::account;
use casper_contract::{self, contract_api::runtime};
use casper_types::bytesrepr::{Bytes, ToBytes};
use casper_types::{runtime_args, Key, RuntimeArgs, U128, U256, U512};
use common::erc20_helpers;
use common::error::require;
use common::utils::{new_purse, u256_to_u512, unwrap_wcspr, unwrap_wcspr_to_purse, wrap_cspr};
use contract_utilities::helpers::{self, get_self_key};
use types::{
    CollectParams, CollectResult, DecreaseLiquidityParams, DecreaseLiquidityResult,
    IncreaseLiquidityParams, IncreaseLiquidityResult, MintParams, MintResult,
};

fn try_wrap_cspr() {
    let amount: U512 = runtime::get_named_arg("amount");
    wrap_cspr(get_wcspr(), new_purse(amount), amount);
}

fn get_wcspr() -> Key {
    runtime::get_named_arg("wcspr")
}

fn try_unwrap_cspr(wcspr: Key, amount: U512) {
    unwrap_wcspr_to_purse(wcspr, account::get_main_purse(), amount);
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
        if entry_point == "create_and_initialize_pool_if_necessary" {
            let (token0, token1, fee, sqrt_price_x96): (Key, Key, u32, U256) =
                helpers::decode_4(data);
            runtime::call_versioned_contract::<Key>(
                router.into_hash().unwrap().into(),
                None,
                "create_and_initialize_pool_if_necessary",
                runtime_args! {
                    "token0" => token0,
                    "token1" => token1,
                    "fee" => fee,
                    "sqrt_price_x96" => sqrt_price_x96,
                },
            );
        } else if entry_point == "increase_liquidity" {
            let (
                token0,
                token1,
                is_cspr,
                token_id,
                amount0_desired,
                amount1_desired,
                amount0_min,
                amount1_min,
                deadline,
            ): (Key, Key, bool, U256, U256, U256, U256, U256, u64) = helpers::decode_9(data);
            erc20_helpers::approve(token0, router, amount0_desired);
            erc20_helpers::approve(token1, router, amount1_desired);
            if is_cspr {
                try_wrap_cspr();
            }
            runtime::call_versioned_contract::<IncreaseLiquidityResult>(
                router.into_hash().unwrap().into(),
                None,
                "increase_liquidity",
                runtime_args! {
                    "data" => Bytes::from(IncreaseLiquidityParams { token_id, amount0_desired, amount1_desired, amount0_min, amount1_min, deadline, token0, token1 }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "mint" {
            let (
                token0,
                token1,
                is_cspr,
                fee,
                tick_lower,
                tick_upper,
                amount0_desired,
                amount1_desired,
                amount0_min,
                amount1_min,
                recipient,
                deadline,
            ): (
                Key,
                Key,
                bool,
                u32,
                i32,
                i32,
                U256,
                U256,
                U256,
                U256,
                Key,
                u64,
            ) = helpers::decode_12(data);
            erc20_helpers::approve(token0, router, amount0_desired);
            erc20_helpers::approve(token1, router, amount1_desired);
            if is_cspr {
                try_wrap_cspr();
            }
            runtime::call_versioned_contract::<MintResult>(
                router.into_hash().unwrap().into(),
                None,
                "mint",
                runtime_args! {
                    "data" => Bytes::from(MintParams { token0, token1, fee, tick_lower, tick_upper, amount0_desired, amount1_desired, amount0_min, amount1_min, recipient, deadline }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "decrease_liquidity" {
            let (token_id, liquidity, amount0_min, amount1_min, deadline): (
                U256,
                U128,
                U256,
                U256,
                u64,
            ) = helpers::decode_5(data);
            runtime::call_versioned_contract::<DecreaseLiquidityResult>(
                router.into_hash().unwrap().into(),
                None,
                "decrease_liquidity",
                runtime_args! {
                    "data" => Bytes::from(DecreaseLiquidityParams { token_id, liquidity, amount0_min, amount1_min, deadline }.to_bytes().unwrap()),
                },
            );
        } else if entry_point == "collect" {
            let (is_cspr, token_id, recipient, amount0_max, amount1_max): (
                bool,
                U256,
                Key,
                U128,
                U128,
            ) = helpers::decode_5(data);
            let collect_result = runtime::call_versioned_contract::<CollectResult>(
                router.into_hash().unwrap().into(),
                None,
                "collect",
                runtime_args! {
                    "data" => Bytes::from(CollectParams {token_id, recipient, amount0_max, amount1_max }.to_bytes().unwrap()),
                },
            );

            if is_cspr {
                let wcspr = get_wcspr();
                if collect_result.token0 == wcspr {
                    try_unwrap_cspr(wcspr, u256_to_u512(collect_result.amount0));
                } else {
                    try_unwrap_cspr(wcspr, u256_to_u512(collect_result.amount1));
                }
            }
        } else if entry_point == "burn" {
            let token_id: U256 = helpers::decode_1(data);
            runtime::call_versioned_contract::<()>(
                router.into_hash().unwrap().into(),
                None,
                "burn",
                runtime_args! {
                    "token_id" => token_id,
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
