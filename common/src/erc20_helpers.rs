use casper_contract::contract_api::runtime;
use casper_types::{runtime_args, Key, RuntimeArgs, U256};
pub fn get_total_supply(contract: Key) -> U256 {
    let total_supply: U256 = runtime::call_versioned_contract(
        contract.into_hash().unwrap().into(),
        None,
        "total_supply",
        runtime_args! {},
    );
    total_supply
}

pub fn get_balance(token: Key, user: Key) -> U256 {
    let b: U256 = runtime::call_versioned_contract(
        token.into_hash().unwrap().into(),
        None,
        "balance_of",
        runtime_args! {
            "address" => user
        },
    );
    b
}

pub fn get_allowance(token: Key, owner: Key, spender: Key) -> U256 {
    runtime::call_versioned_contract(
        token.into_hash().unwrap().into(),
        None,
        "allowance",
        runtime_args! {
            "owner" => owner,
            "spender" => spender
        },
    )
}

pub fn get_decimals(token: Key) -> u8 {
    let d: u8 = runtime::call_versioned_contract(
        token.into_hash().unwrap().into(),
        None,
        "decimals",
        runtime_args! {},
    );
    d
}

pub fn transfer(token: Key, recipient: Key, amount: U256) {
    let _: () = runtime::call_versioned_contract(
        token.into_hash().unwrap().into(),
        None,
        "transfer",
        runtime_args! {
            "recipient" => recipient,
            "amount" => amount
        },
    );
}

pub fn transfer_from(token: Key, from: Key, recipient: Key, amount: U256) {
    let _: () = runtime::call_versioned_contract(
        token.into_hash().unwrap().into(),
        None,
        "transfer_from",
        runtime_args! {
            "owner" => from,
            "recipient" => recipient,
            "amount" => amount
        },
    );
}

pub fn approve(token: Key, spender: Key, amount: U256) {
    let _: () = runtime::call_versioned_contract(
        token.into_hash().unwrap().into(),
        None,
        "approve",
        runtime_args! {
            "spender" => spender,
            "amount" => amount
        },
    );
}
