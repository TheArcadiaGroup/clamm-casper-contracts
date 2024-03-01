#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[cfg(test)]
mod test_burn {
    use casper_types::Key;

    use crate::{
        pool::fixture::{setup, TestContext},
        utils::{expand_to_18_decimals, other, wallet},
    };

    fn before_each() -> TestContext {
        let mut tc = setup();
        tc.initialize_at_zero_tick();
        tc
    }

    fn check_tick_is_clear(tc: &mut TestContext, tick: i32) {
        let tick_info = tc.get_tick(tick);
        assert!(tick_info.liquidity_gross.as_u128() == 0);
        assert!(tick_info.fee_growth_outside0_x128.as_u128() == 0);
        assert!(tick_info.fee_growth_outside1_x128.as_u128() == 0);
        assert!(tick_info.liquidity_net.0 == 0);
    }

    fn check_tick_is_not_clear(tc: &mut TestContext, tick: i32) {
        let tick_info = tc.get_tick(tick);
        assert!(tick_info.liquidity_gross.as_u128() != 0);
    }

    #[test]
    fn test_does_not_clear_the_position_fee_growth_snapshot_if_no_more_liquidity() {
        let mut tc = before_each();
        tc.test_env.advance_block_time_by(10);
        tc.mint(
            Key::from(other()),
            tc.min_tick,
            tc.max_tick,
            expand_to_18_decimals(1).as_u128().into(),
        );
        tc.swap_exact_0_for_1(expand_to_18_decimals(1), wallet().into(), None);
        tc.swap_exact_1_for_0(expand_to_18_decimals(1), wallet().into(), None);

        tc.burn(
            other(),
            tc.min_tick,
            tc.max_tick,
            expand_to_18_decimals(1).as_u128().into(),
        );
        let position_info = tc.get_position(other().into(), tc.min_tick, tc.max_tick);
        assert!(position_info.liquidity.as_u128() == 0);
        assert!(position_info.tokens_owed0.as_u128() != 0);
        assert!(position_info.tokens_owed1.as_u128() != 0);
        assert!(
            position_info.fee_growth_inside0_last_x128.as_u128()
                == 340282366920938463463374607431768211
        );
        assert!(
            position_info.fee_growth_inside1_last_x128.as_u128()
                == 340282366920938576890830247744589365
        );
    }

    #[test]
    fn test_clears_the_tick_if_its_the_last_position_using_it() {
        let mut tc = before_each();
        let (tick_lower, tick_upper) =
            (tc.min_tick + tc.tick_spacing, tc.max_tick - tc.tick_spacing);
        tc.test_env.advance_block_time_by(10);

        tc.mint(Key::from(wallet()), tick_lower, tick_upper, 1.into());
        tc.swap_exact_0_for_1(expand_to_18_decimals(1), wallet().into(), None);

        tc.burn(wallet(), tick_lower, tick_upper, 1.into());
        check_tick_is_clear(&mut tc, tick_lower);
        check_tick_is_clear(&mut tc, tick_upper);
    }

    #[test]
    fn test_clears_only_the_lower_tick_if_upper_is_still_used() {
        let mut tc = before_each();
        let (tick_lower, tick_upper) =
            (tc.min_tick + tc.tick_spacing, tc.max_tick - tc.tick_spacing);
        tc.test_env.advance_block_time_by(10);

        tc.mint(Key::from(wallet()), tick_lower, tick_upper, 1.into());
        tc.mint(
            Key::from(wallet()),
            tick_lower + tc.tick_spacing,
            tick_upper,
            1.into(),
        );
        tc.swap_exact_0_for_1(expand_to_18_decimals(1), wallet().into(), None);

        tc.burn(wallet(), tick_lower, tick_upper, 1.into());
        check_tick_is_clear(&mut tc, tick_lower);
        check_tick_is_not_clear(&mut tc, tick_upper);
    }

    #[test]
    fn test_clears_only_the_uppeer_tick_if_lower_is_still_used() {
        let mut tc = before_each();
        let (tick_lower, tick_upper) =
            (tc.min_tick + tc.tick_spacing, tc.max_tick - tc.tick_spacing);
        tc.test_env.advance_block_time_by(10);

        tc.mint(Key::from(wallet()), tick_lower, tick_upper, 1.into());
        tc.mint(
            Key::from(wallet()),
            tick_lower,
            tick_upper - tc.tick_spacing,
            1.into(),
        );
        tc.swap_exact_0_for_1(expand_to_18_decimals(1), wallet().into(), None);

        tc.burn(wallet(), tick_lower, tick_upper, 1.into());
        check_tick_is_not_clear(&mut tc, tick_lower);
        check_tick_is_clear(&mut tc, tick_upper);
    }
}
