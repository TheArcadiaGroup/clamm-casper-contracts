use core::convert::TryInto;

use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    runtime_args, EntryPoint, EntryPoints, HashAddr, Key, RuntimeArgs, URef, U256, U512,
};
use contract_utilities::helpers::encode_3;

pub fn compute_pool_key(token0: Key, token1: Key, fee: u32) -> HashAddr {
    let b = encode_3(&token0, &token1, &fee);
    runtime::blake2b(b)
}

pub fn position_key(owner: Key, tick_lower: i32, tick_upper: i32) -> String {
    let encoded = encode_3(&owner, &tick_lower, &tick_upper);
    let hash = runtime::blake2b(encoded);
    hex::encode(hash)
}

pub fn set_size_32(primitive: &[u8]) -> [u8; 32] {
    primitive.try_into().expect("slice with incorrect length")
}

pub fn set_size_4(primitive: &[u8]) -> [u8; 4] {
    primitive.try_into().expect("slice with incorrect length")
}

/// Converts an `&[u8]` to a `[u8; 64]`.
pub fn set_size_64(primitive: &[u8]) -> [u8; 64] {
    primitive.try_into().expect("slice with incorrect length")
}

pub fn is_token_sorted(token0: Key, token1: Key) -> bool {
    token0.into_hash().unwrap() < token1.into_hash().unwrap()
}

pub fn sort_tokens(token0: Key, token1: Key) -> (Key, Key) {
    if token0.into_hash().unwrap() < token1.into_hash().unwrap() {
        (token0, token1)
    } else {
        (token1, token0)
    }
}

pub fn add_entry_points(entry_points: &mut EntryPoints, list: &Vec<EntryPoint>) {
    for e in list {
        entry_points.add_entry_point(e.clone());
    }
}

pub fn wrap_cspr(wcspr: Key, purse: URef, amount: U512) {
    let _: () = runtime::call_versioned_contract(
        wcspr.into_hash().unwrap().into(),
        None,
        "deposit",
        runtime_args! {
            "amount" => amount,
            "purse" => purse
        },
    );
}

pub fn unwrap_wcspr(wcspr: Key, to: Key, amount: U512) {
    let purse = system::create_purse();
    let _: () = runtime::call_versioned_contract(
        wcspr.into_hash().unwrap().into(),
        None,
        "withdraw",
        runtime_args! {
            "amount" => amount,
            "purse" => purse
        },
    );
    system::transfer_from_purse_to_account(purse, to.into_account().unwrap(), amount, None)
        .unwrap_or_revert();
}

pub fn unwrap_wcspr_to_purse(wcspr: Key, purse: URef, amount: U512) {
    let _: () = runtime::call_versioned_contract(
        wcspr.into_hash().unwrap().into(),
        None,
        "withdraw",
        runtime_args! {
            "amount" => amount,
            "purse" => purse
        },
    );
}

pub fn u256_to_u512(nb: U256) -> U512 {
    let mut b = [0u8; 32];
    nb.to_big_endian(&mut b);
    U512::from_big_endian(&b)
}

pub fn u512_to_u256(nb: U512) -> U256 {
    let mut b = [0u8; 64];
    nb.to_big_endian(&mut b);
    U256::from_big_endian(&b[32..64])
}

pub fn new_purse(amount: U512) -> URef {
    let main_purse: URef = account::get_main_purse();
    let secondary_purse: URef = system::create_purse();
    system::transfer_from_purse_to_purse(main_purse, secondary_purse, amount, None)
        .unwrap_or_revert();
    secondary_purse
}
