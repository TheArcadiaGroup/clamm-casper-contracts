use crate::store::{read_position, save_position};
use alloc::string::String;
use casper_types::{Key, U256};
use common::{error::require, utils};
use math::{fixed_point_128, fullmath, liquidity_math};
use types::PositionInfo;

pub fn get(owner: Key, tick_lower: i32, tick_upper: i32) -> PositionInfo {
    read_position(&position_key(owner, tick_lower, tick_upper))
}

pub fn position_key(owner: Key, tick_lower: i32, tick_upper: i32) -> String {
    utils::position_key(owner, tick_lower, tick_upper)
}

pub fn update(
    position_key: String,
    _self: &mut PositionInfo,
    liquidity_delta: i128,
    fee_growth_inside0_x128: &U256,
    fee_growth_inside1_x128: &U256,
) {
    let liquidity_next = if liquidity_delta == 0 {
        require(_self.liquidity.as_u128() > 0, common::error::Error::ErrNP);
        _self.liquidity.as_u128()
    } else {
        liquidity_math::add_delta(_self.liquidity.as_u128(), liquidity_delta)
    };
    let tokens_owed0 = fullmath::mul_div(
        &fee_growth_inside0_x128
            .overflowing_sub(_self.fee_growth_inside0_last_x128)
            .0,
        &_self.liquidity.as_u128().into(),
        &fixed_point_128::q128(),
    )
    .as_u128();
    let tokens_owed1 = fullmath::mul_div(
        &fee_growth_inside1_x128
            .overflowing_sub(_self.fee_growth_inside1_last_x128)
            .0,
        &_self.liquidity.as_u128().into(),
        &fixed_point_128::q128(),
    )
    .as_u128();

    if liquidity_delta != 0 {
        _self.liquidity = liquidity_next.into();
    }
    _self.fee_growth_inside0_last_x128 = *fee_growth_inside0_x128;
    _self.fee_growth_inside1_last_x128 = *fee_growth_inside1_x128;
    if tokens_owed0 > 0 || tokens_owed1 > 0 {
        _self.tokens_owed0 = _self.tokens_owed0.overflowing_add(tokens_owed0.into()).0;
        _self.tokens_owed1 = _self.tokens_owed1.overflowing_add(tokens_owed1.into()).0;
    }
    save_position(&position_key, _self);
}
