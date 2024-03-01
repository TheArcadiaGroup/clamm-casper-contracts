#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[cfg(test)]
mod test_snapshot_cumulatives_inside {
    use std::ops::{Div, Shl, Sub};

    use casper_types::{Key, U256};
    use common::pool_events::SnapshotCumulativesInside;

    use crate::{
        pool::fixture::{
            get_tick_spacing, setup, setup_with_fee, TestContext, FEE_LOW, FEE_MEDIUM,
        },
        utils::{
            encode_price_sqrt, expand_to_18_decimals, get_max_tick, get_min_tick, other, wallet,
        },
    };
    fn tick_lower() -> i32 {
        -get_tick_spacing(FEE_MEDIUM)
    }
    fn tick_upper() -> i32 {
        get_tick_spacing(FEE_MEDIUM)
    }
    fn before_each() -> TestContext {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(
            wallet().into(),
            -tc.tick_spacing,
            tc.tick_spacing,
            10.into(),
        );
        tc
    }

    #[test]
    fn test_throws_if_ticks_are_in_reverse_order() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside_fail(tc.tick_spacing, -tc.tick_spacing);
    }

    #[test]
    fn test_throws_if_ticks_are_the_same() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside_fail(tc.tick_spacing, tc.tick_spacing);
    }

    #[test]
    fn test_throws_if_tick_lower_is_too_low() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside_fail(tc.min_tick - 1, tc.tick_spacing);
    }

    #[test]
    fn test_throws_if_tick_upper_is_too_high() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside_fail(-tc.tick_spacing, tc.max_tick + 1);
    }

    #[test]
    fn test_throws_if_tick_lower_is_not_initialized() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside_fail(-tc.tick_spacing - tc.tick_spacing, tc.tick_spacing);
    }

    #[test]
    fn test_throws_if_tick_upper_is_not_initialized() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside_fail(-tc.tick_spacing, tc.tick_spacing + tc.tick_spacing);
    }

    #[test]
    fn test_is_zero_immediately_after_initialize() {
        let mut tc = before_each();
        tc.snapshot_cumulatives_inside(-tc.tick_spacing, tc.tick_spacing);
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data.seconds_per_liquidity_insideX128.eq(&0.into()));
        assert!(snapshot_data.tick_cumulative_inside == 0);
        assert!(snapshot_data.seconds_inside == 0);
    }

    #[test]
    fn test_increases_by_expected_amount_when_time_elapses_in_the_range() {
        let mut tc = before_each();
        tc.test_env.advance_block_time_by(5);
        tc.snapshot_cumulatives_inside(-tc.tick_spacing, tc.tick_spacing);
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(5).shl(128).div(10)));
        assert!(snapshot_data.tick_cumulative_inside == 0);
        assert!(snapshot_data.seconds_inside == 5);
    }

    #[test]
    fn test_does_not_account_for_time_increase_above_range() {
        let mut tc = before_each();
        tc.test_env.advance_block_time_by(5);
        tc.swap_to_higher_price(encode_price_sqrt(2, 1), wallet().into());
        tc.test_env.advance_block_time_by(7);
        tc.snapshot_cumulatives_inside(-tc.tick_spacing, tc.tick_spacing);
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(5).shl(128).div(10)));
        assert!(snapshot_data.tick_cumulative_inside == 0);
        assert!(snapshot_data.seconds_inside == 5);
    }

    #[test]
    fn test_does_not_account_for_time_increase_below_range() {
        let mut tc = before_each();
        tc.test_env.advance_block_time_by(5);
        tc.swap_to_lower_price(encode_price_sqrt(1, 2), wallet().into());
        tc.test_env.advance_block_time_by(7);
        tc.snapshot_cumulatives_inside(-tc.tick_spacing, tc.tick_spacing);
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(5).shl(128).div(10)));
        assert!(snapshot_data.tick_cumulative_inside == 0);
        assert!(snapshot_data.seconds_inside == 5);
    }

    #[test]
    fn test_time_increase_below_range_is_not_counted() {
        let mut tc = before_each();
        tc.swap_to_lower_price(encode_price_sqrt(1, 2), wallet().into());
        tc.test_env.advance_block_time_by(5);
        tc.swap_to_higher_price(encode_price_sqrt(1, 1), wallet().into());
        tc.test_env.advance_block_time_by(7);
        tc.snapshot_cumulatives_inside(-tc.tick_spacing, tc.tick_spacing);
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(7).shl(128).div(10)));
        assert!(snapshot_data.tick_cumulative_inside == 0);
        assert!(snapshot_data.seconds_inside == 7);
    }

    #[test]
    fn test_time_increase_above_range_is_not_counted() {
        let mut tc = before_each();
        tc.swap_to_higher_price(encode_price_sqrt(2, 1), wallet().into());
        tc.test_env.advance_block_time_by(5);
        tc.swap_to_lower_price(encode_price_sqrt(1, 1), wallet().into());
        tc.test_env.advance_block_time_by(7);
        tc.snapshot_cumulatives_inside(tick_lower(), tick_upper());
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(7).shl(128).div(10)));
        assert!(snapshot_data.tick_cumulative_inside == -7);
        assert!(tc.get_slot0().tick == -1);
        assert!(snapshot_data.seconds_inside == 7);
    }

    #[test]
    fn test_positions_minted_after_time_spent() {
        let mut tc = before_each();
        tc.test_env.advance_block_time_by(5);
        tc.mint(
            wallet().into(),
            tick_upper(),
            get_max_tick(tc.tick_spacing),
            15.into(),
        );
        tc.swap_to_higher_price(encode_price_sqrt(2, 1), wallet().into());
        tc.test_env.advance_block_time_by(8);
        tc.snapshot_cumulatives_inside(tick_upper(), get_max_tick(tc.tick_spacing));
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(8).shl(128).div(15)));
        assert!(snapshot_data.tick_cumulative_inside == 55448);
        assert!(snapshot_data.seconds_inside == 8);
    }

    #[test]
    fn test_overlapping_liquidity_is_aggregated() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            tick_lower(),
            get_max_tick(tc.tick_spacing),
            15.into(),
        );
        tc.test_env.advance_block_time_by(5);
        tc.swap_to_higher_price(encode_price_sqrt(2, 1), wallet().into());
        tc.test_env.advance_block_time_by(8);
        tc.snapshot_cumulatives_inside(tick_lower(), tick_upper());
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .eq(&U256::from(5).shl(128).div(25)));
        assert!(snapshot_data.tick_cumulative_inside == 0);
        assert!(snapshot_data.seconds_inside == 5);
    }

    #[test]
    fn test_relative_behavior_of_snapshots() {
        let mut tc = before_each();
        tc.test_env.advance_block_time_by(5);
        tc.mint(
            wallet().into(),
            get_min_tick(tc.tick_spacing),
            tick_lower(),
            15.into(),
        );
        tc.snapshot_cumulatives_inside(get_min_tick(tc.tick_spacing), tick_lower());
        let snapshot_data_start: SnapshotCumulativesInside =
            tc.test_env.get_last_event(tc.pool).unwrap();
        tc.test_env.advance_block_time_by(8);
        tc.swap_to_lower_price(encode_price_sqrt(1, 2), wallet().into());
        tc.test_env.advance_block_time_by(3);
        tc.snapshot_cumulatives_inside(get_min_tick(tc.tick_spacing), tick_lower());
        let snapshot_data: SnapshotCumulativesInside = tc.test_env.get_last_event(tc.pool).unwrap();

        let expected_diff_seconds_per_liquidity = U256::from(3).shl(128).div(15);
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .sub(snapshot_data_start.seconds_per_liquidity_insideX128)
            .eq(&expected_diff_seconds_per_liquidity));
        assert!(snapshot_data
            .seconds_per_liquidity_insideX128
            .ne(&expected_diff_seconds_per_liquidity));
        assert!(
            snapshot_data.tick_cumulative_inside - snapshot_data_start.tick_cumulative_inside
                == -20796
        );
        assert!(snapshot_data.seconds_inside - snapshot_data_start.seconds_inside == 3);
        assert!(snapshot_data.seconds_inside != 3);
    }
}

