#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[cfg(test)]
mod test_miscellaneous_mint_tests {
    use std::ops::{Div, Sub};

    use casper_types::{Key, U128};

    use crate::{
        pool::fixture::{setup, setup_with_fee, TestContext, FEE_LOW},
        utils::{expand_to_18_decimals, other, wallet},
    };

    fn before_each() -> TestContext {
        let mut tc = setup_with_fee(FEE_LOW);
        tc.initialize_at_zero_tick();
        tc
    }

    #[test]
    fn test_mint_to_the_right_of_the_current_price() {
        let mut tc = before_each();
        let liquidity_delta = 1000_u128;
        let lower_tick = tc.tick_spacing;
        let upper_tick = tc.tick_spacing * 2;
        let liquidity_before = tc.get_liquidity();
        let b0 = tc.test_env.balance_of(tc.token0, tc.pool);
        let b1 = tc.test_env.balance_of(tc.token1, tc.pool);
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            liquidity_delta.into(),
        );

        let liquidity_after: casper_types::U128 = tc.get_liquidity();
        assert!(liquidity_after.ge(&liquidity_before));
        assert!(tc
            .test_env
            .balance_of(tc.token0, tc.pool)
            .sub(b0)
            .eq(&1.into()));
        assert!(tc
            .test_env
            .balance_of(tc.token1, tc.pool)
            .sub(b1)
            .eq(&0.into()));
    }

    #[test]
    fn test_mint_to_the_left_of_the_current_price() {
        let mut tc = before_each();
        let liquidity_delta = 1000_u128;
        let lower_tick = -tc.tick_spacing * 2;
        let upper_tick = -tc.tick_spacing;
        let liquidity_before = tc.get_liquidity();
        let b0 = tc.test_env.balance_of(tc.token0, tc.pool);
        let b1 = tc.test_env.balance_of(tc.token1, tc.pool);
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            liquidity_delta.into(),
        );

        let liquidity_after: casper_types::U128 = tc.get_liquidity();
        assert!(liquidity_after.ge(&liquidity_before));
        assert!(tc
            .test_env
            .balance_of(tc.token0, tc.pool)
            .sub(b0)
            .eq(&0.into()));
        assert!(tc
            .test_env
            .balance_of(tc.token1, tc.pool)
            .sub(b1)
            .eq(&1.into()));
    }

    #[test]
    fn test_mint_within_the_current_price() {
        let mut tc = before_each();
        let liquidity_delta = 1000_u128;
        let lower_tick = -tc.tick_spacing;
        let upper_tick = tc.tick_spacing;
        let liquidity_before = tc.get_liquidity();
        let b0 = tc.test_env.balance_of(tc.token0, tc.pool);
        let b1 = tc.test_env.balance_of(tc.token1, tc.pool);
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            liquidity_delta.into(),
        );

        let liquidity_after: casper_types::U128 = tc.get_liquidity();
        assert!(liquidity_after.ge(&liquidity_before));
        assert!(tc
            .test_env
            .balance_of(tc.token0, tc.pool)
            .sub(b0)
            .eq(&1.into()));
        assert!(tc
            .test_env
            .balance_of(tc.token1, tc.pool)
            .sub(b1)
            .eq(&1.into()));
    }

    #[test]
    fn test_cannot_remove_more_than_entire_position() {
        let mut tc = before_each();
        let lower_tick = -tc.tick_spacing;
        let upper_tick = tc.tick_spacing;
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            expand_to_18_decimals(1000).as_u128().into(),
        );

        tc.burn_fail(
            lower_tick,
            upper_tick,
            expand_to_18_decimals(1001).as_u128().into(),
        );
    }

    #[test]
    fn test_collect_fees_within_the_current_price_after_swap() {
        let mut tc = before_each();
        let liquidity_delta = expand_to_18_decimals(100).as_u128();
        let lower_tick = -tc.tick_spacing * 100;
        let upper_tick = tc.tick_spacing * 100;
        tc.mint(
            wallet().into(),
            lower_tick,
            upper_tick,
            liquidity_delta.into(),
        );

        let liquidity_before = tc.get_liquidity();
        let amount0_in = expand_to_18_decimals(1);
        tc.swap_exact_0_for_1(amount0_in, wallet().into(), None);

        let liquidity_after: casper_types::U128 = tc.get_liquidity();
        assert!(liquidity_after.ge(&liquidity_before));

        let token0_balance_before_pool = tc.test_env.balance_of(tc.token0, tc.pool);
        let token1_balance_before_pool = tc.test_env.balance_of(tc.token1, tc.pool);
        let token0_balance_before_wallet = tc.test_env.balance_of(tc.token0, wallet().into());
        let token1_balance_before_wallet = tc.test_env.balance_of(tc.token1, wallet().into());
        tc.burn(wallet(), lower_tick, upper_tick, 0.into());
        tc.collect(
            wallet().into(),
            lower_tick,
            upper_tick,
            U128::MAX,
            U128::MAX,
        );

        tc.burn(wallet(), lower_tick, upper_tick, 0.into());
        tc.collect(
            wallet().into(),
            lower_tick,
            upper_tick,
            U128::MAX,
            U128::MAX,
        );

        let token0_balance_after_pool = tc.test_env.balance_of(tc.token0, tc.pool);
        let token1_balance_after_pool = tc.test_env.balance_of(tc.token1, tc.pool);
        let token0_balance_after_wallet = tc.test_env.balance_of(tc.token0, wallet().into());
        let token1_balance_after_wallet = tc.test_env.balance_of(tc.token1, wallet().into());

        assert!(token0_balance_after_wallet.gt(&token0_balance_before_wallet));
        assert!(token1_balance_after_wallet.eq(&token1_balance_before_wallet));
        assert!(token0_balance_after_pool.lt(&token0_balance_before_pool));
        assert!(token1_balance_after_pool.eq(&token1_balance_before_pool));
    }
}
