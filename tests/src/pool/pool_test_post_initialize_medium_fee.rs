#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[cfg(test)]
mod test_post_initialize_medium_fee {
    use std::ops::Div;

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

    #[test]
    fn test_returns_0_before_initialization() {
        let mut tc = setup();
        assert!(tc.get_liquidity().as_u128() == 0);
    }

    #[test]
    fn test_returns_initial_liquidity() {
        let mut tc = before_each();
        assert!(tc.get_liquidity().as_u128() == expand_to_18_decimals(2).as_u128());
    }

    #[test]
    fn test_returns_in_supply_in_range() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            -tc.tick_spacing,
            tc.tick_spacing,
            expand_to_18_decimals(3).as_u128().into(),
        );
        assert!(tc.get_liquidity().as_u128() == expand_to_18_decimals(5).as_u128());
    }

    #[test]
    fn test_excludes_supply_at_tick_above_current_tick() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            tc.tick_spacing,
            tc.tick_spacing * 2,
            expand_to_18_decimals(3).as_u128().into(),
        );
        assert!(tc.get_liquidity().as_u128() == expand_to_18_decimals(2).as_u128());
    }

    #[test]
    fn test_excludes_supply_at_tick_below_current_tick() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            -tc.tick_spacing * 2,
            -tc.tick_spacing,
            expand_to_18_decimals(3).as_u128().into(),
        );
        assert!(tc.get_liquidity().as_u128() == expand_to_18_decimals(2).as_u128());
    }

    #[test]
    fn test_updates_correctly_when_exiting_range() {
        let mut tc = before_each();
        let k_before: casper_types::U128 = tc.get_liquidity();
        assert!(k_before.as_u128() == expand_to_18_decimals(2).as_u128());

        let liquidity_delta = expand_to_18_decimals(1).as_u128();
        let lower_tick = 0;
        let upper_tick = tc.tick_spacing;
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            liquidity_delta.into(),
        );

        let k_after: casper_types::U128 = tc.get_liquidity();
        assert!(k_after.eq(&expand_to_18_decimals(3).as_u128().into()));

        tc.swap_exact_0_for_1(1.into(), wallet().into(), None);
        let tick = tc.get_slot0().tick;
        assert!(tick == -1);

        let k_after_swap: casper_types::U128 = tc.get_liquidity();
        assert!(k_after_swap.eq(&expand_to_18_decimals(2).as_u128().into()));
    }

    #[test]
    fn test_updates_correctly_when_entering_range() {
        let mut tc = before_each();
        let k_before: casper_types::U128 = tc.get_liquidity();
        assert!(k_before.as_u128() == expand_to_18_decimals(2).as_u128());

        let liquidity_delta = expand_to_18_decimals(1).as_u128();
        let lower_tick = -tc.tick_spacing;
        let upper_tick = 0;
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            liquidity_delta.into(),
        );

        let k_after: casper_types::U128 = tc.get_liquidity();
        assert!(k_after.eq(&k_before));

        tc.swap_exact_0_for_1(1.into(), wallet().into(), None);
        let tick = tc.get_slot0().tick;
        assert!(tick == -1);

        let k_after_swap: casper_types::U128 = tc.get_liquidity();
        assert!(k_after_swap.eq(&expand_to_18_decimals(3).as_u128().into()));
    }
}

#[cfg(test)]
mod test_limit_orders {
    use std::ops::Div;

    use casper_types::{Key, U128};

    use crate::{
        pool::fixture::{setup, TestContext},
        utils::{expand_to_18_decimals, other, wallet},
    };

    fn before_each() -> TestContext {
        let mut tc = setup();
        tc.initialize_at_zero_tick();
        tc
    }

    #[test]
    fn test_limit_selling_0_for_1_at_tick_0_thru_1() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            0,
            120,
            expand_to_18_decimals(1).as_u128().into(),
        );
        tc.swap_exact_1_for_0(expand_to_18_decimals(2), other().into(), None);
        tc.burn(wallet(), 0, 120, expand_to_18_decimals(1).as_u128().into());
        tc.collect(wallet().into(), 0, 120, U128::MAX, U128::MAX);
        assert!(tc.get_slot0().tick >= 120);
    }

    #[test]
    fn test_limit_selling_1_for_0_at_tick_0_thru_minus_1() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            -120,
            0,
            expand_to_18_decimals(1).as_u128().into(),
        );
        tc.swap_exact_0_for_1(expand_to_18_decimals(2), other().into(), None);
        tc.burn(wallet(), -120, 0, expand_to_18_decimals(1).as_u128().into());
        tc.collect(wallet().into(), -120, 0, U128::MAX, U128::MAX);
        assert!(tc.get_slot0().tick < 120);
    }

    #[cfg(test)]
    mod fee_is_on {
        use casper_types::U128;

        use crate::{
            pool::pool_test_post_initialize_medium_fee::test_limit_orders::before_each,
            utils::{expand_to_18_decimals, other, wallet},
        };

        #[test]
        fn test_limit_selling_0_for_1_at_tick_0_thru_1() {
            let mut tc = before_each();
            tc.set_fee_protocol(6, 6);
            tc.mint(
                wallet().into(),
                0,
                120,
                expand_to_18_decimals(1).as_u128().into(),
            );
            tc.swap_exact_1_for_0(expand_to_18_decimals(2), other().into(), None);
            tc.burn(wallet(), 0, 120, expand_to_18_decimals(1).as_u128().into());
            tc.collect(wallet().into(), 0, 120, U128::MAX, U128::MAX);
            assert!(tc.get_slot0().tick >= 120);
        }

        #[test]
        fn test_limit_selling_1_for_0_at_tick_0_thru_minus_1() {
            let mut tc = before_each();
            tc.mint(
                wallet().into(),
                -120,
                0,
                expand_to_18_decimals(1).as_u128().into(),
            );
            tc.swap_exact_0_for_1(expand_to_18_decimals(2), other().into(), None);
            tc.burn(wallet(), -120, 0, expand_to_18_decimals(1).as_u128().into());
            tc.collect(wallet().into(), -120, 0, U128::MAX, U128::MAX);
            assert!(tc.get_slot0().tick < 120);
        }
    }
}
