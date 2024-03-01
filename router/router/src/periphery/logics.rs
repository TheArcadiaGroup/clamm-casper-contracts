use core::ops::Add;

use super::{
    checks::{check_deadline, is_authorized_for_token},
    liquidity_amounts::get_liquidity_for_amounts,
    pool_key::{get_pool_address, get_pool_key},
    store::{
        self, cache_pool_key, read_next_id, read_pool_key, read_position, save_factory,
        save_next_id, save_position,
    },
};
use crate::NFTToken;
use crate::{cep47::CEP47, periphery::store::read_factory};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use casper_contract::contract_api::runtime;
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{bytesrepr::Bytes, runtime_args, Key, RuntimeArgs, U128, U256};
use common::{
    erc20_helpers,
    error::require,
    intf::{create_pool, initialize_pool_price},
    router_events::{Collect, DecreaseLiquidity, IncreaseLiquidity},
    utils::{is_token_sorted, position_key},
};
use contract_utilities::helpers::{self, get_self_key, null_key};
use math::{fixed_point_128, fullmath, tickmath};
use types::{
    AddLiquidityParams, CollectParams, DecreaseLiquidityParams, IncreaseLiquidityParams,
    MintCallbackData, MintParams, PoolKey, Position, PositionInfo, Slot0,
};

pub fn initialize(factory: Key) {
    store::initialize();
    save_factory(factory);
}

pub fn _verify_callback(tokena: Key, tokenb: Key, fee: u32) -> Key {
    verify_callback2(&get_pool_key(tokena, tokenb, fee))
}

pub fn verify_callback2(pool_key: &PoolKey) -> Key {
    let pool = get_pool_address(pool_key);
    require(
        pool == helpers::get_immediate_caller_key(),
        common::error::Error::ErrInvalidMintCallback,
    );
    pool
}

pub fn mint_callback_internal(amount0_owed: U256, amount1_owed: U256, data: &[u8]) {
    let decoded: MintCallbackData = helpers::decode_1(data);
    verify_callback2(&decoded.pool_key);

    if amount0_owed.gt(&U256::zero()) {
        erc20_helpers::transfer_from(
            decoded.pool_key.token0,
            decoded.payer,
            helpers::get_immediate_caller_key(),
            amount0_owed,
        );
    }
    if amount1_owed.gt(&U256::zero()) {
        erc20_helpers::transfer_from(
            decoded.pool_key.token1,
            decoded.payer,
            helpers::get_immediate_caller_key(),
            amount1_owed,
        );
    }
}

pub fn mint_internal(params: &MintParams) -> (U256, u128, U256, U256) {
    check_deadline(params.deadline);

    let (liquidity, amount0, amount1, pool) = add_liquidity_internal(&AddLiquidityParams {
        token0: params.token0,
        token1: params.token1,
        fee: params.fee,
        recipient: get_self_key(),
        tick_lower: params.tick_lower,
        tick_upper: params.tick_upper,
        amount0_desired: params.amount0_desired,
        amount1_desired: params.amount1_desired,
        amount0_min: params.amount0_min,
        amount1_min: params.amount1_min,
    });

    // mint an nft
    let token_id = read_next_id();
    save_next_id(token_id.add(1));
    let token_ids = vec![token_id];
    let mut token_metas: Vec<BTreeMap<String, String>> = vec![];
    token_metas.push(BTreeMap::new());
    NFTToken::default()
        .mint(params.recipient, token_ids, token_metas)
        .unwrap_or_revert();
    let position_key = position_key(get_self_key(), params.tick_lower, params.tick_upper);
    let position_info: PositionInfo = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "get_position",
        runtime_args! {
            "position_key" => position_key
        },
    );
    let fee_growth_inside0_last_x128 = position_info.fee_growth_inside0_last_x128;
    let fee_growth_inside1_last_x128 = position_info.fee_growth_inside1_last_x128;

    let pool_id = cache_pool_key(
        pool,
        &PoolKey {
            token0: params.token0,
            token1: params.token1,
            fee: params.fee,
        },
    );

    save_position(
        &token_id,
        &Position {
            pool_id: pool_id,
            tick_lower: params.tick_lower,
            tick_upper: params.tick_upper,
            liquidity: liquidity.into(),
            fee_growth_inside0_last_x128,
            fee_growth_inside1_last_x128,
            tokens_owed0: 0.into(),
            tokens_owed1: 0.into(),
            fee: params.fee,
            token0: params.token0,
            token1: params.token1,
        },
    );

    casper_event_standard::emit(IncreaseLiquidity::new(
        token_id,
        liquidity.into(),
        amount0,
        amount1,
    ));

    (token_id, liquidity, amount0, amount1)
}

