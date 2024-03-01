use crate::error::{require, Error};
use casper_contract::contract_api::{runtime, storage};
use contract_utilities::helpers;

pub fn when_not_locked() {
    let locked: bool = helpers::get_key("is_locked").unwrap();
    require(!locked, Error::ContractLocked);
}

pub fn lock_contract() {
    helpers::set_key("is_locked", true);
}

pub fn unlock_contract() {
    helpers::set_key("is_locked", false);
}

pub fn init() {
    runtime::put_key("is_locked", storage::new_uref(false).into());
}
