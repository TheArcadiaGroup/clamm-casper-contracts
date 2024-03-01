#![allow(unused_parens)]
#![allow(non_snake_case)]
// #![allow(dead_code)]

extern crate alloc;
use casper_event_standard::Event;
use casper_types::U128;
use casper_types::{Key, U256};
use contract_utilities::helpers::current_block_timestamp;
#[derive(Event, Debug, PartialEq, Eq)]
pub struct IncreaseLiquidity {
    pub token_id: U256,
    pub liquidity: U128,
    pub amount0: U256,
    pub amount1: U256,
    pub timestamp: u64,
}

impl IncreaseLiquidity {
    pub fn new(token_id: U256, liquidity: U128, amount0: U256, amount1: U256) -> Self {
        Self {
            token_id,
            liquidity,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct DecreaseLiquidity {
    pub token_id: U256,
    pub liquidity: U128,
    pub amount0: U256,
    pub amount1: U256,
    pub timestamp: u64,
}

impl DecreaseLiquidity {
    pub fn new(token_id: U256, liquidity: U128, amount0: U256, amount1: U256) -> Self {
        Self {
            token_id,
            liquidity,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Collect {
    pub token_id: U256,
    pub recipient: Key,
    pub amount0: U256,
    pub amount1: U256,
    pub timestamp: u64,
}

impl Collect {
    pub fn new(token_id: U256, recipient: Key, amount0: U256, amount1: U256) -> Self {
        Self {
            token_id,
            recipient,
            amount0,
            amount1,
            timestamp: current_block_timestamp(),
        }
    }
}