pub fn add_liquidity_internal(params: &AddLiquidityParams) -> (u128, U256, U256, Key) {
    let pool_key = get_pool_key(params.token0, params.token1, params.fee);

    let pool = get_pool_address(&pool_key);

    // compute liquidity amount
    let slot0: Slot0 = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "get_slot0",
        runtime_args! {},
    );
    let sqrt_ratio_a_x96 = tickmath::get_sqrt_ratio_at_tick(params.tick_lower);
    let sqrt_ratio_b_x96 = tickmath::get_sqrt_ratio_at_tick(params.tick_upper);

    let liquidity = get_liquidity_for_amounts(
        &slot0.sqrt_price_x96,
        &sqrt_ratio_a_x96,
        &sqrt_ratio_b_x96,
        &params.amount0_desired,
        &params.amount1_desired,
    );

    let (amount0, amount1): (U256, U256) = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "mint",
        runtime_args! {
            "recipient" => params.recipient,
            "tick_lower" => params.tick_lower,
            "tick_upper" => params.tick_upper,
            "amount" => U128::from(liquidity),
            "data" => Bytes::from(helpers::encode_1(&MintCallbackData{ pool_key, payer: helpers::get_immediate_caller_key()}))
        },
    );

    require(
        amount0 >= params.amount0_min && amount1 >= params.amount1_min,
        common::error::Error::ErrPriceSlippageCheck,
    );
    (liquidity, amount0, amount1, pool)
}

pub fn increase_liquidity_internal(params: &IncreaseLiquidityParams) -> (u128, U256, U256) {
    check_deadline(params.deadline);
    let mut position = read_position(&params.token_id);
    let pool_key = read_pool_key(&position.pool_id);
    let self_key = get_self_key();
    let (liquidity, amount0, amount1, pool) = add_liquidity_internal(&AddLiquidityParams {
        token0: pool_key.token0,
        token1: pool_key.token1,
        fee: pool_key.fee,
        recipient: self_key,
        tick_lower: position.tick_lower,
        tick_upper: position.tick_upper,
        amount0_desired: params.amount0_desired,
        amount1_desired: params.amount1_desired,
        amount0_min: params.amount0_min,
        amount1_min: params.amount1_min,
    });

    let position_key = position_key(self_key, position.tick_lower, position.tick_upper);
    let position_info: PositionInfo = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "get_position",
        runtime_args! {
            "position_key" => position_key
        },
    );

    position.tokens_owed0 = position
        .tokens_owed0
        .overflowing_add(
            fullmath::mul_div(
                &(position_info
                    .fee_growth_inside0_last_x128
                    .overflowing_sub(position.fee_growth_inside0_last_x128)
                    .0),
                &position.liquidity.as_u128().into(),
                &fixed_point_128::q128(),
            )
            .as_u128()
            .into(),
        )
        .0;

    position.tokens_owed1 = position
        .tokens_owed1
        .overflowing_add(
            fullmath::mul_div(
                &(position_info
                    .fee_growth_inside1_last_x128
                    .overflowing_sub(position.fee_growth_inside1_last_x128)
                    .0),
                &position.liquidity.as_u128().into(),
                &fixed_point_128::q128(),
            )
            .as_u128()
            .into(),
        )
        .0;

    position.fee_growth_inside0_last_x128 = position_info.fee_growth_inside0_last_x128;
    position.fee_growth_inside1_last_x128 = position_info.fee_growth_inside1_last_x128;
    position.liquidity += liquidity.into();

    save_position(&params.token_id, &position);
    casper_event_standard::emit(IncreaseLiquidity::new(
        params.token_id,
        liquidity.into(),
        amount0,
        amount1,
    ));

    (liquidity, amount0, amount1)
}

