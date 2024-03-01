use alloc::{string::String, vec};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, Key, Parameter, U256,
};
use common::{error::Error, get_set_dict, get_set_no_set};
use contract_utilities::helpers::{self, null_key};
use types::{PoolKey, Position};

pub const DEFAULT_AMOUNT_IN_CACHED: U256 = U256::MAX;
pub fn initialize() {
    storage::new_dictionary("pool_ids").unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary("pool_id_to_pool_key")
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary("positions").unwrap_or_revert_with(Error::FailedToCreateDictionary);
    save_next_id(1.into());
    save_next_pool_id(1);
    save_factory(null_key());
    save_amount_in_cached(DEFAULT_AMOUNT_IN_CACHED);
}

get_set_dict!(
    "pool_ids",
    "pool",
    Key,
    u64,
    u64::default(),
    save_pool_id,
    read_pool_id,
    get_pool_id,
    get_pool_id_ep,
    "get_pool_id"
);

get_set_dict!(
    "pool_id_to_pool_key",
    "pool_id",
    u64,
    PoolKey,
    PoolKey::default(),
    save_pool_key,
    read_pool_key,
    get_pool_key,
    get_pool_key_ep,
    "get_pool_key"
);

get_set_dict!(
    "positions",
    "token_id",
    U256,
    Position,
    Position::default(),
    save_position,
    read_position,
    get_position,
    get_position_ep,
    "get_position"
);

get_set_no_set!(
    next_id,
    "next_id",
    U256,
    U256::one(),
    save_next_id,
    read_next_id,
    get_next_id,
    get_next_id_ep,
    "get_next_id"
);

get_set_no_set!(
    next_pool_id,
    "next_pool_id",
    u64,
    1_u64,
    save_next_pool_id,
    read_next_pool_id,
    get_next_pool_id,
    get_next_pool_id_ep,
    "get_next_pool_id"
);

get_set_no_set!(
    factory,
    "factory",
    Key,
    helpers::null_key(),
    save_factory,
    read_factory,
    get_factory,
    get_factory_ep,
    "get_factory"
);

get_set_no_set!(
    amount_in_cached,
    "amount_in_cached",
    U256,
    U256::zero(),
    save_amount_in_cached,
    read_amount_in_cached,
    get_amount_in_cached,
    get_amount_in_cached_ep,
    "get_amount_in_cached"
);

pub fn cache_pool_key(pool: Key, pool_key: &PoolKey) -> u64 {
    let mut pool_id = read_pool_id(&pool);
    if pool_id == 0 {
        pool_id = read_next_pool_id();
        save_next_pool_id(pool_id + 1);
        save_pool_id(&pool, &pool_id);
        save_pool_key(&pool_id, pool_key);
    }
    pool_id
}
