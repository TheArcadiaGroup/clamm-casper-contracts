use crate::error::{require, Error};
use alloc::{string::String, vec, vec::*};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::Key;
use casper_types::{CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, Parameter};
use contract_utilities::helpers::{self};

pub fn only_owner() {
    require(
        owner_internal() == helpers::get_immediate_caller_key(),
        Error::OnlyOwnerCanRevoke,
    );
}

#[no_mangle]
pub extern "C" fn owner() {
    runtime::ret(CLValue::from_t(owner_internal()).unwrap_or_revert());
}

pub fn owner_internal() -> Key {
    let owner_key: Key = helpers::get_key("contract_owner").unwrap();
    owner_key
}

#[no_mangle]
pub extern "C" fn transfer_owner() {
    only_owner();
    let contract_owner: Key = runtime::get_named_arg("contract_owner");
    helpers::set_key("contract_owner", contract_owner);
}

pub fn init(contract_owner: Key) {
    runtime::put_key("contract_owner", storage::new_uref(contract_owner).into());
}

pub fn entry_points() -> Vec<EntryPoint> {
    vec![
        EntryPoint::new(
            String::from("transfer_owner"),
            vec![Parameter::new("contract_owner", CLType::Key)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ),
        EntryPoint::new(
            String::from("owner"),
            vec![],
            CLType::Key,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ),
    ]
}