pub fn decrease_liquidity_internal(params: &DecreaseLiquidityParams) -> (U256, U256) {
    is_authorized_for_token(&params.token_id);
    check_deadline(params.deadline);

    require(
        params.liquidity.gt(&0.into()),
        common::error::Error::ErrInvalidLiquidity,
    );

    let mut position = read_position(&params.token_id);
    let position_liquidity = position.liquidity;

    require(
        position_liquidity >= params.liquidity,
        common::error::Error::ErrInvalidLiquidity,
    );

    let pool_key = read_pool_key(&position.pool_id);
    let pool = get_pool_address(&pool_key);
    let (amount0, amount1): (U256, U256) = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "burn",
        runtime_args! {
            "tick_lower" => position.tick_lower,
            "tick_upper" => position.tick_upper,
            "amount" => params.liquidity,
        },
    );
    require(
        amount0 >= params.amount0_min && amount1 >= params.amount1_min,
        common::error::Error::ErrPriceSlippageCheck,
    );

    let self_key = get_self_key();
    let position_key = position_key(self_key, position.tick_lower, position.tick_upper);
    let position_info: PositionInfo = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "get_position",
        runtime_args! {
            "position_key" => position_key
        },
    );

    position.tokens_owed0 = position
        .tokens_owed0
        .overflowing_add(amount0.as_u128().into())
        .0
        .overflowing_add(
            fullmath::mul_div(
                &(position_info
                    .fee_growth_inside0_last_x128
                    .overflowing_sub(position.fee_growth_inside0_last_x128)
                    .0),
                &position.liquidity.as_u128().into(),
                &fixed_point_128::q128(),
            )
            .as_u128()
            .into(),
        )
        .0;

    position.tokens_owed1 = position
        .tokens_owed1
        .overflowing_add(amount1.as_u128().into())
        .0
        .overflowing_add(
            fullmath::mul_div(
                &(position_info
                    .fee_growth_inside1_last_x128
                    .overflowing_sub(position.fee_growth_inside1_last_x128)
                    .0),
                &position.liquidity.as_u128().into(),
                &fixed_point_128::q128(),
            )
            .as_u128()
            .into(),
        )
        .0;

    position.fee_growth_inside0_last_x128 = position_info.fee_growth_inside0_last_x128;
    position.fee_growth_inside1_last_x128 = position_info.fee_growth_inside1_last_x128;
    position.liquidity = position_liquidity.overflowing_sub(params.liquidity).0;

    save_position(&params.token_id, &position);
    casper_event_standard::emit(DecreaseLiquidity::new(
        params.token_id,
        params.liquidity,
        amount0,
        amount1,
    ));
    (amount0, amount1)
}

