use core::ops::{Div, Shl};

use alloc::{format, vec, vec::Vec};
use casper_types::U256;
use common::{console, error::require};
use math::safe_cast;
use types::Observation;

use crate::store::{read_observation, save_observation};

pub fn transform(last: &Observation, block_time: u64, tick: i32, liquidity: u128) -> Observation {
    let delta = block_time - last.block_timestamp;
    console::log(&format!(
        "updating secondsPerLiquidityCumulativeX128 {:?}, liquidity {:?}",
        last.seconds_per_liquidity_cumulative_x128, liquidity
    ));
    Observation {
        block_timestamp: block_time,
        tick_cumulative: last.tick_cumulative + (tick as i64) * (delta as i64),
        seconds_per_liquidity_cumulative_x128: last.seconds_per_liquidity_cumulative_x128
            + U256::from(delta).shl(128).div(if liquidity > 0 {
                liquidity.into()
            } else {
                U256::one()
            }),
        initialized: true,
    }
}

pub fn initialize(time: u64) -> (u16, u16) {
    save_observation(
        &0_u64,
        &Observation {
            block_timestamp: time,
            tick_cumulative: 0,
            seconds_per_liquidity_cumulative_x128: U256::zero(),
            initialized: true,
        },
    );
    (1, 1)
}

pub fn write(
    index: u16,
    block_time: u64,
    tick: i32,
    liquidity: u128,
    cardinality: u16,
    cardinality_next: u16,
) -> (u16, u16) {
    let last = read_observation(&index.into());
    if last.block_timestamp == block_time {
        return (index, cardinality);
    }

    let cardinality_updated = if cardinality_next > cardinality && index == cardinality - 1 {
        cardinality_next
    } else {
        cardinality
    };

    let index_updated = (index + 1) % cardinality_updated;
    console::log(&format!("calling transform 2 {:?}", liquidity));
    save_observation(
        &index_updated.into(),
        &transform(&last, block_time, tick, liquidity),
    );
    (index_updated, cardinality_updated)
}

pub fn grow(current: u16, next: u16) -> u16 {
    require(current > 0, common::error::Error::ErrI);
    if next <= current {
        return current;
    }

    for i in current..next {
        let mut obs = read_observation(&i.into());
        obs.block_timestamp = 1;
        save_observation(&i.into(), &obs);
    }
    next
}

pub fn lte(time: u64, a: u32, b: u32) -> bool {
    if a <= time as u32 && b <= time as u32 {
        return a <= b;
    }

    let a_adjusted: U256 = if a > time as u32 {
        a.into()
    } else {
        U256::one().shl(32) + U256::from(a)
    };
    let b_adjusted: U256 = if b > time as u32 {
        b.into()
    } else {
        U256::one().shl(32) + U256::from(b)
    };
    a_adjusted <= b_adjusted
}

pub fn binary_search(
    time: u64,
    target: u32,
    index: u16,
    cardinality: u16,
) -> (Observation, Observation) {
    let mut l = ((index + 1) % cardinality) as u64;
    let mut r = l + cardinality as u64 - 1;
    let mut i;
    let mut before_or_at;
    let mut at_or_after;
    loop {
        i = (l + r) / 2;
        before_or_at = read_observation(&(i % cardinality as u64));
        if !before_or_at.initialized {
            l = i + 1;
            continue;
        }
        at_or_after = read_observation(&((i + 1) % cardinality as u64));
        let target_at_or_after = lte(time, before_or_at.block_timestamp as u32, target);
        if target_at_or_after && lte(time, target, at_or_after.block_timestamp as u32) {
            break;
        }

        if !target_at_or_after {
            r = i - 1;
        } else {
            l = i + 1;
        }
    }
    (before_or_at, at_or_after)
}

pub fn get_surrounding_observations(
    time: u64,
    target: u32,
    tick: i32,
    index: u16,
    liquidity: u128,
    cardinality: u16,
) -> (Observation, Observation) {
    let mut before_or_at = read_observation(&index.into());
    if lte(time, before_or_at.block_timestamp as u32, target) {
        if before_or_at.block_timestamp == target as u64 {
            return (before_or_at, Observation::default());
        } else {
            return (
                before_or_at.clone(),
                transform(&before_or_at, target.into(), tick, liquidity),
            );
        }
    }

    before_or_at = read_observation(&((index + 1) % cardinality).into());
    if !before_or_at.initialized {
        before_or_at = read_observation(&0);
    }

    require(
        lte(time, before_or_at.block_timestamp as u32, target),
        common::error::Error::ErrOld,
    );
    binary_search(time, target, index, cardinality)
}

pub fn observe_single(
    time: u64,
    seconds_ago: u64,
    tick: i32,
    index: u16,
    liquidity: u128,
    cardinality: u16,
) -> (i64, U256) {
    if seconds_ago == 0 {
        let mut last = read_observation(&index.into());
        if last.block_timestamp != time {
            console::log("calling transform");
            last = transform(&last, time, tick, liquidity);
        }
        return (
            last.tick_cumulative,
            last.seconds_per_liquidity_cumulative_x128,
        );
    }

    let target = time - seconds_ago;
    let (before_or_at, at_or_after) =
        get_surrounding_observations(time, target as u32, tick, index, liquidity, cardinality);
    if target == before_or_at.block_timestamp {
        (
            before_or_at.tick_cumulative,
            before_or_at.seconds_per_liquidity_cumulative_x128,
        )
    } else if target == at_or_after.block_timestamp {
        (
            at_or_after.tick_cumulative,
            at_or_after.seconds_per_liquidity_cumulative_x128,
        )
    } else {
        let observation_time_delta = at_or_after.block_timestamp - before_or_at.block_timestamp;
        let target_delta = target - before_or_at.block_timestamp;
        (
            before_or_at.tick_cumulative
                + ((at_or_after.tick_cumulative - before_or_at.tick_cumulative)
                    / observation_time_delta as i64)
                    * target_delta as i64,
            before_or_at.seconds_per_liquidity_cumulative_x128
                + safe_cast::to_u160(
                    ((at_or_after.seconds_per_liquidity_cumulative_x128
                        - before_or_at.seconds_per_liquidity_cumulative_x128)
                        * target_delta)
                        / observation_time_delta,
                ),
        )
    }
}

pub fn observe(
    time: u64,
    seconds_ago: &[u32],
    tick: i32,
    index: u16,
    liquidity: u128,
    cardinality: u16,
) -> (Vec<i64>, Vec<U256>) {
    require(cardinality > 0, common::error::Error::ErrI);
    let mut tick_cumulatives = vec![0_i64; seconds_ago.len()];
    let mut seconds_per_liquidity_cumulative_x128s = vec![U256::zero(); seconds_ago.len()];
    for i in 0..seconds_ago.len() {
        (
            tick_cumulatives[i],
            seconds_per_liquidity_cumulative_x128s[i],
        ) = observe_single(
            time,
            seconds_ago[i] as u64,
            tick,
            index,
            liquidity,
            cardinality,
        );
    }

    (tick_cumulatives, seconds_per_liquidity_cumulative_x128s)
}
