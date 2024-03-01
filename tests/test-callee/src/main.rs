#![no_main]
#![no_std]
#![feature(type_ascription)]
extern crate alloc;
mod entry_points;

pub mod logics;
pub mod store;

use alloc::{format, string::String};
use casper_contract::contract_api::runtime;
use casper_types::{contracts::NamedKeys, runtime_args, Key, RuntimeArgs};
use common::error::Error;
use common::upgrade;
use contract_utilities::helpers;

#[no_mangle]
pub extern "C" fn init() {
    if helpers::get_key::<Key>("contract_hash").is_some() {
        runtime::revert(Error::ContractAlreadyInitialized);
    }

    let contract_hash: Key = runtime::get_named_arg("contract_hash");
    let contract_package_hash: Key = runtime::get_named_arg("contract_package_hash");
    helpers::set_key("contract_hash", contract_hash);
    helpers::set_key("contract_package_hash", contract_package_hash);
    logics::initialize();
}

#[no_mangle]
fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    if !runtime::has_key(&format!("{}_package_hash", contract_name)) {
        let (contract_hash, contract_package_hash) =
            upgrade::install_contract(contract_name, entry_points::default(), NamedKeys::new());

        runtime::call_contract::<()>(
            contract_hash,
            "init",
            runtime_args! {
                "contract_hash" => Key::from(contract_hash),
                "contract_package_hash" => Key::from(contract_package_hash)
            },
        );
    } else {
        upgrade::upgrade_contract(contract_name, entry_points::default(), NamedKeys::new());
    }
}
