use alloc::vec::Vec;
use casper_contract::contract_api::runtime::call_versioned_contract;
use casper_types::{runtime_args, RuntimeArgs, U256};
use contract_utilities::helpers::get_immediate_caller_key;
use types::i256::I256;

pub fn call_flash_callback(fee0: U256, fee1: U256, data: Vec<u8>) {
    let caller = get_immediate_caller_key();
    call_versioned_contract::<()>(
        caller.into_hash().unwrap().into(),
        None,
        "flash_callback",
        runtime_args! {
            "fee0" => fee0,
            "fee1" => fee1,
            "data" => data,
        },
    );
}

pub fn call_mint_callback(amount0_owed: U256, amount1_owed: U256, data: Vec<u8>) {
    let caller = get_immediate_caller_key();
    call_versioned_contract::<()>(
        caller.into_hash().unwrap().into(),
        None,
        "mint_callback",
        runtime_args! {
            "amount0_owed" => amount0_owed,
            "amount1_owed" => amount1_owed,
            "data" => data,
        },
    );
}

pub fn call_swap_callback(amount0_delta: I256, amount1_delta: I256, data: Vec<u8>) {
    let caller = get_immediate_caller_key();
    call_versioned_contract::<()>(
        caller.into_hash().unwrap().into(),
        None,
        "swap_callback",
        runtime_args! {
            "amount0_delta" => amount0_delta,
            "amount1_delta" => amount1_delta,
            "data" => data,
        },
    );
}
