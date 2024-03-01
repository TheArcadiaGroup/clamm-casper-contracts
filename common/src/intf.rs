use alloc::vec::Vec;
use casper_contract::contract_api::runtime;
use casper_types::{runtime_args, Key, RuntimeArgs, U128, U256};
use types::{i256::I256, PoolKey};

pub fn initialize_pool_price(pool: Key, price: &U256) {
    runtime::call_versioned_contract::<()>(
        pool.into_hash().unwrap().into(),
        None,
        "init_pool_price",
        runtime_args! {
            "sqrt_price_x96" => price
        },
    );
}

pub fn get_pool_address(factory: Key, pool_key: &PoolKey) -> Key {
    runtime::call_versioned_contract(
        factory.into_hash().unwrap().into(),
        None,
        "get_pool_address",
        runtime_args! {
            "token0" => pool_key.token0,
            "token1" => pool_key.token1,
            "fee" => pool_key.fee,
        },
    )
}

pub fn get_pool_key(token0: Key, token1: Key, fee: u32) -> PoolKey {
    let (token0, token1) = if token0.into_hash().unwrap() < token1.into_hash().unwrap() {
        (token0, token1)
    } else {
        (token1, token0)
    };

    PoolKey {
        token0,
        token1,
        fee,
    }
}

pub fn create_pool(factory: Key, token0: Key, token1: Key, fee: u32) -> Key {
    runtime::call_versioned_contract::<()>(
        factory.into_hash().unwrap().into(),
        None,
        "create_pool",
        runtime_args! {
            "token0" => token0,
            "token1" => token1,
            "fee" => fee,
        },
    );
    get_pool_address(factory, &get_pool_key(token0, token1, fee))
}

pub fn swap(
    pool: Key,
    recipient: Key,
    zero_for_one: bool,
    amount_specified: U256,
    is_amount_specified_positive: bool,
    sqrt_price_limit_x96: U256,
    data: Vec<u8>,
) -> (I256, I256) {
    runtime::call_versioned_contract::<(I256, I256)>(
        pool.into_hash().unwrap().into(),
        None,
        "swap",
        runtime_args! {
            "recipient" => recipient,
            "zero_for_one" => zero_for_one,
            "amount_specified" => amount_specified,
            "is_amount_specified_positive" => is_amount_specified_positive,
            "sqrt_price_limit_x96" => sqrt_price_limit_x96,
            "data" => data,
        },
    )
}

pub fn flash(pool: Key, recipient: Key, amount0: U256, amount1: U256, data: Vec<u8>) {
    runtime::call_versioned_contract::<()>(
        pool.into_hash().unwrap().into(),
        None,
        "flash",
        runtime_args! {
            "recipient" => recipient,
            "amount0" => amount0,
            "amount1" => amount1,
            "data" => data,
        },
    );
}

pub fn mint(
    pool: Key,
    recipient: Key,
    tick_lower: i32,
    tick_upper: i32,
    amount: U128,
    data: Vec<u8>,
) {
    runtime::call_versioned_contract::<(U256, U256)>(
        pool.into_hash().unwrap().into(),
        None,
        "mint",
        runtime_args! {
            "recipient" => recipient,
            "tick_lower" => tick_lower,
            "tick_upper" => tick_upper,
            "amount" => amount,
            "data" => data,
        },
    );
}