#[cfg(test)]
mod test_fees_overflow_scenarios {
    use std::ops::Shl;

    use casper_types::{U128, U256};
    use common::pool_events::Collect;

    use crate::{
        pool::fixture::setup,
        utils::{encode_price_sqrt, other, wallet},
    };

    fn max_u128() -> U256 {
        U128::MAX.as_u128().into()
    }

    #[test]
    fn test_up_to_max_uint_128() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(wallet().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(max_u128()),
        );
        let (fee_growth_global0_x128, fee_growth_global1_x128) = (
            tc.get_fee_growth_global0_x128(),
            tc.get_fee_growth_global1_x128(),
        );
        assert!(fee_growth_global0_x128.eq(&max_u128().shl(128)));
        assert!(fee_growth_global1_x128.eq(&max_u128().shl(128)));

        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&max_u128()));
        assert!(collect_event.amount1.eq(&max_u128()));
    }

    #[test]
    fn test_overflow_max_uint_128() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(wallet().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(max_u128()),
        );
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(U256::one()),
            Some(U256::one()),
        );
        let (fee_growth_global0_x128, fee_growth_global1_x128) = (
            tc.get_fee_growth_global0_x128(),
            tc.get_fee_growth_global1_x128(),
        );
        assert!(fee_growth_global0_x128.eq(&0.into()));
        assert!(fee_growth_global1_x128.eq(&0.into()));

        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&0.into()));
        assert!(collect_event.amount1.eq(&0.into()));
    }

    #[test]
    fn test_overflow_max_uint_128_after_poke_burns_fees_owed_to_0() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(wallet().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(max_u128()),
        );

        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(U256::one()),
            Some(U256::one()),
        );
        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());

        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&0.into()));
        assert!(collect_event.amount1.eq(&0.into()));
    }

    #[test]
    fn test_two_positions_at_the_same_snapshot() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(wallet().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.mint(other().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(0.into()),
        );
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(0.into()),
        );
        let fee_growth_global0_x128 = tc.get_fee_growth_global0_x128();
        assert!(fee_growth_global0_x128.eq(&max_u128().shl(128)));

        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(2.into()),
            Some(0.into()),
        );
        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.burn(other(), tc.min_tick, tc.max_tick, 0.into());

        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&0.into()));

        tc.collect_with(
            other(),
            other().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );
        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&0.into()));
    }

    #[test]
    fn test_two_positions_1_wei_of_fees_apart_overflows_exactly_once() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(wallet().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(1.into()),
            Some(0.into()),
        );

        tc.mint(other().into(), tc.min_tick, tc.max_tick, 1.into());
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(0.into()),
        );
        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(max_u128()),
            Some(0.into()),
        );

        let fee_growth_global0_x128 = tc.get_fee_growth_global0_x128();
        assert!(fee_growth_global0_x128.eq(&0.into()));

        tc.flash(
            0.into(),
            0.into(),
            wallet().into(),
            Some(2.into()),
            Some(0.into()),
        );

        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.burn(other(), tc.min_tick, tc.max_tick, 0.into());

        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&1.into()));

        tc.collect_with(
            other(),
            other().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );
        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.eq(&0.into()));
    }
}