pub fn collect_internal(params: &CollectParams) -> (Key, Key, U256, U256) {
    is_authorized_for_token(&params.token_id);

    require(
        params.amount0_max != U128::zero() && params.amount1_max != U128::zero(),
        common::error::Error::ErrInvalidCollectParams,
    );
    let self_key = get_self_key();

    let recipient = if params.recipient == null_key() {
        self_key
    } else {
        params.recipient
    };

    let mut position = read_position(&params.token_id);
    let pool_key = read_pool_key(&position.pool_id);
    let pool = get_pool_address(&pool_key);

    let (mut tokens_owed0, mut tokens_owed1) = (position.tokens_owed0, position.tokens_owed1);

    if position.liquidity != U128::zero() {
        let (_, _): (U256, U256) = runtime::call_versioned_contract(
            pool.into_hash().unwrap().into(),
            None,
            "burn",
            runtime_args! {
                "tick_lower" => position.tick_lower,
                "tick_upper" => position.tick_upper,
                "amount" => U128::zero(),
            },
        );

        let position_key = position_key(self_key, position.tick_lower, position.tick_upper);
        let position_info: PositionInfo = runtime::call_versioned_contract(
            pool.into_hash().unwrap().into(),
            None,
            "get_position",
            runtime_args! {
                "position_key" => position_key
            },
        );

        tokens_owed0 = tokens_owed0
            .overflowing_add(
                fullmath::mul_div(
                    &(position_info
                        .fee_growth_inside0_last_x128
                        .overflowing_sub(position.fee_growth_inside0_last_x128)
                        .0),
                    &position.liquidity.as_u128().into(),
                    &fixed_point_128::q128(),
                )
                .as_u128()
                .into(),
            )
            .0;

        tokens_owed1 = tokens_owed1
            .overflowing_add(
                fullmath::mul_div(
                    &(position_info
                        .fee_growth_inside1_last_x128
                        .overflowing_sub(position.fee_growth_inside1_last_x128)
                        .0),
                    &position.liquidity.as_u128().into(),
                    &fixed_point_128::q128(),
                )
                .as_u128()
                .into(),
            )
            .0;

        position.fee_growth_inside0_last_x128 = position_info.fee_growth_inside0_last_x128;
        position.fee_growth_inside1_last_x128 = position_info.fee_growth_inside1_last_x128;
    }

    let (amount0_collect, amount1_collect) = (
        if params.amount0_max > tokens_owed0 {
            tokens_owed0
        } else {
            params.amount0_max
        },
        if params.amount1_max > tokens_owed1 {
            tokens_owed1
        } else {
            params.amount1_max
        },
    );

    let (amount0, amount1): (U256, U256) = runtime::call_versioned_contract(
        pool.into_hash().unwrap().into(),
        None,
        "collect",
        runtime_args! {
            "recipient" => recipient,
            "tick_lower" => position.tick_lower,
            "tick_upper" => position.tick_upper,
            "amount0_requested" => amount0_collect,
            "amount1_requested" => amount1_collect,
        },
    );

    position.tokens_owed0 = tokens_owed0 - amount0_collect;
    position.tokens_owed1 = tokens_owed1 - amount1_collect;

    save_position(&params.token_id, &position);
    casper_event_standard::emit(Collect::new(params.token_id, recipient, amount0, amount1));

    (pool_key.token0, pool_key.token1, amount0, amount1)
}

pub fn burn_internal(token_id: U256) {
    is_authorized_for_token(&token_id);
    let position = read_position(&token_id);
    require(
        position.liquidity == U128::zero()
            && position.tokens_owed0 == U128::zero()
            && position.tokens_owed1 == U128::zero(),
        common::error::Error::ErrPositionNotCleared,
    );

    save_position(&token_id, &Position::default());

    NFTToken::default()
        .burn(
            NFTToken::default().owner_of(token_id).unwrap(),
            vec![token_id],
        )
        .unwrap_or_revert();
}

pub fn create_and_initialize_pool_if_necessary_internal(
    token0: Key,
    token1: Key,
    fee: u32,
    sqrt_price_x96: &U256,
) -> Key {
    require(
        is_token_sorted(token0, token1),
        common::error::Error::ErrInvalidTokenOrder,
    );
    let pool_key = get_pool_key(token0, token1, fee);
    let mut pool = get_pool_address(&pool_key);

    if pool == null_key() {
        pool = create_pool(read_factory(), token0, token1, fee);
        initialize_pool_price(pool, sqrt_price_x96);
    } else {
        let slot0: Slot0 = runtime::call_versioned_contract(
            pool.into_hash().unwrap().into(),
            None,
            "get_slot0",
            runtime_args! {},
        );
        if slot0.sqrt_price_x96.is_zero() {
            initialize_pool_price(pool, sqrt_price_x96);
        }
    }
    pool
}
