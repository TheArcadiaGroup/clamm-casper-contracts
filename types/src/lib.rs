#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
pub mod i128;
pub mod i256;
use crate::i128::I128;
use crate::i256::I256;
use alloc::vec::Vec;
use casper_types::{bytesrepr::Bytes, Key, U128, U256};
use casper_types_derive::{CLTyped, FromBytes, ToBytes};
use contract_utilities::helpers::null_key;
#[derive(Clone, CLTyped, ToBytes, FromBytes, Default, PartialEq, Debug)]
pub struct Observation {
    pub block_timestamp: u64,
    pub tick_cumulative: i64,
    pub seconds_per_liquidity_cumulative_x128: U256,
    pub initialized: bool,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes, Default)]
pub struct PositionInfo {
    pub liquidity: U128,
    pub fee_growth_inside0_last_x128: U256,
    pub fee_growth_inside1_last_x128: U256,
    pub tokens_owed0: U128,
    pub tokens_owed1: U128,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes, Default)]
pub struct TickInfo {
    pub liquidity_gross: U128,
    pub liquidity_net: I128,
    pub fee_growth_outside0_x128: U256,
    pub fee_growth_outside1_x128: U256,
    pub tick_cumulative_outside: i64,
    pub seconds_per_liquidity_outside_x128: U256,
    pub seconds_outside: u32,
    pub initialized: bool,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes, Default)]
pub struct Slot0 {
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub observation_index: u16,
    pub observation_cardinality: u16,
    pub observation_cardinality_next: u16,
    pub fee_protocol: u8,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes, Default)]
pub struct ProtocolFees {
    pub token0: U128,
    pub token1: U128,
}

#[derive(Clone)]
pub struct ModifyPositionParams {
    pub owner: Key,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity_delta: i128,
}

#[derive(Clone)]
pub struct SwapCache {
    pub fee_protocol: u8,
    pub liquidity_start: u128,
    pub block_time: u32,
    pub tick_cumulative: i64,
    pub seconds_per_liquidity_cumulative_x128: U256,
    pub computed_latest_observation: bool,
}

#[derive(Clone)]
pub struct SwapState {
    pub amount_specified_remaining: I256,
    pub amount_calculated: I256,
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub fee_growth_global_x128: U256,
    pub protocol_fee: u128,
    pub liquidity: u128,
}

#[derive(Clone, Default)]
pub struct StepComputations {
    pub sqrt_price_start_x96: U256,
    pub tick_next: i32,
    pub initialized: bool,
    pub sqrt_price_next_x96: U256,
    pub amount_in: U256,
    pub amount_out: U256,
    pub fee_amount: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct Position {
    // the ID of the pool with which this token is connected
    pub pool_id: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: U128,
    pub fee_growth_inside0_last_x128: U256,
    pub fee_growth_inside1_last_x128: U256,
    pub tokens_owed0: U128,
    pub tokens_owed1: U128,
    pub fee: u32,
    pub token0: Key,
    pub token1: Key,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            pool_id: Default::default(),
            tick_lower: Default::default(),
            tick_upper: Default::default(),
            liquidity: Default::default(),
            fee_growth_inside0_last_x128: Default::default(),
            fee_growth_inside1_last_x128: Default::default(),
            tokens_owed0: Default::default(),
            tokens_owed1: Default::default(),
            fee: Default::default(),
            token0: null_key(),
            token1: null_key(),
        }
    }
}

/// @notice The identifying key of the pool
#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct PoolKey {
    // the ID of the pool with which this token is connected
    pub token0: Key,
    pub token1: Key,
    pub fee: u32,
}

impl Default for PoolKey {
    fn default() -> Self {
        Self {
            token0: null_key(),
            token1: null_key(),
            fee: 0,
        }
    }
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct MintParams {
    // the ID of the pool with which this token is connected
    pub token0: Key,
    pub token1: Key,
    pub fee: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub amount0_desired: U256,
    pub amount1_desired: U256,
    pub amount0_min: U256,
    pub amount1_min: U256,
    pub recipient: Key,
    pub deadline: u64,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct MintResult {
    pub token_id: U256,
    pub liquidity: U128,
    pub amount0: U256,
    pub amount1: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct AddLiquidityParams {
    // the ID of the pool with which this token is connected
    pub token0: Key,
    pub token1: Key,
    pub fee: u32,
    pub recipient: Key,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub amount0_desired: U256,
    pub amount1_desired: U256,
    pub amount0_min: U256,
    pub amount1_min: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct MintCallbackData {
    pub pool_key: PoolKey,
    pub payer: Key,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct IncreaseLiquidityParams {
    // the ID of the pool with which this token is connected
    pub token_id: U256,
    pub amount0_desired: U256,
    pub amount1_desired: U256,
    pub amount0_min: U256,
    pub amount1_min: U256,
    pub deadline: u64,
    pub token0: Key,
    pub token1: Key,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct DecreaseLiquidityParams {
    // the ID of the pool with which this token is connected
    pub token_id: U256,
    pub liquidity: U128,
    pub amount0_min: U256,
    pub amount1_min: U256,
    pub deadline: u64,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct CollectParams {
    // the ID of the pool with which this token is connected
    pub token_id: U256,
    pub recipient: Key,
    pub amount0_max: U128,
    pub amount1_max: U128,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct SwapCallbackData {
    pub path: Bytes,
    pub payer: Key,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct ExactOutputSingleParams {
    pub token_in: Key,
    pub token_out: Key,
    pub fee: u32,
    pub recipient: Key,
    pub deadline: u64,
    pub amount_out: U256,
    pub amount_in_maximum: U256,
    pub sqrt_price_limit_x96: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct ExactOutputParams {
    pub path: Bytes,
    pub recipient: Key,
    pub deadline: u64,
    pub amount_out: U256,
    pub amount_in_maximum: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct ExactInputSingleParams {
    pub token_in: Key,
    pub token_out: Key,
    pub fee: u32,
    pub recipient: Key,
    pub deadline: u64,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub sqrt_price_limit_x96: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct ExactInputParams {
    pub path: Bytes,
    pub recipient: Key,
    pub deadline: u64,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct IncreaseLiquidityResult {
    pub liquidity: U128,
    pub amount0: U256,
    pub amount1: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct DecreaseLiquidityResult {
    pub amount0: U256,
    pub amount1: U256,
}

#[derive(Clone, CLTyped, ToBytes, FromBytes)]
pub struct CollectResult {
    pub token0: Key,
    pub token1: Key,
    pub amount0: U256,
    pub amount1: U256,
}
