#![no_main]
#![no_std]
#![feature(type_ascription)]

extern crate alloc;
mod entry_points;

pub mod callbacks;
pub mod checks;
pub mod events;
pub mod factory;
pub mod logics;
pub mod oracle;
pub mod position;
pub mod store;
pub mod tick;
pub mod tick_bitmap;

use alloc::{format, string::String};
use casper_contract::contract_api::runtime;
use casper_types::{contracts::NamedKeys, runtime_args, Key, RuntimeArgs, U256};
use common::{
    error::{require, Error},
    lock, owner,
    timestamp_testing::{self, with_testing_mod},
    upgrade,
};
use contract_utilities::helpers;
use events::event_schemas;
use store::read_slot0;

#[no_mangle]
pub extern "C" fn init_pool() {
    if runtime::has_key("contract_hash") {
        runtime::revert(Error::ContractAlreadyInitialized);
    }
    let caller = helpers::get_immediate_caller_key();
    let contract_hash: Key = runtime::get_named_arg("contract_hash");
    let contract_package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let factory: Key = runtime::get_named_arg("factory");
    let token0: Key = runtime::get_named_arg("token0");
    let token1: Key = runtime::get_named_arg("token1");
    let fee: u32 = runtime::get_named_arg("fee");
    let tick_spacing: i32 = runtime::get_named_arg("tick_spacing");
    helpers::set_key("contract_hash", contract_hash);
    helpers::set_key("contract_package_hash", contract_package_hash);
    owner::init(caller);
    lock::init();
    events::init_events();
    logics::initialize(factory, token0, token1, fee, tick_spacing);
}

#[no_mangle]
pub extern "C" fn init_pool_price() {
    let slot0 = read_slot0();

    require(slot0.sqrt_price_x96 == U256::zero(), Error::ErrAI);
    let sqrt_price_x96: U256 = runtime::get_named_arg("sqrt_price_x96");
    logics::initialize_pool_price(sqrt_price_x96);
}

#[no_mangle]
pub extern "C" fn init_factory() {
    if runtime::has_key("contract_hash") {
        runtime::revert(Error::ContractAlreadyInitialized);
    }

    let caller = helpers::get_immediate_caller_key();
    let contract_hash: Key = runtime::get_named_arg("contract_hash");
    let contract_package_hash: Key = runtime::get_named_arg("contract_package_hash");

    helpers::set_key("contract_hash", contract_hash);
    helpers::set_key("contract_package_hash", contract_package_hash);
    owner::init(caller);
    lock::init();
    events::init_events();
    factory::fac::initialize();
    timestamp_testing::init();
}

#[no_mangle]
fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    if !runtime::has_key(&format!("{}_package_hash", contract_name)) {
        let (contract_hash, contract_package_hash) =
            upgrade::install_contract(contract_name, entry_points::default(true), NamedKeys::new());

        runtime::call_contract::<()>(
            contract_hash,
            "init_factory",
            with_testing_mod(&mut runtime_args! {
                "contract_hash" => Key::from(contract_hash),
                "contract_package_hash" => Key::from(contract_package_hash),
            }),
        );
    } else {
        upgrade::upgrade(
            contract_name,
            entry_points::default(true),
            NamedKeys::new(),
            event_schemas(),
        );
    }
}
