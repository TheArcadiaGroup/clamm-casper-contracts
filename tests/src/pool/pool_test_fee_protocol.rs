#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use super::fixture::{setup_with_fee, FEE_MEDIUM};

#[cfg(test)]
mod test_fee_protocol {
    use std::ops::Div;

    use casper_types::{runtime_args, Key, RuntimeArgs, U128, U256};
    use common::pool_events::Collect;

    use crate::{
        pool::fixture::{setup, setup_with_fee, TestContext, FEE_LOW},
        utils::{encode_price_sqrt, expand_to_18_decimals, other, wallet},
    };

    fn liquidity_amount() -> U128 {
        expand_to_18_decimals(1000).as_u128().into()
    }
    fn before_each() -> TestContext {
        let mut tc = setup_with_fee(FEE_LOW);
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.mint(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            liquidity_amount(),
        );
        tc
    }

    #[test]
    fn test_is_initially_set_to_0() {
        let mut tc = before_each();
        assert!(tc.get_slot0().fee_protocol == 0);
    }

    #[test]
    fn test_can_be_changed_by_the_owner() {
        let mut tc = before_each();
        tc.set_fee_protocol(6, 6);
        assert!(tc.get_slot0().fee_protocol == 102);
    }

    #[test]
    fn test_cannot_be_changed_out_of_bounds() {
        let mut tc = before_each();
        tc.set_fee_protocol_fail(3, 3);
        tc.set_fee_protocol_fail(11, 11);
    }

    #[test]
    fn test_cannot_be_changed_by_addresses_that_are_not_owner() {
        let mut tc = before_each();
        tc.test_env.call_contract(
            Some(other()),
            tc.pool.into_hash().unwrap().into(),
            "set_fee_protocol",
            runtime_args! {
                "fee_protocol0" => 6_u8,
                "fee_protocol1" => 6_u8,
            },
            false,
        );
    }

    fn swap_and_get_fees_owed(
        tc: &mut TestContext,
        amount: U256,
        zero_for_one: bool,
        poke: bool,
    ) -> (U128, U128) {
        if zero_for_one {
            tc.swap_exact_0_for_1(amount, wallet().into(), None);
        } else {
            tc.swap_exact_1_for_0(amount, wallet().into(), None);
        }
        if poke {
            tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        }

        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let collect_event: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(collect_event.amount0.ge(&0.into()));
        assert!(collect_event.amount1.ge(&0.into()));
        (
            collect_event.amount0.as_u128().into(),
            collect_event.amount1.as_u128().into(),
        )
    }

