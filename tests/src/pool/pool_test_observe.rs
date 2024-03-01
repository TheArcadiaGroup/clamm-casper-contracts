#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[cfg(test)]
mod test_observe {
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
    fn test_current_tick_accumulator_increases_by_tick_over_time() {
        let mut tc = before_each();
        let (tick_cumulatives, _) = tc.observe(vec![0]);
        let tick_cumulative = tick_cumulatives[0];
        assert!(tick_cumulative == 0);
        tc.test_env.advance_block_time_by(10);
        let (tick_cumulatives, _) = tc.observe(vec![0]);
        let tick_cumulative = tick_cumulatives[0];
        assert!(tick_cumulative == 0);
    }

    #[test]
    fn test_current_tick_accumulator_after_single_swap() {
        let mut tc = before_each();
        tc.swap_exact_0_for_1(100.into(), wallet().into(), None);
        tc.test_env.advance_block_time_by(4);

        let (tick_cumulatives, _) = tc.observe(vec![0]);
        let tick_cumulative = tick_cumulatives[0];
        assert!(tick_cumulative == -4);
    }

    #[test]
    fn test_current_tick_accumulator_after_two_swaps() {
        let mut tc = before_each();
        println!(
            "sqrt_price_x96 before swap {:?}",
            tc.get_slot0().sqrt_price_x96
        );
        tc.swap_exact_0_for_1(expand_to_18_decimals(1).div(2), wallet().into(), None);
        println!(
            "sqrt_price_x96 before after {:?}",
            tc.get_slot0().sqrt_price_x96
        );
        println!("tc.get_slot0().tick 2 {:?}", tc.get_slot0().tick);
        assert!(tc.get_slot0().tick == -4452);
        tc.test_env.advance_block_time_by(4);
        tc.swap_exact_1_for_0(expand_to_18_decimals(1).div(4), wallet().into(), None);
        assert!(tc.get_slot0().tick == -1558);
        tc.test_env.advance_block_time_by(6);

        let (tick_cumulatives, _) = tc.observe(vec![0]);
        let tick_cumulative = tick_cumulatives[0];
        assert!(tick_cumulative == -27156);
    }
}
