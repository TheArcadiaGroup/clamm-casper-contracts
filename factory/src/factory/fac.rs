use alloc::{
    string::{String, ToString},
    vec,
};
use casper_contract::contract_api::runtime::call_versioned_contract;
use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, HashAddr, Key,
    Parameter, RuntimeArgs,
};
use common::owner::only_owner;
use common::{
    error::{require, Error},
    utils,
};
use common::{get_set_dict, pool_events};
use contract_utilities::helpers::{self, get_named_args_3};
use contract_utilities::helpers::{get_named_args_2, get_self_key, null_key};

use crate::entry_points;

get_set_dict!(
    "fee_amount_tick_spacing",
    "fee",
    u32,
    i32,
    i32::default(),
    save_fee_amount_tick_spacing,
    read_fee_amount_tick_spacing,
    get_fee_amount_tick_spacing,
    get_fee_amount_tick_spacing_ep,
    "get_fee_amount_tick_spacing"
);

get_set_dict!(
    "pool_map",
    "pool_key",
    HashAddr,
    HashAddr,
    HashAddr::default(),
    save_pool_map,
    read_pool_map,
    get_pool_map,
    get_pool_map_ep,
    "get_pool_map"
);

pub fn compute_pool_key(token0: Key, token1: Key, fee: u32) -> HashAddr {
    utils::compute_pool_key(token0, token1, fee)
}

pub fn initialize() {
    storage::new_dictionary("fee_amount_tick_spacing")
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary("pool_map").unwrap_or_revert_with(Error::FailedToCreateDictionary);

    save_fee_amount_tick_spacing(&500, &10);
    save_fee_amount_tick_spacing(&3000, &60);
    save_fee_amount_tick_spacing(&10000, &200);
}

#[no_mangle]
pub extern "C" fn get_pool_address() {
    let (token0, token1, fee): (Key, Key, u32) = get_named_args_3(vec![
        "token0".to_string(),
        "token1".to_string(),
        "fee".to_string(),
    ]);
    let pool_key = compute_pool_key(token0, token1, fee);
    let pool = read_pool_map(&pool_key);
    let pool = Key::Hash(pool);
    runtime::ret(CLValue::from_t(pool).unwrap())
}

#[no_mangle]
pub extern "C" fn create_pool() {
    let (token0, token1, fee): (Key, Key, u32) = get_named_args_3(vec![
        "token0".to_string(),
        "token1".to_string(),
        "fee".to_string(),
    ]);
    require(token0 != token1, Error::ErrSameToken);
    let (token0, token1) = if token0.into_hash().unwrap() < token1.into_hash().unwrap() {
        (token0, token1)
    } else {
        (token1, token0)
    };

    require(token0 != null_key(), Error::ErrTokenNull);
    let tick_spacing = read_fee_amount_tick_spacing(&fee);
    require(tick_spacing != 0, Error::ErrTickSpacingNull);
    let pool_key = compute_pool_key(token0, token1, fee);
    let pool = read_pool_map(&pool_key);
    let pool = Key::Hash(pool);
    require(pool == Key::Hash(HashAddr::default()), Error::ErrPoolExist);
    // deploy new pool
    let (package_hash, _) = storage::create_contract_package_at_hash();
    let (contract_hash, _) =
        storage::add_contract_version(package_hash, entry_points::default(false), NamedKeys::new());
    // console::log(&runtime::has_key(FAKE_TIMESTAMP).to_string());
    call_versioned_contract::<()>(
        package_hash,
        None,
        "init_pool",
        runtime_args! {
            "contract_hash" => Key::from(contract_hash),
            "contract_package_hash" => Key::from(package_hash),
            "factory" => get_self_key(),
            "token0" => token0,
            "token1" => token1,
            "fee" => fee,
            "tick_spacing" => tick_spacing
        },
    );
    save_pool_map(
        &compute_pool_key(token0, token1, fee),
        &package_hash.value(),
    );
    casper_event_standard::emit(pool_events::PoolCreated::new(
        token0,
        token1,
        fee,
        tick_spacing,
        Key::from(package_hash),
    ));
}

#[no_mangle]
pub extern "C" fn enable_fee_amount() {
    let (fee, tick_spacing): (u32, i32) =
        get_named_args_2(vec!["fee".to_string(), "tick_spacing".to_string()]);
    only_owner();
    require(fee < 1000000, Error::ErrFactoryFee);
    require(
        tick_spacing > 0 && tick_spacing < 16384,
        Error::ErrInvalidTickSpacing,
    );
    require(read_fee_amount_tick_spacing(&fee) == 0, Error::ErrFeeExist);
    save_fee_amount_tick_spacing(&fee, &tick_spacing);
}