    #[test]
    fn test_position_owner_gets_full_fees_when_protocol_fee_is_off() {
        let mut tc = before_each();
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 499999999999999);
        assert!(token1_fees.as_u128() == 0);
    }

    #[test]
    fn test_swap_fees_accumulate_as_expected_0_for_1() {
        let mut tc = before_each();
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 499999999999999);
        assert!(token1_fees.as_u128() == 0);

        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 999999999999998);
        assert!(token1_fees.as_u128() == 0);

        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 1499999999999997);
        assert!(token1_fees.as_u128() == 0);
    }

    #[test]
    fn test_swap_fees_accumulate_as_expected_1_for_0() {
        let mut tc = before_each();
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), false, true);
        assert!(token1_fees.as_u128() == 499999999999999);
        assert!(token0_fees.as_u128() == 0);

        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), false, true);
        assert!(token1_fees.as_u128() == 999999999999998);
        assert!(token0_fees.as_u128() == 0);

        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), false, true);
        assert!(token1_fees.as_u128() == 1499999999999997);
        assert!(token0_fees.as_u128() == 0);
    }

    #[test]
    fn test_position_owner_gets_partial_fees_when_protocol_fee_is_on() {
        let mut tc = before_each();
        tc.set_fee_protocol(6, 6);
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 416666666666666);
        assert!(token1_fees.as_u128() == 0);
    }

    #[cfg(test)]
    mod collect_protocol {
        use casper_types::U128;
        use common::pool_events::CollectProtocol;

        use crate::{
            pool::pool_test_fee_protocol::test_fee_protocol::{
                before_each, swap_and_get_fees_owed,
            },
            utils::{expand_to_18_decimals, other, wallet},
        };

        #[test]
        fn test_returns_0_if_no_fees() {
            let mut tc = before_each();
            tc.set_fee_protocol(6, 6);
            tc.collect_protocol(wallet().into(), U128::MAX, U128::MAX);
            let collect_protocol: CollectProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
            assert!(collect_protocol.amount0.as_u128() == 0);
            assert!(collect_protocol.amount1.as_u128() == 0);
        }

        #[test]
        fn test_can_collect_fees() {
            let mut tc = before_each();
            tc.set_fee_protocol(6, 6);
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
            tc.collect_protocol(wallet().into(), U128::MAX, U128::MAX);
            let collect_protocol: CollectProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
            assert!(collect_protocol.amount0.as_u128() == 83333333333332);
        }

        #[test]
        fn test_fees_collected_can_differ_between_token0_and_token1() {
            let mut tc = before_each();
            tc.set_fee_protocol(8, 5);
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, false);
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), false, false);
            tc.collect_protocol(other().into(), U128::MAX, U128::MAX);
            let collect_protocol: CollectProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
            assert!(collect_protocol.amount0.as_u128() == 62499999999999);
            assert!(collect_protocol.amount1.as_u128() == 99999999999998);
        }
    }

    #[test]
    fn test_fees_collected_by_lp_after_two_swaps_should_be_double_one_swap() {
        let mut tc = before_each();
        swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 999999999999998);
        assert!(token1_fees.as_u128() == 0);
    }

    #[test]
    fn test_fees_collected_after_two_swaps_with_fee_turned_on_in_middle_are_fees_from_last_swap_not_confiscatory(
    ) {
        let mut tc = before_each();
        swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, false);
        tc.set_fee_protocol(6, 6);
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 916666666666666);
        assert!(token1_fees.as_u128() == 0);
    }

    #[test]
    fn test_fees_collected_by_lp_after_two_swaps_with_intermediate_withdrawal() {
        let mut tc = before_each();
        tc.set_fee_protocol(6, 6);
        let (token0_fees, token1_fees) =
            swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, true);
        assert!(token0_fees.as_u128() == 416666666666666);
        assert!(token1_fees.as_u128() == 0);

        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        swap_and_get_fees_owed(&mut tc, expand_to_18_decimals(1), true, false);
        // assert!(token0_fees_next.as_u128() == 0);
        // assert!(token1_fees_next.as_u128() == 0);

        let protocol_fees: types::ProtocolFees = tc.get_protocol_fees();
        assert!(protocol_fees.token0.as_u128() == 166666666666666);
        assert!(protocol_fees.token1.as_u128() == 0);

        tc.burn(wallet(), tc.min_tick, tc.max_tick, 0.into());
        tc.collect(
            wallet().into(),
            tc.min_tick,
            tc.max_tick,
            U128::MAX,
            U128::MAX,
        );

        let protocol_fees: types::ProtocolFees = tc.get_protocol_fees();
        assert!(protocol_fees.token0.as_u128() == 166666666666666);
        assert!(protocol_fees.token1.as_u128() == 0);
    }
}

#[cfg(test)]
mod test_increase_observation_cardinality_next {
    use crate::{pool::fixture::setup, utils::wallet};

    #[test]
    fn test_cannot_be_called_before_initialization() {
        let mut tc = setup();
        tc.increase_observation_cardinality_next_fail(Some(wallet()), 2);
    }

    #[cfg(test)]
    mod after_initialization {
        use crate::{
            pool::fixture::{setup, TestContext, TEST_POOL_START_TIME},
            utils::{encode_price_sqrt, wallet},
        };

        fn before_each() -> TestContext {
            let mut tc = setup();
            tc.initialize_pool_price(encode_price_sqrt(1, 1));
            tc
        }

        #[test]
        fn test_oracle_starting_state_after_initialization() {
            let mut tc = before_each();
            let slot0 = tc.get_slot0();
            assert!(slot0.observation_cardinality == 1);
            assert!(slot0.observation_index == 0);
            assert!(slot0.observation_cardinality_next == 1);

            let observation = tc.get_observation(0);
            assert!(observation
                .seconds_per_liquidity_cumulative_x128
                .eq(&0.into()));
            assert!(observation.tick_cumulative == 0);
            assert!(observation.initialized);
            assert!(observation.block_timestamp == TEST_POOL_START_TIME);
        }

