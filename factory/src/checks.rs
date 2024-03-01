use casper_contract::contract_api::runtime;
use casper_types::{runtime_args, Key, RuntimeArgs};
use common::error::require;
use contract_utilities::helpers;
use math::tickmath::{MAX_TICK, MIN_TICK};

use crate::store::{read_factory, read_slot0_unlocked};

pub fn only_factory_owner() {
    let caller = helpers::get_immediate_caller_key();
    let factory = read_factory();
    let factory_owner: Key = runtime::call_versioned_contract(
        factory.into_hash().unwrap().into(),
        None,
        "owner",
        runtime_args! {},
    );
    require(
        caller == factory_owner,
        common::error::Error::InvalidFactoryOwner,
    );
}

pub fn check_ticks(tick_lower: i32, tick_upper: i32) {
    require(tick_lower < tick_upper, common::error::Error::ErrTLU);
    require(tick_lower >= MIN_TICK, common::error::Error::ErrTLM);
    require(tick_upper <= MAX_TICK, common::error::Error::ErrTUM);
}

pub fn check_pool() {
    require(
        runtime::has_key("is_pool"),
        common::error::Error::ErrNotPool,
    );
}

pub fn check_slot0_unlocked() {
    require(read_slot0_unlocked(), common::error::Error::ErrLOK);
}
