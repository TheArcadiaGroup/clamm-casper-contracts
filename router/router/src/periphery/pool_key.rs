use casper_types::Key;
use types::PoolKey;

use super::store::read_factory;

pub fn get_pool_address(pool_key: &PoolKey) -> Key {
    common::intf::get_pool_address(read_factory(), pool_key)
}

pub fn get_pool_key(token0: Key, token1: Key, fee: u32) -> PoolKey {
    common::intf::get_pool_key(token0, token1, fee)
}
