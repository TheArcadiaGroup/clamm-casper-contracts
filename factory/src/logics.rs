use core::ops::{Add, Shr};

use crate::{
    callbacks::{call_flash_callback, call_mint_callback, call_swap_callback},
    checks::{self, check_pool, check_ticks, only_factory_owner},
    oracle,
    position::{self, position_key},
    store::{
        self, read_fee, read_fee_growth_global0_x128, read_fee_growth_global1_x128, read_liquidity,
        read_max_liquidity_per_tick, read_protocol_fees, read_slot0, read_tick, read_tick_spacing,
        read_token0, read_token1, save_factory, save_fee, save_fee_growth_global0_x128,
        save_fee_growth_global1_x128, save_liquidity, save_max_liquidity_per_tick, save_position,
        save_protocol_fees, save_slot0, save_slot0_unlocked, save_tick_spacing, save_token0,
        save_token1,
    },
    tick, tick_bitmap,
};
use alloc::{format, string::ToString, vec, vec::Vec};
use casper_contract::contract_api::runtime;
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{bytesrepr::Bytes, CLValue, Key, U128, U256};
use common::{
    console, erc20_helpers,
    error::{require, Error},
    lock, pool_events,
};
use contract_utilities::helpers::{
    current_block_timestamp, encode_3, get_immediate_caller_key, get_named_args_2,
    get_named_args_3, get_named_args_4, get_named_args_5, get_named_args_6, get_self_key, set_key,
};
use math::{
    fixed_point_128, fullmath, liquidity_math, sqrt_price_math,
    swap_math::compute_swap_step,
    tickmath::{self, max_sqrt_ratio, MAX_TICK, MIN_TICK},
};
use types::{
    i256::I256, ModifyPositionParams, PositionInfo, ProtocolFees, Slot0, StepComputations,
    SwapCache, SwapState,
};
pub fn initialize(factory: Key, token0: Key, token1: Key, fee: u32, tick_spacing: i32) {
    store::initialize();
    save_tick_spacing(tick_spacing);
    save_max_liquidity_per_tick(tick::tick_spacing_to_max_liquidity_per_tick(tick_spacing).into());
    save_factory(factory);
    save_token0(token0);
    save_token1(token1);
    save_slot0_unlocked(false);
    save_fee(fee);
    set_key("is_pool", true);
    save_slot0(Slot0::default());
}

pub fn initialize_pool_price(sqrt_price_x96: U256) {
    let tick = tickmath::get_tick_at_sqrt_ratio(sqrt_price_x96);
    let (cardinality, cardinality_next) = oracle::initialize(current_block_timestamp());
    save_slot0(Slot0 {
        sqrt_price_x96,
        tick,
        observation_index: 0,
        observation_cardinality: cardinality,
        observation_cardinality_next: cardinality_next,
        fee_protocol: 0,
    });
    save_slot0_unlocked(true);

    casper_event_standard::emit(pool_events::Initialize::new(sqrt_price_x96, tick));
}

pub fn balance0() -> U256 {
    erc20_helpers::get_balance(read_token0(), get_self_key())
}

pub fn balance1() -> U256 {
    erc20_helpers::get_balance(read_token1(), get_self_key())
}