        #[test]
        fn test_increases_observation_cardinality_next() {
            let mut tc = before_each();
            tc.increase_observation_cardinality_next(Some(wallet()), 2);
            let slot0 = tc.get_slot0();
            assert!(slot0.observation_cardinality == 1);
            assert!(slot0.observation_index == 0);
            assert!(slot0.observation_cardinality_next == 2);
        }

        #[test]
        fn test_is_no_op_if_target_is_already_exceeded() {
            let mut tc = before_each();
            tc.increase_observation_cardinality_next(Some(wallet()), 5);
            tc.increase_observation_cardinality_next(Some(wallet()), 3);
            let slot0 = tc.get_slot0();
            assert!(slot0.observation_cardinality == 1);
            assert!(slot0.observation_index == 0);
            assert!(slot0.observation_cardinality_next == 5);
        }
    }
}

#[cfg(test)]
mod test_set_fee_protocol {
    use common::pool_events::SetFeeProtocol;

    use crate::{
        pool::fixture::{setup, TestContext},
        utils::{encode_price_sqrt, other},
    };

    fn before_each() -> TestContext {
        let mut tc: TestContext = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc
    }

    #[test]
    fn test_fails_if_fee_is_lt_4_or_gt_10() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol_fail(3, 3);
        tc.set_fee_protocol_fail(6, 3);
        tc.set_fee_protocol_fail(3, 6);
        tc.set_fee_protocol_fail(11, 11);
        tc.set_fee_protocol_fail(6, 11);
        tc.set_fee_protocol_fail(11, 6);
    }

    #[test]
    fn test_succeed_for_fee_of_4() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(4, 4);
    }

    #[test]
    fn test_succeed_for_fee_of_10() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(10, 10);
    }

    #[test]
    fn test_can_change_protocol_fee() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(7, 7);
        tc.set_fee_protocol(5, 8);
        assert!(tc.get_slot0().fee_protocol == 133);
    }

    #[test]
    fn test_turn_off_protocol_fee() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(4, 4);
        tc.set_fee_protocol(0, 0);
        assert!(tc.get_slot0().fee_protocol == 0);
    }

    #[test]
    fn test_emits_an_event_when_turned_on() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(7, 7);
        let set_fee_protocol_event: SetFeeProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(set_fee_protocol_event.fee_protocol0_old == 0);
        assert!(set_fee_protocol_event.fee_protocol1_old == 0);
        assert!(set_fee_protocol_event.fee_protocol0_new == 7);
        assert!(set_fee_protocol_event.fee_protocol1_new == 7);
    }

    #[test]
    fn test_emits_an_event_when_turned_off() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(7, 5);
        tc.set_fee_protocol(0, 0);
        let set_fee_protocol_event: SetFeeProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(set_fee_protocol_event.fee_protocol0_old == 7);
        assert!(set_fee_protocol_event.fee_protocol1_old == 5);
        assert!(set_fee_protocol_event.fee_protocol0_new == 0);
        assert!(set_fee_protocol_event.fee_protocol1_new == 0);
    }

    #[test]
    fn test_emits_an_event_when_changed() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(4, 10);
        tc.set_fee_protocol(6, 8);
        let set_fee_protocol_event: SetFeeProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(set_fee_protocol_event.fee_protocol0_old == 4);
        assert!(set_fee_protocol_event.fee_protocol1_old == 10);
        assert!(set_fee_protocol_event.fee_protocol0_new == 6);
        assert!(set_fee_protocol_event.fee_protocol1_new == 8);
    }

    #[test]
    fn test_emits_an_event_when_unchanged() {
        let mut tc: TestContext = before_each();
        tc.set_fee_protocol(5, 9);
        tc.set_fee_protocol(5, 9);
        let set_fee_protocol_event: SetFeeProtocol = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(set_fee_protocol_event.fee_protocol0_old == 5);
        assert!(set_fee_protocol_event.fee_protocol1_old == 9);
        assert!(set_fee_protocol_event.fee_protocol0_new == 5);
        assert!(set_fee_protocol_event.fee_protocol1_new == 9);
    }
}
