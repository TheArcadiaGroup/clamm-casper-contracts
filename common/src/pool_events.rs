#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use casper_event_standard::Event;
use casper_types::U128;
use casper_types::{Key, U256};
use contract_utilities::helpers;
use contract_utilities::helpers::current_block_timestamp;
use types::i256::I256;
#[derive(Event, Debug, PartialEq, Eq)]
pub struct Initialize {
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub timestamp: u64,
}
impl Initialize {
    pub fn new(sqrt_price_x96: U256, tick: i32) -> Self {
        Self {
            sqrt_price_x96,
            tick,
            timestamp: helpers::current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Mint {
    sender: Key,
    owner: Key,
    tick_lower: i32,
    tick_upper: i32,
    amount: U128,
    amount0: U256,
    amount1: U256,
    timestamp: u64,
}

impl Mint {
    pub fn new(
        sender: Key,
        owner: Key,
        tick_lower: i32,
        tick_upper: i32,
        amount: U128,
        amount0: U256,
        amount1: U256,
    ) -> Self {
        Self {
            sender,
            owner,
            tick_lower,
            tick_upper,
            amount,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Collect {
    pub owner: Key,
    pub recipient: Key,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub amount0: U256,
    pub amount1: U256,
    pub timestamp: u64,
}

impl Collect {
    pub fn new(
        owner: Key,
        recipient: Key,
        tick_lower: i32,
        tick_upper: i32,
        amount0: U256,
        amount1: U256,
    ) -> Self {
        Self {
            owner,
            recipient,
            tick_lower,
            tick_upper,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Burn {
    owner: Key,
    tick_lower: i32,
    tick_upper: i32,
    amount: U128,
    amount0: U256,
    amount1: U256,
    timestamp: u64,
}

impl Burn {
    pub fn new(
        owner: Key,
        tick_lower: i32,
        tick_upper: i32,
        amount: U128,
        amount0: U256,
        amount1: U256,
    ) -> Self {
        Self {
            owner,
            tick_lower,
            tick_upper,
            amount,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq)]
pub struct Swap {
    sender: Key,
    recipient: Key,
    amount0: String,
    amount1: String,
    sqrt_price_x96: U256,
    liquidity: U128,
    tick: i32,
    timestamp: u64,
}

impl Swap {
    pub fn new(
        sender: Key,
        recipient: Key,
        amount0: I256,
        amount1: I256,
        sqrt_price_x96: U256,
        liquidity: U128,
        tick: i32,
    ) -> Self {
        Self {
            sender,
            recipient,
            amount0: amount0.0.to_string(),
            amount1: amount1.0.to_string(),
            sqrt_price_x96,
            liquidity,
            tick,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Flash {
    sender: Key,
    recipient: Key,
    amount0: U256,
    amount1: U256,
    paid0: U256,
    paid1: U256,
    timestamp: u64,
}

impl Flash {
    pub fn new(
        sender: Key,
        recipient: Key,
        amount0: U256,
        amount1: U256,
        paid0: U256,
        paid1: U256,
    ) -> Self {
        Self {
            sender,
            recipient,
            amount0,
            amount1,
            paid0,
            paid1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct IncreaseObservationCardinalityNext {
    pub observation_cardinality_next_old: u32,
    pub observation_cardinality_next_new: u32,
    timestamp: u64,
}

impl IncreaseObservationCardinalityNext {
    pub fn new(
        observation_cardinality_next_old: u32,
        observation_cardinality_next_new: u32,
    ) -> Self {
        Self {
            observation_cardinality_next_old,
            observation_cardinality_next_new,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct SetFeeProtocol {
    pub fee_protocol0_old: u8,
    pub fee_protocol1_old: u8,
    pub fee_protocol0_new: u8,
    pub fee_protocol1_new: u8,
    timestamp: u64,
}

impl SetFeeProtocol {
    pub fn new(
        fee_protocol0_old: u8,
        fee_protocol1_old: u8,
        fee_protocol0_new: u8,
        fee_protocol1_new: u8,
    ) -> Self {
        Self {
            fee_protocol0_old,
            fee_protocol1_old,
            fee_protocol0_new,
            fee_protocol1_new,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct CollectProtocol {
    pub sender: Key,
    pub recipient: Key,
    pub amount0: U128,
    pub amount1: U128,
    pub timestamp: u64,
}

impl CollectProtocol {
    pub fn new(sender: Key, recipient: Key, amount0: U128, amount1: U128) -> Self {
        Self {
            sender,
            recipient,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct SnapshotCumulativesInside {
    pub tick_cumulative_inside: i64,
    pub seconds_per_liquidity_insideX128: U256,
    pub seconds_inside: u32,
}

impl SnapshotCumulativesInside {
    pub fn new(
        tick_cumulative_inside: i64,
        seconds_per_liquidity_insideX128: U256,
        seconds_inside: u32,
    ) -> Self {
        Self {
            tick_cumulative_inside,
            seconds_per_liquidity_insideX128,
            seconds_inside,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct PoolCreated {
    token0: Key,
    token1: Key,
    fee: u32,
    tick_spacing: i32,
    pool: Key,
    timestamp: u64,
}

impl PoolCreated {
    pub fn new(token0: Key, token1: Key, fee: u32, tick_spacing: i32, pool: Key) -> Self {
        Self {
            token0,
            token1,
            fee,
            tick_spacing,
            pool,
            timestamp: current_block_timestamp(),
        }
    }
}