#[no_mangle]
pub extern "C" fn snapshot_cumulatives_inside() {
    check_pool();
    let tick_lower: i32 = runtime::get_named_arg("tick_lower");
    let tick_upper: i32 = runtime::get_named_arg("tick_upper");
    check_ticks(tick_lower, tick_upper);
    let lower = read_tick(&tick_lower);
    let upper = read_tick(&tick_upper);

    let tick_cumulative_lower;
    let tick_cumulative_upper;
    let seconds_per_liquidity_outside_lower_x128;
    let seconds_per_liquidity_outside_upper_x128;
    let seconds_outside_lower;
    let seconds_outside_upper;

    {
        let initialized_lower;
        (
            tick_cumulative_lower,
            seconds_per_liquidity_outside_lower_x128,
            seconds_outside_lower,
            initialized_lower,
        ) = (
            lower.tick_cumulative_outside,
            lower.seconds_per_liquidity_outside_x128,
            lower.seconds_outside,
            lower.initialized,
        );
        require(initialized_lower, Error::ErrLowerUninitialized);

        let initialized_upper;
        (
            tick_cumulative_upper,
            seconds_per_liquidity_outside_upper_x128,
            seconds_outside_upper,
            initialized_upper,
        ) = (
            upper.tick_cumulative_outside,
            upper.seconds_per_liquidity_outside_x128,
            upper.seconds_outside,
            upper.initialized,
        );
        require(initialized_upper, Error::ErrUpperUninitialized);
    }
    let slot0 = read_slot0();
    if slot0.tick < tick_lower {
        casper_event_standard::emit(pool_events::SnapshotCumulativesInside::new(
            tick_cumulative_lower
                .overflowing_sub(tick_cumulative_upper)
                .0,
            fullmath::overflow_sub_u160(
                &seconds_per_liquidity_outside_lower_x128,
                &seconds_per_liquidity_outside_upper_x128,
            ),
            seconds_outside_lower
                .overflowing_sub(seconds_outside_upper)
                .0,
        ));
        runtime::ret(
            CLValue::from_t((
                tick_cumulative_lower
                    .overflowing_sub(tick_cumulative_upper)
                    .0,
                fullmath::overflow_sub_u160(
                    &seconds_per_liquidity_outside_lower_x128,
                    &seconds_per_liquidity_outside_upper_x128,
                ),
                seconds_outside_lower
                    .overflowing_sub(seconds_outside_upper)
                    .0,
            ))
            .unwrap_or_revert(),
        );
    } else if slot0.tick < tick_upper {
        let time = current_block_timestamp();
        let (tick_cumulative, seconds_per_liquidity_cumulative_x128) = oracle::observe_single(
            time,
            0,
            slot0.tick,
            slot0.observation_index,
            read_liquidity().as_u128(),
            slot0.observation_cardinality,
        );
        casper_event_standard::emit(pool_events::SnapshotCumulativesInside::new(
            tick_cumulative
                .overflowing_sub(tick_cumulative_lower)
                .0
                .overflowing_sub(tick_cumulative_upper)
                .0,
            fullmath::overflow_sub_u160(
                &fullmath::overflow_sub_u160(
                    &seconds_per_liquidity_cumulative_x128,
                    &seconds_per_liquidity_outside_lower_x128,
                ),
                &seconds_per_liquidity_outside_upper_x128,
            ),
            (time as u32)
                .overflowing_sub(seconds_outside_lower)
                .0
                .overflowing_sub(seconds_outside_upper)
                .0,
        ));
        runtime::ret(
            CLValue::from_t((
                tick_cumulative
                    .overflowing_sub(tick_cumulative_lower)
                    .0
                    .overflowing_sub(tick_cumulative_upper)
                    .0,
                fullmath::overflow_sub_u160(
                    &fullmath::overflow_sub_u160(
                        &seconds_per_liquidity_cumulative_x128,
                        &seconds_per_liquidity_outside_lower_x128,
                    ),
                    &seconds_per_liquidity_outside_upper_x128,
                ),
                (time as u32)
                    .overflowing_sub(seconds_outside_lower)
                    .0
                    .overflowing_sub(seconds_outside_upper)
                    .0,
            ))
            .unwrap_or_revert(),
        );
    } else {
        console::log(&format!(
            "seconds_per_liquidity_outside_upper_x128 {:?}, {:?}",
            seconds_outside_upper, seconds_outside_lower
        ));
        let s = fullmath::overflow_sub_u160(
            &seconds_per_liquidity_outside_upper_x128,
            &seconds_per_liquidity_outside_lower_x128,
        );
        casper_event_standard::emit(pool_events::SnapshotCumulativesInside::new(
            tick_cumulative_upper
                .overflowing_sub(tick_cumulative_lower)
                .0,
            s,
            seconds_outside_upper
                .overflowing_sub(seconds_outside_lower)
                .0,
        ));
        console::log("snapshot_cumulatives_inside");
        runtime::ret(
            CLValue::from_t((
                tick_cumulative_upper
                    .overflowing_sub(tick_cumulative_lower)
                    .0,
                s,
                seconds_outside_upper
                    .overflowing_sub(seconds_outside_lower)
                    .0,
            ))
            .unwrap_or_revert(),
        );
    }
}

