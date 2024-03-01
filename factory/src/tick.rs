use crate::store::{read_tick, save_tick};
use casper_types::U256;
use common::error::require;
use math::{liquidity_math, tickmath};
use types::i128::I128;
use types::TickInfo;

pub fn tick_spacing_to_max_liquidity_per_tick(tick_spacing: i32) -> u128 {
    let min_tick = (tickmath::MIN_TICK / tick_spacing) * tick_spacing;
    let max_tick = (tickmath::MAX_TICK / tick_spacing) * tick_spacing;
    let num_ticks = ((max_tick - min_tick) / tick_spacing) as u32 + 1;
    u128::MAX / num_ticks as u128
}

pub fn get_fee_growth_inside(
    tick_lower: i32,
    tick_upper: i32,
    tick_current: i32,
    fee_growth_global0_x128: U256,
    fee_growth_global1_x128: U256,
) -> (U256, U256) {
    let lower = read_tick(&tick_lower);
    let upper = read_tick(&tick_upper);
    let (fee_growth_below0_x128, fee_growth_below1_x128) = if tick_current >= tick_lower {
        (
            lower.fee_growth_outside0_x128,
            lower.fee_growth_outside1_x128,
        )
    } else {
        (
            fee_growth_global0_x128 - lower.fee_growth_outside0_x128,
            fee_growth_global1_x128 - lower.fee_growth_outside1_x128,
        )
    };
    let (fee_growth_above0_x128, fee_growth_above1_x128) = if tick_current < tick_upper {
        (
            upper.fee_growth_outside0_x128,
            upper.fee_growth_outside1_x128,
        )
    } else {
        (
            fee_growth_global0_x128 - upper.fee_growth_outside0_x128,
            fee_growth_global1_x128 - upper.fee_growth_outside1_x128,
        )
    };
    (
        fee_growth_global0_x128 - fee_growth_below0_x128 - fee_growth_above0_x128,
        fee_growth_global1_x128 - fee_growth_below1_x128 - fee_growth_above1_x128,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    tick: i32,
    tick_current: i32,
    liquidity_delta: i128,
    fee_growth_global0_x128: U256,
    fee_growth_global1_x128: U256,
    seconds_per_liquidity_cumulative_x128: U256,
    tick_cumulative: i64,
    time: u64,
    upper: bool,
    max_liquidity: u128,
) -> bool {
    let mut info = read_tick(&tick);
    let liquidity_gross_before = info.liquidity_gross.as_u128();
    // console::log(&format!("start updating tick {:?}, {:?}, liquidity_gross_before {:?}", tick, upper, liquidity_gross_before));
    let liquidity_gross_after = liquidity_math::add_delta(liquidity_gross_before, liquidity_delta);

    require(
        liquidity_gross_after <= max_liquidity,
        common::error::Error::ErrLO,
    );
    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    if liquidity_gross_before == 0 {
        if tick <= tick_current {
            info.fee_growth_outside0_x128 = fee_growth_global0_x128;
            info.fee_growth_outside1_x128 = fee_growth_global1_x128;
            info.seconds_per_liquidity_outside_x128 = seconds_per_liquidity_cumulative_x128;
            info.tick_cumulative_outside = tick_cumulative;
            info.seconds_outside = time as u32;
        }
        info.initialized = true;
    }
    info.liquidity_gross = liquidity_gross_after.into();
    info.liquidity_net = if upper {
        info.liquidity_net - I128::from(liquidity_delta)
    } else {
        info.liquidity_net + I128::from(liquidity_delta)
    };
    save_tick(&tick, &info);
    flipped
}

pub fn clear(tick: i32) {
    save_tick(&tick, &TickInfo::default());
}

pub fn cross(
    tick: i32,
    fee_growth_global0_x128: U256,
    fee_growth_global1_x128: U256,
    seconds_per_liquidity_cumulative_x128: U256,
    tick_cumulative: i64,
    time: u64,
) -> i128 {
    let mut info = read_tick(&tick);
    info.fee_growth_outside0_x128 = fee_growth_global0_x128 - info.fee_growth_outside0_x128;
    info.fee_growth_outside1_x128 = fee_growth_global1_x128 - info.fee_growth_outside1_x128;
    info.seconds_per_liquidity_outside_x128 = seconds_per_liquidity_cumulative_x128;
    info.tick_cumulative_outside = tick_cumulative - info.tick_cumulative_outside;
    info.seconds_outside = time as u32 - info.seconds_outside;
    save_tick(&tick, &info);
    info.liquidity_net.0
}
