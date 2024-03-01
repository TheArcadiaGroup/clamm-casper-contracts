use crate::error::Error;
use alloc::{string::String, vec, vec::*};
use casper_contract::contract_api::runtime;
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{
    CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, Parameter, RuntimeArgs,
};
use contract_utilities::helpers::{self, current_block_timestamp};
pub const IS_TESTING: &str = "is_testing";
pub const FAKE_TIMESTAMP: &str = "fake_timestamp";
pub const ARG_FAKE_TIMESTAMP: &str = "fake_timestamp";
pub const ROLL_TIMESTAMP: &str = "roll_timestamp";

#[no_mangle]
pub extern "C" fn roll_timestamp() {
    // only_owner();
    let roll_timestamp: u64 = runtime::get_named_arg(ROLL_TIMESTAMP);
    helpers::set_key(FAKE_TIMESTAMP, current_block_timestamp() + roll_timestamp);
}

#[no_mangle]
pub extern "C" fn warp_fake_timestamp() {
    // only_owner();
    let warp_timestamp: u64 = runtime::get_named_arg("warp_timestamp");
    helpers::set_key(FAKE_TIMESTAMP, warp_timestamp);
}

#[no_mangle]
pub extern "C" fn get_timestamp() {
    runtime::ret(CLValue::from_t(current_block_timestamp()).unwrap_or_revert())
}

pub fn get_testing_mode() -> bool {
    helpers::get_optional_named_arg_with_user_errors("is_testing", Error::InvalidTestingMode.into())
        .unwrap_or_default()
}

pub fn with_testing_mod(args: &mut RuntimeArgs) -> RuntimeArgs {
    args.insert("is_testing", get_testing_mode()).unwrap();
    args.clone()
}

pub fn init() {
    if get_testing_mode() {
        helpers::set_key(FAKE_TIMESTAMP, 0u64);
    }
}

pub fn entry_points() -> Vec<EntryPoint> {
    if get_testing_mode() {
        vec![
            EntryPoint::new(
                String::from("roll_timestamp"),
                vec![Parameter::new(ROLL_TIMESTAMP, CLType::U64)],
                CLType::Unit,
                EntryPointAccess::Public,
                EntryPointType::Contract,
            ),
            EntryPoint::new(
                String::from("warp_fake_timestamp"),
                vec![Parameter::new("warp_timestamp", CLType::U64)],
                CLType::U64,
                EntryPointAccess::Public,
                EntryPointType::Contract,
            ),
            EntryPoint::new(
                String::from("get_timestamp"),
                vec![],
                CLType::U64,
                EntryPointAccess::Public,
                EntryPointType::Contract,
            ),
        ]
    } else {
        vec![]
    }
}