#[no_mangle]
pub extern "C" fn observe() {
    check_pool();
    let seconds_agos: Vec<u32> = runtime::get_named_arg("seconds_agos");
    let slot0 = read_slot0();
    runtime::ret(
        CLValue::from_t(oracle::observe(
            current_block_timestamp(),
            &seconds_agos,
            slot0.tick,
            slot0.observation_index,
            read_liquidity().as_u128(),
            slot0.observation_cardinality,
        ))
        .unwrap_or_revert(),
    );
}

#[no_mangle]
pub extern "C" fn increase_observation_cardinality_next() {
    check_pool();
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();
    let observation_cardinality_next: u32 = runtime::get_named_arg("observation_cardinality_next");
    let mut slot0 = read_slot0();
    let observation_cardinality_next_old = slot0.observation_cardinality_next;
    let observation_cardinality_next_new = oracle::grow(
        observation_cardinality_next_old,
        observation_cardinality_next as u16,
    );
    slot0.observation_cardinality_next = observation_cardinality_next_new;
    if observation_cardinality_next_old != observation_cardinality_next_new {
        casper_event_standard::emit(pool_events::IncreaseObservationCardinalityNext::new(
            observation_cardinality_next_old.into(),
            observation_cardinality_next,
        ));
    }
    save_slot0(slot0);
    lock::unlock_contract();
}

