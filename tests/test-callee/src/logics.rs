use alloc::{string::ToString, vec};
use casper_contract::contract_api::runtime::call_versioned_contract;
use casper_types::{bytesrepr::Bytes, runtime_args, Key, RuntimeArgs, U128, U256};
use common::{
    console, erc20_helpers,
    error::require,
    intf::{self, swap},
};
use contract_utilities::helpers::{
    decode_1, decode_3, encode_1, encode_3, get_immediate_caller_key, get_named_args_3,
    get_named_args_4, get_named_args_5, get_named_args_6,
};
use types::i256::I256;

use crate::store;
pub fn initialize() {
    store::initialize();
}

#[no_mangle]
pub extern "C" fn swap_exact_0_for_1() {
    console::log("swap_exact_0_for_1");
    let (pool, amount0_in, recipient, sqrt_price_limit_x96): (Key, U256, Key, U256) =
        get_named_args_4(vec![
            "pool".to_string(),
            "amount0_in".to_string(),
            "recipient".to_string(),
            "sqrt_price_limit_x96".to_string(),
        ]);
    console::log("calling swap function in testcallee");
    swap(
        pool,
        recipient,
        true,
        amount0_in,
        true,
        sqrt_price_limit_x96,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn swap_0_for_exact_1() {
    let (pool, amount1_out, recipient, sqrt_price_limit_x96): (Key, U256, Key, U256) =
        get_named_args_4(vec![
            "pool".to_string(),
            "amount1_out".to_string(),
            "recipient".to_string(),
            "sqrt_price_limit_x96".to_string(),
        ]);
    swap(
        pool,
        recipient,
        true,
        amount1_out,
        false,
        sqrt_price_limit_x96,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn swap_exact_1_for_0() {
    let (pool, amount1_in, recipient, sqrt_price_limit_x96): (Key, U256, Key, U256) =
        get_named_args_4(vec![
            "pool".to_string(),
            "amount1_in".to_string(),
            "recipient".to_string(),
            "sqrt_price_limit_x96".to_string(),
        ]);
    swap(
        pool,
        recipient,
        false,
        amount1_in,
        true,
        sqrt_price_limit_x96,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn swap_1_for_exact_0() {
    let (pool, amount0_out, recipient, sqrt_price_limit_x96): (Key, U256, Key, U256) =
        get_named_args_4(vec![
            "pool".to_string(),
            "amount0_out".to_string(),
            "recipient".to_string(),
            "sqrt_price_limit_x96".to_string(),
        ]);
    swap(
        pool,
        recipient,
        false,
        amount0_out,
        false,
        sqrt_price_limit_x96,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn swap_to_lower_sqrt_price() {
    console::log("swap_to_lower_sqrt_price");
    let (pool, recipient, sqrt_price_x96): (Key, Key, U256) = get_named_args_3(vec![
        "pool".to_string(),
        "recipient".to_string(),
        "sqrt_price_x96".to_string(),
    ]);
    swap(
        pool,
        recipient,
        true,
        U256::from_dec_str(&ethnum::I256::MAX.to_string()).unwrap(),
        true,
        sqrt_price_x96,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn swap_to_higher_sqrt_price() {
    let (pool, recipient, sqrt_price_x96): (Key, Key, U256) = get_named_args_3(vec![
        "pool".to_string(),
        "recipient".to_string(),
        "sqrt_price_x96".to_string(),
    ]);
    swap(
        pool,
        recipient,
        false,
        U256::from_dec_str(&ethnum::I256::MAX.to_string()).unwrap(),
        true,
        sqrt_price_x96,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn swap_callback() {
    let (amount0_delta, amount1_delta, data): (I256, I256, Bytes) = get_named_args_3(vec![
        "amount0_delta".to_string(),
        "amount1_delta".to_string(),
        "data".to_string(),
    ]);
    let sender: Key = decode_1(&data);
    let caller = get_immediate_caller_key();
    if amount0_delta.0 > 0 {
        let token0: Key = call_versioned_contract(
            caller.into_hash().unwrap().into(),
            None,
            "get_token0",
            runtime_args! {},
        );
        erc20_helpers::transfer_from(
            token0,
            sender,
            caller,
            U256::from_dec_str(&amount0_delta.0.to_string()).unwrap(),
        );
    } else if amount1_delta.0 > 0 {
        let token1: Key = call_versioned_contract(
            caller.into_hash().unwrap().into(),
            None,
            "get_token1",
            runtime_args! {},
        );
        erc20_helpers::transfer_from(
            token1,
            sender,
            caller,
            U256::from_dec_str(&amount1_delta.0.to_string()).unwrap(),
        );
    } else {
        require(
            amount0_delta.0 == 0 && amount1_delta.0 == 0,
            common::error::Error::ErrSwapCallee,
        );
    }
}

#[no_mangle]
pub extern "C" fn mint() {
    let (pool, recipient, tick_lower, tick_upper, amount): (Key, Key, i32, i32, U128) =
        get_named_args_5(vec![
            "pool".to_string(),
            "recipient".to_string(),
            "tick_lower".to_string(),
            "tick_upper".to_string(),
            "amount".to_string(),
        ]);
    intf::mint(
        pool,
        recipient,
        tick_lower,
        tick_upper,
        amount,
        encode_1(&get_immediate_caller_key()),
    );
}

#[no_mangle]
pub extern "C" fn mint_callback() {
    let (amount0_owed, amount1_owed, data): (U256, U256, Bytes) = get_named_args_3(vec![
        "amount0_owed".to_string(),
        "amount1_owed".to_string(),
        "data".to_string(),
    ]);
    let sender: Key = decode_1(&data);
    let caller = get_immediate_caller_key();
    if amount0_owed > U256::zero() {
        let token0: Key = call_versioned_contract(
            caller.into_hash().unwrap().into(),
            None,
            "get_token0",
            runtime_args! {},
        );
        erc20_helpers::transfer_from(token0, sender, caller, amount0_owed);
    }
    if amount1_owed > U256::zero() {
        let token1: Key = call_versioned_contract(
            caller.into_hash().unwrap().into(),
            None,
            "get_token1",
            runtime_args! {},
        );
        erc20_helpers::transfer_from(token1, sender, caller, amount1_owed);
    }
}

#[no_mangle]
pub extern "C" fn flash() {
    let (pool, recipient, amount0, amount1, pay0, pay1): (Key, Key, U256, U256, U256, U256) =
        get_named_args_6(vec![
            "pool".to_string(),
            "recipient".to_string(),
            "amount0".to_string(),
            "amount1".to_string(),
            "pay0".to_string(),
            "pay1".to_string(),
        ]);

    intf::flash(
        pool,
        recipient,
        amount0,
        amount1,
        encode_3(&get_immediate_caller_key(), &pay0, &pay1),
    );
    console::log("done flash");
}

#[no_mangle]
pub extern "C" fn flash_callback() {
    let (_fee0, _fee1, data): (U256, U256, Bytes) = get_named_args_3(vec![
        "fee0".to_string(),
        "fee1".to_string(),
        "data".to_string(),
    ]);
    let (sender, pay0, pay1): (Key, U256, U256) = decode_3(&data);
    let caller = get_immediate_caller_key();
    if pay0 > U256::zero() {
        let token0: Key = call_versioned_contract(
            caller.into_hash().unwrap().into(),
            None,
            "get_token0",
            runtime_args! {},
        );
        erc20_helpers::transfer_from(token0, sender, caller, pay0);
    }
    if pay1 > U256::zero() {
        let token1: Key = call_versioned_contract(
            caller.into_hash().unwrap().into(),
            None,
            "get_token1",
            runtime_args! {},
        );
        erc20_helpers::transfer_from(token1, sender, caller, pay1);
    }
}
