#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[cfg(test)]
mod test_collect {
    use std::ops::Div;

    use casper_types::Key;

    use crate::{
        pool::fixture::{setup, setup_with_fee, TestContext, FEE_LOW},
        utils::{encode_price_sqrt, expand_to_18_decimals, other, wallet},
    };

    fn before_each() -> TestContext {
        let mut tc = setup_with_fee(FEE_LOW);
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc
    }

    #[test]
    fn test_works_with_multiple_lps() {
        let mut tc = before_each();
        tc.mint(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            expand_to_18_decimals(1).as_u128().into(),
        );
        tc.mint(
            wallet().into(),
            tc.min_tick + tc.tick_spacing,
            tc.max_tick - tc.tick_spacing,
            expand_to_18_decimals(2).as_u128().into(),
        );
        tc.swap_exact_0_for_1(expand_to_18_decimals(1), wallet().into(), None);

        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.burn(
            wallet(),
            tc.min_tick + tc.tick_spacing,
            tc.max_tick - tc.tick_spacing,
            0.into(),
        );

        let tokens_owed0_position0 = tc
            .get_position(wallet().into(), tc.min_tick, tc.max_tick)
            .tokens_owed0;
        let tokens_owed0_position1 = tc
            .get_position(
                wallet().into(),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
            )
            .tokens_owed0;
        println!("tokens_owed0_position0 {:?}", tokens_owed0_position0);
        assert!(tokens_owed0_position0.as_u128() == 166666666666667);
        assert!(tokens_owed0_position1.as_u128() == 333333333333334);
    }

    #[cfg(test)]
    mod works_across_large_increases {
        use casper_types::U256;

        use crate::{
            pool::fixture::TestContext,
            utils::{expand_to_18_decimals, wallet},
        };

        use super::before_each;

        fn magic_number() -> U256 {
            U256::from_dec_str("115792089237316195423570985008687907852929702298719625575994")
                .unwrap()
        }

        fn before_each_test() -> TestContext {
            let mut tc = before_each();
            tc.mint(
                wallet().into(),
                tc.min_tick,
                tc.max_tick,
                expand_to_18_decimals(1).as_u128().into(),
            );
            tc
        }

        // #[test]
        // fn test_works_just_before_the_cap_binds() {
        //     let mut tc = before_each_test();
        // }
        // TODO: add tests for works across large increases
    }
}