#[no_mangle]
pub extern "C" fn mint() {
    check_pool();
    let (recipient, tick_lower, tick_upper, amount, data): (Key, i32, i32, U128, Bytes) =
        get_named_args_5(
            vec!["recipient", "tick_lower", "tick_upper", "amount", "data"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        );
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();
    let amount = amount.as_u128();
    require(amount > 0, Error::ErrMintAmount);
    let (_, amount0_int, amount1_int) = _modify_position(&ModifyPositionParams {
        owner: recipient,
        tick_lower,
        tick_upper,
        liquidity_delta: amount as i128,
    });
    let (amount0, amount1) = (U256::from(amount0_int), U256::from(amount1_int));
    let balance0_before = if amount0 > U256::zero() {
        balance0()
    } else {
        U256::zero()
    };
    let balance1_before = if amount1 > U256::zero() {
        balance1()
    } else {
        U256::zero()
    };
    // call mint call back
    call_mint_callback(amount0, amount1, data.to_vec());

    if amount0 > U256::zero() {
        require(balance0_before.add(amount0) <= balance0(), Error::ErrMintM0);
    }

    if amount1 > U256::zero() {
        require(balance1_before.add(amount1) <= balance1(), Error::ErrMintM1);
    }

    // emit mint
    casper_event_standard::emit(pool_events::Mint::new(
        get_immediate_caller_key(),
        recipient,
        tick_lower,
        tick_upper,
        amount.into(),
        amount0,
        amount1,
    ));
    lock::unlock_contract();
    // console::log("done updating position 1");
    runtime::ret(CLValue::from_t((amount0, amount1)).unwrap_or_revert())
}

// internal functions
pub fn _modify_position(params: &ModifyPositionParams) -> (PositionInfo, I256, I256) {
    check_ticks(params.tick_lower, params.tick_upper);

    let mut slot0 = read_slot0();
    // console::log("updating position");
    let position: PositionInfo = _update_position(
        params.owner,
        params.tick_lower,
        params.tick_upper,
        params.liquidity_delta,
        slot0.tick,
    );
    let mut amount0 = I256::default();
    let mut amount1 = I256::default();

    if params.liquidity_delta != 0 {
        if slot0.tick < params.tick_lower {
            amount0 = sqrt_price_math::get_amount0_delta_2(
                &tickmath::get_sqrt_ratio_at_tick(params.tick_lower),
                &tickmath::get_sqrt_ratio_at_tick(params.tick_upper),
                params.liquidity_delta,
            );
        } else if slot0.tick < params.tick_upper {
            let liquidity_before = read_liquidity().as_u128();
            (slot0.observation_index, slot0.observation_cardinality) = oracle::write(
                slot0.observation_index,
                current_block_timestamp(),
                slot0.tick,
                liquidity_before,
                slot0.observation_cardinality,
                slot0.observation_cardinality_next,
            );
            amount0 = sqrt_price_math::get_amount0_delta_2(
                &slot0.sqrt_price_x96,
                &tickmath::get_sqrt_ratio_at_tick(params.tick_upper),
                params.liquidity_delta,
            );
            amount1 = sqrt_price_math::get_amount1_delta_2(
                &tickmath::get_sqrt_ratio_at_tick(params.tick_lower),
                &slot0.sqrt_price_x96,
                params.liquidity_delta,
            );
            save_liquidity(
                liquidity_math::add_delta(liquidity_before, params.liquidity_delta).into(),
            );
        } else {
            amount1 = sqrt_price_math::get_amount1_delta_2(
                &tickmath::get_sqrt_ratio_at_tick(params.tick_lower),
                &tickmath::get_sqrt_ratio_at_tick(params.tick_upper),
                params.liquidity_delta,
            );
        }
    }
    (position, amount0, amount1)
}

pub fn _update_position(
    owner: Key,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
    tick: i32,
) -> PositionInfo {
    let mut position = position::get(owner, tick_lower, tick_upper);
    let _fee_growth_global0_x128 = read_fee_growth_global0_x128();
    let _fee_growth_global1_x128 = read_fee_growth_global1_x128();
    let mut flipped_lower = false;
    let mut flipped_upper = false;
    let slot0 = read_slot0();
    if liquidity_delta != 0 {
        let time = current_block_timestamp();
        let (tick_cumulative, seconds_per_liquidity_cumulative_x128) = oracle::observe_single(
            time,
            0,
            slot0.tick,
            slot0.observation_index,
            read_liquidity().as_u128(),
            slot0.observation_cardinality,
        );
        let max_liquidity_per_tick = read_max_liquidity_per_tick().as_u128();
        flipped_lower = tick::update(
            tick_lower,
            tick,
            liquidity_delta,
            _fee_growth_global0_x128,
            _fee_growth_global1_x128,
            seconds_per_liquidity_cumulative_x128,
            tick_cumulative,
            time,
            false,
            max_liquidity_per_tick,
        );
        flipped_upper = tick::update(
            tick_upper,
            tick,
            liquidity_delta,
            _fee_growth_global0_x128,
            _fee_growth_global1_x128,
            seconds_per_liquidity_cumulative_x128,
            tick_cumulative,
            time,
            true,
            max_liquidity_per_tick,
        );
        let tick_spacing = read_tick_spacing();
        if flipped_lower {
            tick_bitmap::flip_tick(tick_lower, tick_spacing);
        }

        if flipped_upper {
            tick_bitmap::flip_tick(tick_upper, tick_spacing);
        }
    }
    let (fee_growth_inside0_x128, fee_growth_inside1_x128) = tick::get_fee_growth_inside(
        tick_lower,
        tick_upper,
        tick,
        _fee_growth_global0_x128,
        _fee_growth_global1_x128,
    );
    position::update(
        position_key(owner, tick_lower, tick_upper),
        &mut position,
        liquidity_delta,
        &fee_growth_inside0_x128,
        &fee_growth_inside1_x128,
    );
    if liquidity_delta < 0 {
        if flipped_lower {
            tick::clear(tick_lower);
        }

        if flipped_upper {
            tick::clear(tick_upper);
        }
    }
    position
}

#[no_mangle]
pub extern "C" fn collect() {
    check_pool();
    let (recipient, tick_lower, tick_upper, amount0_requested, amount1_requested): (
        Key,
        i32,
        i32,
        U128,
        U128,
    ) = get_named_args_5(
        vec![
            "recipient",
            "tick_lower",
            "tick_upper",
            "amount0_requested",
            "amount1_requested",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect(),
    );
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();

    let mut position = position::get(get_immediate_caller_key(), tick_lower, tick_upper);
    let amount0 = if amount0_requested > position.tokens_owed0 {
        position.tokens_owed0
    } else {
        amount0_requested
    };
    let amount1 = if amount1_requested > position.tokens_owed1 {
        position.tokens_owed1
    } else {
        amount1_requested
    };

    if amount0 > U128::zero() {
        position.tokens_owed0 -= amount0;
        erc20_helpers::transfer(read_token0(), recipient, amount0.as_u128().into());
    }

    if amount1 > U128::zero() {
        position.tokens_owed1 -= amount1;
        erc20_helpers::transfer(read_token1(), recipient, amount1.as_u128().into());
    }

    casper_event_standard::emit(pool_events::Collect::new(
        get_immediate_caller_key(),
        recipient,
        tick_lower,
        tick_upper,
        amount0.as_u128().into(),
        amount1.as_u128().into(),
    ));
    lock::unlock_contract();
    runtime::ret(CLValue::from_t((amount0, amount1)).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn burn() {
    check_pool();
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();
    let (tick_lower, tick_upper, amount): (i32, i32, U128) = get_named_args_3(
        vec!["tick_lower", "tick_upper", "amount"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
    );
    let (mut position, amount0_int, amount1_int) = _modify_position(&ModifyPositionParams {
        owner: get_immediate_caller_key(),
        tick_lower,
        tick_upper,
        liquidity_delta: (-I256::from(U256::from(amount.as_u128()))).0.as_i128(),
    });
    let amount0 = U256::from(-amount0_int);
    let amount1 = U256::from(-amount1_int);
    if amount0 > U256::zero() || amount1 > U256::zero() {
        (position.tokens_owed0, position.tokens_owed1) = (
            position
                .tokens_owed0
                .overflowing_add(U128::from(amount0.as_u128()))
                .0,
            position
                .tokens_owed1
                .overflowing_add(U128::from(amount1.as_u128()))
                .0,
        );
        save_position(
            &position_key(get_immediate_caller_key(), tick_lower, tick_upper),
            &position,
        );
    }
    casper_event_standard::emit(pool_events::Burn::new(
        get_immediate_caller_key(),
        tick_lower,
        tick_upper,
        amount,
        amount0,
        amount1,
    ));

    lock::unlock_contract();
    runtime::ret(CLValue::from_t((amount0, amount1)).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn swap() {
    check_pool();
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();
    let (
        recipient,
        zero_for_one,
        amount_specified,
        is_amount_specified_positive,
        sqrt_price_limit_x96,
        data,
    ): (Key, bool, U256, bool, U256, Bytes) = get_named_args_6(
        vec![
            "recipient",
            "zero_for_one",
            "amount_specified",
            "is_amount_specified_positive",
            "sqrt_price_limit_x96",
            "data",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect(),
    );
    let data: Vec<u8> = data.to_vec();
    let amount_specified = if is_amount_specified_positive {
        I256::from(amount_specified)
    } else {
        -I256::from(amount_specified)
    };

    require(amount_specified != I256::from(0), Error::ErrSwapAS);
    let slot0_start = read_slot0();
    require(
        if zero_for_one {
            sqrt_price_limit_x96 < slot0_start.sqrt_price_x96
                && sqrt_price_limit_x96 > tickmath::min_sqrt_ratio()
        } else {
            sqrt_price_limit_x96 > slot0_start.sqrt_price_x96
                && sqrt_price_limit_x96 < max_sqrt_ratio()
        },
        Error::ErrSwapSPL,
    );
    let caller = get_immediate_caller_key();

    let mut slot0 = read_slot0();
    let mut cache = SwapCache {
        liquidity_start: read_liquidity().as_u128(),
        block_time: current_block_timestamp() as u32,
        fee_protocol: if zero_for_one {
            slot0_start.fee_protocol % 16
        } else {
            slot0_start.fee_protocol.shr(4)
        },
        seconds_per_liquidity_cumulative_x128: 0.into(),
        tick_cumulative: 0,
        computed_latest_observation: false,
    };
    let exact_in = amount_specified.0.is_positive();
    let mut state = SwapState {
        amount_specified_remaining: amount_specified,
        amount_calculated: I256::from(0),
        sqrt_price_x96: slot0_start.sqrt_price_x96,
        tick: slot0_start.tick,
        fee_growth_global_x128: if zero_for_one {
            read_fee_growth_global0_x128()
        } else {
            read_fee_growth_global1_x128()
        },
        protocol_fee: 0,
        liquidity: cache.liquidity_start,
    };
    let tick_spacing = read_tick_spacing();
    while state.amount_specified_remaining != I256::from(0)
        && state.sqrt_price_x96 != sqrt_price_limit_x96
    {
        let mut step = StepComputations {
            sqrt_price_start_x96: state.sqrt_price_x96,
            ..Default::default()
        };
        (step.tick_next, step.initialized) =
            tick_bitmap::tick_next_initialized_tick_within_one_word(
                state.tick,
                tick_spacing,
                zero_for_one,
            );
        if step.tick_next < MIN_TICK {
            step.tick_next = MIN_TICK;
        } else if step.tick_next > MAX_TICK {
            step.tick_next = MAX_TICK;
        };

        step.sqrt_price_next_x96 = tickmath::get_sqrt_ratio_at_tick(step.tick_next);
        let cond = if zero_for_one {
            step.sqrt_price_next_x96 < sqrt_price_limit_x96
        } else {
            step.sqrt_price_next_x96 > sqrt_price_limit_x96
        };
        (
            state.sqrt_price_x96,
            step.amount_in,
            step.amount_out,
            step.fee_amount,
        ) = compute_swap_step(
            &state.sqrt_price_x96,
            if cond {
                &sqrt_price_limit_x96
            } else {
                &step.sqrt_price_next_x96
            },
            state.liquidity,
            state.amount_specified_remaining,
            read_fee().into(),
        );
        if exact_in {
            state.amount_specified_remaining =
                state.amount_specified_remaining - I256::from(step.amount_in + step.fee_amount);
            state.amount_calculated = state.amount_calculated - I256::from(step.amount_out);
        } else {
            state.amount_specified_remaining =
                state.amount_specified_remaining + I256::from(step.amount_out);
            state.amount_calculated =
                state.amount_calculated + I256::from(step.amount_in + step.fee_amount);
        }
        // if the protocol fee is on, calculate how much is owed, decrement feeAmount, and increment protocolFee
        if cache.fee_protocol > 0 {
            let delta: U256 = step.fee_amount / cache.fee_protocol;
            step.fee_amount -= delta;
            state.protocol_fee += delta.as_u128()
        }
        // update global fee tracker
        if state.liquidity > 0 {
            state.fee_growth_global_x128 = state
                .fee_growth_global_x128
                .overflowing_add(fullmath::mul_div(
                    &step.fee_amount,
                    &fixed_point_128::q128(),
                    &state.liquidity.into(),
                ))
                .0;
        }
        // shift tick if we reached the next price
        if state.sqrt_price_x96 == step.sqrt_price_next_x96 {
            // if the tick is initialized, run the tick transition
            if step.initialized {
                // check for the placeholder value, which we replace with the actual value the first time the swap
                // crosses an initialized tick
                if !cache.computed_latest_observation {
                    (
                        cache.tick_cumulative,
                        cache.seconds_per_liquidity_cumulative_x128,
                    ) = oracle::observe_single(
                        cache.block_time.into(),
                        0,
                        slot0_start.tick,
                        slot0_start.observation_index,
                        cache.liquidity_start,
                        slot0_start.observation_cardinality,
                    );
                    cache.computed_latest_observation = true;
                }
                let mut liquidity_net = tick::cross(
                    step.tick_next,
                    if zero_for_one {
                        state.fee_growth_global_x128
                    } else {
                        read_fee_growth_global0_x128()
                    },
                    if zero_for_one {
                        read_fee_growth_global1_x128()
                    } else {
                        state.fee_growth_global_x128
                    },
                    cache.seconds_per_liquidity_cumulative_x128,
                    cache.tick_cumulative,
                    cache.block_time.into(),
                );
                // if we're moving leftward, we interpret liquidityNet as the opposite sign
                // safe because liquidityNet cannot be type(int128).min
                if zero_for_one {
                    liquidity_net = -liquidity_net
                }
                state.liquidity = liquidity_math::add_delta(state.liquidity, liquidity_net);
            }
            state.tick = if zero_for_one {
                step.tick_next - 1
            } else {
                step.tick_next
            };
        } else if state.sqrt_price_x96 != step.sqrt_price_start_x96 {
            state.tick = tickmath::get_tick_at_sqrt_ratio(state.sqrt_price_x96);
        }
    }
    // update tick and write an oracle entry if the tick change
    if state.tick != slot0_start.tick {
        let (observation_index, observation_cardinality) = oracle::write(
            slot0_start.observation_index,
            cache.block_time.into(),
            slot0_start.tick,
            cache.liquidity_start,
            slot0_start.observation_cardinality,
            slot0_start.observation_cardinality_next,
        );
        (
            slot0.sqrt_price_x96,
            slot0.tick,
            slot0.observation_index,
            slot0.observation_cardinality,
        ) = (
            state.sqrt_price_x96,
            state.tick,
            observation_index,
            observation_cardinality,
        );
    } else {
        // otherwise just update the price
        slot0.sqrt_price_x96 = state.sqrt_price_x96;
    }
    // update liquidity if it changed
    if cache.liquidity_start != state.liquidity {
        save_liquidity(state.liquidity.into());
    }
    // update fee growth global and, if necessary, protocol fees
    // overflow is acceptable, protocol has to withdraw before it hits type(uint128).max fees
    let protocol_fees = read_protocol_fees();
    if zero_for_one {
        save_fee_growth_global0_x128(state.fee_growth_global_x128);
        if state.protocol_fee > 0 {
            save_protocol_fees(ProtocolFees {
                token0: protocol_fees.token0 + state.protocol_fee,
                token1: protocol_fees.token1,
            });
        }
    } else {
        save_fee_growth_global1_x128(state.fee_growth_global_x128);
        if state.protocol_fee > 0 {
            save_protocol_fees(ProtocolFees {
                token0: protocol_fees.token0,
                token1: protocol_fees.token1 + state.protocol_fee,
            })
        }
    }

    let (amount0, amount1) = if zero_for_one == exact_in {
        (
            amount_specified - state.amount_specified_remaining,
            state.amount_calculated,
        )
    } else {
        (
            state.amount_calculated,
            amount_specified - state.amount_specified_remaining,
        )
    };
    // do the transfers and collect payment
    if zero_for_one {
        if amount1.0.is_negative() {
            erc20_helpers::transfer(read_token1(), recipient, U256::from(-amount1));
        }
        let balance0_before = balance0();
        call_swap_callback(amount0, amount1, data);
        require(
            balance0_before + U256::from(amount0) <= balance0(),
            Error::ErrIIA,
        );
    } else {
        if amount0.0.is_negative() {
            erc20_helpers::transfer(read_token0(), recipient, U256::from(-amount0));
        }
        let balance1_before = balance1();
        call_swap_callback(amount0, amount1, data);
        require(
            balance1_before + U256::from(amount1) <= balance1(),
            Error::ErrIIA,
        );
    }
    casper_event_standard::emit(pool_events::Swap::new(
        caller,
        recipient,
        amount0,
        amount1,
        state.sqrt_price_x96,
        state.liquidity.into(),
        state.tick,
    ));
    save_slot0(slot0);
    lock::unlock_contract();

    runtime::ret(CLValue::from_t((amount0, amount1)).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn flash() {
    check_pool();
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();
    let (recipient, amount0, amount1, data): (Key, U256, U256, Bytes) = get_named_args_4(
        vec!["recipient", "amount0", "amount1", "data"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
    );
    let data = data.to_vec();
    let caller = get_immediate_caller_key();
    let _liquidity = read_liquidity();
    require(_liquidity > U128::zero(), Error::ErrL);
    let const_denom = U256::from("1000000");
    let fee0 = fullmath::mul_div_rounding_up(&amount0, &read_fee().into(), &const_denom);
    let fee1 = fullmath::mul_div_rounding_up(&amount1, &read_fee().into(), &const_denom);
    let balance0_before = balance0();
    let balance1_before = balance1();
    if amount0 > U256::zero() {
        erc20_helpers::transfer(read_token0(), recipient, amount0);
    }
    if amount1 > U256::zero() {
        erc20_helpers::transfer(read_token1(), recipient, amount1);
    }
    call_flash_callback(fee0, fee1, data);
    let balance0_after = balance0();
    let balance1_after = balance1();
    require(balance0_before + fee0 <= balance0_after, Error::ErrF0);
    require(balance1_before + fee1 <= balance1_after, Error::ErrF1);
    // sub is safe because we know balanceAfter is gt balanceBefore by at least fee
    let paid0 = balance0_after - balance0_before;
    let paid1 = balance1_after - balance1_before;
    let protocol_fees = read_protocol_fees();
    let slot0 = read_slot0();
    if paid0 > U256::zero() {
        let fee_protocol0 = slot0.fee_protocol % 16;
        let fees0: U256 = if fee_protocol0 == 0 {
            U256::zero()
        } else {
            paid0 / fee_protocol0
        };
        if fees0 > U256::zero() {
            save_protocol_fees(ProtocolFees {
                token0: protocol_fees.token0 + fees0.as_u128(),
                token1: protocol_fees.token1,
            });
        }
        console::log("here");
        save_fee_growth_global0_x128(
            read_fee_growth_global0_x128()
                .overflowing_add(fullmath::mul_div(
                    &(paid0 - fees0),
                    &fixed_point_128::q128(),
                    &U256::from(_liquidity.as_u128()),
                ))
                .0,
        );
    }
    if paid1 > U256::zero() {
        let fee_protocol1 = slot0.fee_protocol >> 4;
        let fees1: U256 = if fee_protocol1 == 0 {
            U256::zero()
        } else {
            paid1 / fee_protocol1
        };
        if fees1 > U256::zero() {
            save_protocol_fees(ProtocolFees {
                token0: protocol_fees.token0,
                token1: protocol_fees.token1 + fees1.as_u128(),
            });
        }
        save_fee_growth_global1_x128(
            read_fee_growth_global1_x128()
                .overflowing_add(fullmath::mul_div(
                    &(paid1 - fees1),
                    &fixed_point_128::q128(),
                    &U256::from(_liquidity.as_u128()),
                ))
                .0,
        );
    }
    casper_event_standard::emit(pool_events::Flash::new(
        caller, recipient, amount0, amount1, paid0, paid1,
    ));
    lock::unlock_contract();
}

#[no_mangle]
pub extern "C" fn set_fee_protocol() {
    check_pool();
    only_factory_owner();
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();

    let (fee_protocol0, fee_protocol1): (u8, u8) = get_named_args_2(vec![
        "fee_protocol0".to_string(),
        "fee_protocol1".to_string(),
    ]);
    require(
        (fee_protocol0 == 0 || (4..=10).contains(&fee_protocol0))
            && (fee_protocol1 == 0 || (4..=10).contains(&fee_protocol1)),
        Error::ErrFeeProtocol,
    );
    let mut slot0 = read_slot0();
    let fee_protocol_old = slot0.fee_protocol;
    slot0.fee_protocol = fee_protocol0 + (fee_protocol1 << 4);
    save_slot0(slot0);
    casper_event_standard::emit(pool_events::SetFeeProtocol::new(
        fee_protocol_old % 16,
        fee_protocol_old >> 4,
        fee_protocol0,
        fee_protocol1,
    ));
    lock::unlock_contract();
}

#[no_mangle]
pub extern "C" fn collect_protocol() {
    check_pool();
    only_factory_owner();
    checks::check_slot0_unlocked();
    lock::when_not_locked();
    lock::lock_contract();
    let (recipient, amount0_requested, amount1_requested): (Key, U128, U128) = get_named_args_3(
        vec!["recipient", "amount0_requested", "amount1_requested"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
    );
    let protocol_fees = read_protocol_fees();
    let mut amount0 = if amount0_requested > protocol_fees.token0 {
        protocol_fees.token0
    } else {
        amount0_requested
    };
    let mut amount1 = if amount1_requested > protocol_fees.token1 {
        protocol_fees.token1
    } else {
        amount1_requested
    };
    if amount0 > 0_u128.into() {
        if amount0 == protocol_fees.token0 {
            amount0 -= 1_u128.into();
        }
        save_protocol_fees(ProtocolFees {
            token0: protocol_fees.token0 - amount0,
            token1: protocol_fees.token1,
        });
        erc20_helpers::transfer(read_token0(), recipient, U256::from(amount0.as_u128()));
    }
    if amount1 > 0_u128.into() {
        if amount1 == protocol_fees.token1 {
            amount1 -= 1_u128.into();
        }
        save_protocol_fees(ProtocolFees {
            token0: protocol_fees.token0,
            token1: protocol_fees.token1 - amount1,
        });
        erc20_helpers::transfer(read_token1(), recipient, U256::from(amount1.as_u128()));
    }
    let caller = get_immediate_caller_key();
    casper_event_standard::emit(pool_events::CollectProtocol::new(
        caller, recipient, amount0, amount1,
    ));
    lock::unlock_contract();
}

#[no_mangle]
pub extern "C" fn get_position_key() {
    console::log("get_position_key 0");
    let (owner, tick_lower, tick_upper): (Key, i32, i32) = get_named_args_3(
        vec!["owner", "tick_lower", "tick_upper"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
    );
    console::log("get_position_key 1");
    let encoded = encode_3(&owner, &tick_lower, &tick_upper);
    let hash = runtime::blake2b(encoded);
    runtime::ret(CLValue::from_t(hex::encode(hash)).unwrap_or_revert())
}
