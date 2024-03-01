#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use super::fixture::setup;
use crate::utils::{call_and_get_any, get_max_liquidity_per_tick};
use casper_types::{runtime_args, Key, RuntimeArgs, U128};
use types::Observation;

#[test]
fn test_constructor_initialize_immutables() {
    let mut tc = setup();
    println!("pool {:?}", tc.pool);
    let factory: Key = tc
        .test_env
        .call_view_function(tc.pool, "get_factory", runtime_args! {});
    let token0: Key = tc
        .test_env
        .call_view_function(tc.pool, "get_token0", runtime_args! {});
    let token1: Key = tc
        .test_env
        .call_view_function(tc.pool, "get_token1", runtime_args! {});

    let max_liquidity_per_tick: U128 =
        tc.test_env
            .call_view_function(tc.pool, "get_max_liquidity_per_tick", runtime_args! {});

    assert!(factory == tc.factory);
    assert!(token0 == tc.token0);
    assert!(token1 == tc.token1);
    assert!(max_liquidity_per_tick == get_max_liquidity_per_tick(tc.tick_spacing))
}

#[cfg(test)]
mod initialize {
    use std::ops::{Shl, Sub};

    use casper_types::U256;
    use common::pool_events::Initialize;
    use math::{
        sqrt_price_math,
        tickmath::{max_sqrt_ratio, min_sqrt_ratio},
    };
    use types::Observation;

    use crate::{
        pool::fixture::{setup, TEST_POOL_START_TIME},
        utils::{encode_price_sqrt, get_max_tick, get_min_tick, wallet, warp_fake_timestamp},
    };

    #[test]
    #[should_panic = "User(15042)"]
    fn test_fail_if_already_initialized() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
    }

    #[test]
    #[should_panic = "User(15009)"]
    fn test_fail_starting_price_too_low_1() {
        let mut tc = setup();
        tc.initialize_pool_price(1.into());
    }

    #[test]
    #[should_panic = "User(15009)"]
    fn test_fail_starting_price_too_low_2() {
        let mut tc = setup();
        tc.initialize_pool_price(min_sqrt_ratio() - U256::one());
    }

    #[test]
    #[should_panic = "User(15009)"]
    fn test_fail_starting_price_too_high_1() {
        let mut tc = setup();
        tc.initialize_pool_price(max_sqrt_ratio());
    }

    #[test]
    #[should_panic = "User(15009)"]
    fn test_fail_starting_price_too_high_2() {
        let mut tc = setup();
        tc.initialize_pool_price(U256::one().shl(160) - U256::one());
    }

    #[test]
    fn test_can_initialize_at_min_sqrt_ratio() {
        let mut tc = setup();
        tc.initialize_pool_price(min_sqrt_ratio());
        let slot0 = tc.get_slot0();
        assert!(slot0.tick == get_min_tick(1));
    }

    #[test]
    fn test_can_initialize_at_max_sqrt_ratio() {
        let mut tc = setup();
        tc.initialize_pool_price(max_sqrt_ratio().sub(1));
        let slot0 = tc.get_slot0();
        assert!(slot0.tick == get_max_tick(1) - 1);
    }

    #[test]
    fn test_set_initial_variable() {
        let mut tc = setup();
        let price = encode_price_sqrt(1, 2);
        tc.initialize_pool_price(price);
        let slot0 = tc.get_slot0();
        assert!(slot0.sqrt_price_x96 == price);
        assert!(slot0.observation_index == 0);
        assert!(slot0.tick == -6932);
    }

    #[test]
    fn test_initializes_first_observations_slot() {
        let mut tc = setup();
        let price = encode_price_sqrt(1, 1);
        tc.test_env.set_block_time(TEST_POOL_START_TIME);

        tc.initialize_pool_price(price);
        let observation = tc.get_observation(0);
        assert!(
            observation
                == Observation {
                    seconds_per_liquidity_cumulative_x128: 0.into(),
                    initialized: true,
                    block_timestamp: TEST_POOL_START_TIME,
                    tick_cumulative: 0
                }
        );
    }

    #[test]
    fn test_emit_an_initialize_event_with_input_tick() {
        let mut tc = setup();
        let sqrt_price_x96 = encode_price_sqrt(1, 2);
        tc.initialize_pool_price(sqrt_price_x96);
        let initialize_event: Initialize = tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(initialize_event.sqrt_price_x96 == sqrt_price_x96);
        assert!(initialize_event.tick == -6932);
    }
}

#[cfg(test)]
mod test_increase_observation_cardinality_next {
    use casper_types::{account::AccountHash, RuntimeArgs};
    use common::pool_events::IncreaseObservationCardinalityNext;

    use crate::{
        pool::fixture::{setup, TestContext},
        utils::{encode_price_sqrt, wallet},
    };

    #[test]
    #[should_panic = "User(15048)"]
    fn test_can_only_be_called_after_initialize() {
        let mut tc = setup();
        tc.increase_observation_cardinality_next(Some(wallet()), 2);
    }
    #[test]
    fn test_emit_an_event_with_both_old_and_new() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.increase_observation_cardinality_next(Some(wallet()), 2);
        let event_data: IncreaseObservationCardinalityNext =
            tc.test_env.get_last_event(tc.pool).unwrap();
        assert!(event_data.observation_cardinality_next_old == 1);
        assert!(event_data.observation_cardinality_next_new == 2);
    }

    #[test]
    fn test_does_not_emit_an_event_for_no_op_call() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.increase_observation_cardinality_next(Some(wallet()), 3);
        let event_length = tc.test_env.get_event_length(tc.pool);
        tc.increase_observation_cardinality_next(Some(wallet()), 2);
        let event_length_after = tc.test_env.get_event_length(tc.pool);
        assert!(event_length == event_length_after);
    }

    #[test]
    fn test_does_not_change_cardinality_next_uf_less_than_current() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.increase_observation_cardinality_next(Some(wallet()), 3);
        tc.increase_observation_cardinality_next(Some(wallet()), 2);
        let slot0 = tc.get_slot0();
        assert!(slot0.observation_cardinality_next == 3);
    }

    #[test]
    fn test_increases_cardinality_and_cardinality_next_first_time() {
        let mut tc = setup();
        tc.initialize_pool_price(encode_price_sqrt(1, 1));
        tc.increase_observation_cardinality_next(Some(wallet()), 2);
        let slot0 = tc.get_slot0();
        assert!(slot0.observation_cardinality == 1);
        assert!(slot0.observation_cardinality_next == 2);
    }
}

#[cfg(test)]
mod test_mint {
    use casper_types::Key;

    use crate::{pool::fixture::setup, utils::wallet};

    #[test]
    #[should_panic = "User(15048)"]
    fn test_fails_if_not_initialized() {
        let mut tc = setup();
        tc.mint(
            Key::from(wallet()),
            -tc.tick_spacing,
            tc.tick_spacing,
            1.into(),
        );
    }
    #[cfg(test)]
    mod after_initialization {
        use std::ops::{Add, Div};

        use casper_types::Key;

        use crate::{
            pool::fixture::{setup, TestContext},
            utils::{encode_price_sqrt, expand_to_18_decimals, other, wallet},
        };

        // initialize the pool at price of 10:1
        fn before_each() -> TestContext {
            let mut tc = setup();
            tc.initialize_pool_price(encode_price_sqrt(1, 10));
            tc.mint(Key::from(wallet()), tc.min_tick, tc.max_tick, 3161.into());
            tc
        }

        #[cfg(test)]
        mod failure_cases {
            use std::ops::Sub;

            use super::*;
            #[test]
            #[should_panic]
            fn test_fails_if_tick_lower_greater_than_tick_upper() {
                let mut tc = before_each();
                tc.mint(Key::from(wallet()), 1, 0, 1.into());
            }

            #[test]
            #[should_panic]
            fn test_fails_if_tick_lower_less_than_min_tick() {
                let mut tc = before_each();
                tc.mint(Key::from(wallet()), -887273, 0, 1.into());
            }

            #[test]
            #[should_panic]
            fn test_fails_if_tick_upper_greater_than_max_tick() {
                let mut tc = before_each();
                tc.mint(Key::from(wallet()), 0, 887273, 1.into());
            }

            #[test]
            fn test_fails_if_amount_exceeds_max() {
                let mut tc = before_each();
                let max_liquidity_gross = tc.get_max_liquidity_per_tick();
                tc.mint_fail(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing,
                    max_liquidity_gross.add(1),
                );
                tc.mint(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing,
                    max_liquidity_gross,
                );
            }

            #[test]
            fn test_fails_if_total_amount_at_tick_exceeds_the_max() {
                let mut tc = before_each();
                tc.mint(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing,
                    1000.into(),
                );
                let max_liquidity_gross = tc.get_max_liquidity_per_tick();
                tc.mint_fail(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing,
                    max_liquidity_gross.sub(1000).add(1),
                );
                tc.mint_fail(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing * 2,
                    tc.max_tick - tc.tick_spacing,
                    max_liquidity_gross.sub(1000).add(1),
                );

                tc.mint_fail(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing * 2,
                    max_liquidity_gross.sub(1000).add(1),
                );

                tc.mint(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing,
                    max_liquidity_gross.sub(1000),
                );
            }

            #[test]
            fn test_fails_if_amount_0() {
                let mut tc = before_each();
                tc.mint_fail(
                    Key::from(wallet()),
                    tc.min_tick + tc.tick_spacing,
                    tc.max_tick - tc.tick_spacing,
                    0.into(),
                );
            }
        }

        #[cfg(test)]
        mod success_cases {
            use std::ops::Sub;

            use super::*;

            #[test]
            fn test_initial_balances() {
                let mut tc = before_each();
                assert!(tc.test_env.balance_of(tc.token0, tc.pool).eq(&9996.into()));
                assert!(tc.test_env.balance_of(tc.token1, tc.pool).eq(&1000.into()));
            }

            #[test]
            fn test_initial_tick() {
                let mut tc = before_each();
                assert!(tc.get_slot0().tick == -23028);
            }
            #[cfg(test)]
            mod above_current_price {
                use std::ops::Shl;

                use casper_types::{U128, U256};
                use common::pool_events::Collect;

                use crate::pool::fixture::TEST_POOL_START_TIME;

                use super::*;
                #[test]
                fn test_transfers_token0_only() {
                    let mut tc = before_each();
                    assert!(tc.get_slot0().tick == -23028);
                    tc.mint(Key::from(wallet()), -22980, 0, 10000.into());

                    assert!(tc
                        .test_env
                        .balance_of(tc.token0, tc.pool)
                        .eq(&(9996 + 21549).into()));
                    assert!(tc.test_env.balance_of(tc.token1, tc.pool).eq(&1000.into()));
                }

                #[test]
                fn test_max_tick_with_max_leverage() {
                    let mut tc = before_each();
                    assert!(tc.get_slot0().tick == -23028);
                    tc.mint(
                        Key::from(wallet()),
                        tc.max_tick - tc.tick_spacing,
                        tc.max_tick,
                        U128::from(1).shl(102),
                    );

                    assert!(tc
                        .test_env
                        .balance_of(tc.token0, tc.pool)
                        .eq(&(9996 + 828011525).into()));
                    assert!(tc.test_env.balance_of(tc.token1, tc.pool).eq(&1000.into()));
                }

                #[test]
                fn test_works_for_max_tick() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -22980, tc.max_tick, 10000.into());

                    assert!(tc
                        .test_env
                        .balance_of(tc.token0, tc.pool)
                        .eq(&(9996 + 31549).into()));
                    assert!(tc.test_env.balance_of(tc.token1, tc.pool).eq(&1000.into()));
                }

                #[test]
                fn test_removing_works() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -240, 0, 10000.into());
                    tc.burn(wallet(), -240, 0, 10000.into());
                    tc.collect(wallet().into(), -240, 0, U128::MAX, U128::MAX);
                    let collect_event_data: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
                    assert!(collect_event_data.amount0.eq(&120.into()));
                    assert!(collect_event_data.amount1.eq(&0.into()));
                }

                #[test]
                fn test_adds_liquidity_to_liquidity_gross() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -240, 0, 100.into());
                    {
                        assert!(tc.get_tick(-240).liquidity_gross.eq(&100.into()));
                        assert!(tc.get_tick(0).liquidity_gross.eq(&100.into()));
                        assert!(tc.get_tick(tc.tick_spacing).liquidity_gross.eq(&0.into()));
                        assert!(tc
                            .get_tick(tc.tick_spacing * 2)
                            .liquidity_gross
                            .eq(&0.into()));
                    }

                    tc.mint(Key::from(wallet()), -240, tc.tick_spacing, 150.into());
                    {
                        assert!(tc.get_tick(-240).liquidity_gross.eq(&250.into()));
                        assert!(tc.get_tick(0).liquidity_gross.eq(&100.into()));
                        assert!(tc.get_tick(tc.tick_spacing).liquidity_gross.eq(&150.into()));
                        assert!(tc
                            .get_tick(tc.tick_spacing * 2)
                            .liquidity_gross
                            .eq(&0.into()));
                    }

                    tc.mint(Key::from(wallet()), 0, tc.tick_spacing * 2, 60.into());
                    {
                        assert!(tc.get_tick(-240).liquidity_gross.eq(&250.into()));
                        assert!(tc.get_tick(0).liquidity_gross.eq(&160.into()));
                        assert!(tc.get_tick(tc.tick_spacing).liquidity_gross.eq(&150.into()));
                        assert!(tc
                            .get_tick(tc.tick_spacing * 2)
                            .liquidity_gross
                            .eq(&60.into()));
                    }
                }

                #[test]
                fn test_remove_liquidity_from_liquidity_gross() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -240, 0, 100.into());
                    tc.mint(Key::from(wallet()), -240, 0, 40.into());
                    tc.burn(wallet(), -240, 0, 90.into());
                    {
                        assert!(tc.get_tick(-240).liquidity_gross.eq(&50.into()));
                        assert!(tc.get_tick(0).liquidity_gross.eq(&50.into()));
                    }
                }

                #[test]
                fn test_clears_tick_lower_if_last_position_is_removed() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -240, 0, 100.into());
                    tc.burn(wallet(), -240, 0, 100.into());
                    {
                        assert!(tc.get_tick(-240).liquidity_gross.eq(&0.into()));
                        assert!(tc.get_tick(-240).fee_growth_outside0_x128.eq(&0.into()));
                        assert!(tc.get_tick(-240).fee_growth_outside1_x128.eq(&0.into()));
                    }
                }

                #[test]
                fn test_only_clears_the_tick_that_is_not_used_at_all() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -240, 0, 100.into());
                    tc.mint(Key::from(wallet()), -tc.tick_spacing, 0, 250.into());
                    tc.burn(wallet(), -240, 0, 100.into());
                    {
                        assert!(tc.get_tick(-240).liquidity_gross.eq(&0.into()));
                        assert!(tc.get_tick(-240).fee_growth_outside0_x128.eq(&0.into()));
                        assert!(tc.get_tick(-240).fee_growth_outside1_x128.eq(&0.into()));
                    }

                    {
                        assert!(tc
                            .get_tick(-tc.tick_spacing)
                            .liquidity_gross
                            .eq(&250.into()));
                        assert!(tc
                            .get_tick(-tc.tick_spacing)
                            .fee_growth_outside0_x128
                            .eq(&0.into()));
                        assert!(tc
                            .get_tick(-tc.tick_spacing)
                            .fee_growth_outside1_x128
                            .eq(&0.into()));
                    }
                }

                #[test]
                fn test_does_not_write_an_observation() {
                    let mut tc = before_each();
                    {
                        let ob0 = tc.get_observation(0);
                        assert!(ob0.tick_cumulative == 0);
                        assert!(ob0.block_timestamp == TEST_POOL_START_TIME);
                        assert!(ob0.initialized);
                        assert!(ob0.seconds_per_liquidity_cumulative_x128 == U256::zero());
                    }
                    tc.test_env.advance_block_time_by(1);
                    tc.mint(Key::from(wallet()), -240, 0, 100.into());
                    {
                        let ob0 = tc.get_observation(0);
                        assert!(ob0.tick_cumulative == 0);
                        assert!(ob0.block_timestamp == TEST_POOL_START_TIME);
                        assert!(ob0.initialized);
                        assert!(ob0.seconds_per_liquidity_cumulative_x128 == U256::zero());
                    }
                }
            }

            #[cfg(test)]
            mod including_current_price {
                use std::ops::Sub;

                use casper_types::{U128, U256};
                use common::pool_events::Collect;

                use crate::pool::fixture::TEST_POOL_START_TIME;

                use super::*;
                #[test]
                fn test_price_within_range_transfers_current_price_of_both_tokens() {
                    let mut tc = before_each();
                    tc.mint(
                        Key::from(wallet()),
                        tc.min_tick + tc.tick_spacing,
                        tc.max_tick - tc.tick_spacing,
                        100.into(),
                    );
                    assert!(tc
                        .test_env
                        .balance_of(tc.token0, tc.pool)
                        .eq(&(9996 + 317).into()));
                    assert!(tc
                        .test_env
                        .balance_of(tc.token1, tc.pool)
                        .eq(&(1000 + 32).into()));
                }

                #[test]
                fn test_initializes_lower_tick() {
                    let mut tc = before_each();
                    tc.mint(
                        Key::from(wallet()),
                        tc.min_tick + tc.tick_spacing,
                        tc.max_tick - tc.tick_spacing,
                        100.into(),
                    );
                    assert!(tc
                        .get_tick(tc.min_tick + tc.tick_spacing)
                        .liquidity_gross
                        .eq(&100.into()));
                }

                #[test]
                fn test_initializes_upper_tick() {
                    let mut tc = before_each();
                    tc.mint(
                        Key::from(wallet()),
                        tc.min_tick + tc.tick_spacing,
                        tc.max_tick - tc.tick_spacing,
                        100.into(),
                    );
                    assert!(tc
                        .get_tick(tc.max_tick - tc.tick_spacing)
                        .liquidity_gross
                        .eq(&100.into()));
                }

                #[test]
                fn test_works_for_min_max_tick() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), tc.min_tick, tc.max_tick, 10000.into());
                    assert!(tc
                        .test_env
                        .balance_of(tc.token0, tc.pool)
                        .eq(&(9996 + 31623).into()));
                    assert!(tc
                        .test_env
                        .balance_of(tc.token1, tc.pool)
                        .eq(&(1000 + 3163).into()));
                }

                #[test]
                fn test_removing_works() {
                    let mut tc = before_each();
                    tc.mint(
                        Key::from(wallet()),
                        tc.min_tick + tc.tick_spacing,
                        tc.max_tick - tc.tick_spacing,
                        100.into(),
                    );
                    tc.burn(
                        wallet(),
                        tc.min_tick + tc.tick_spacing,
                        tc.max_tick - tc.tick_spacing,
                        100.into(),
                    );
                    tc.collect(
                        wallet().into(),
                        tc.min_tick + tc.tick_spacing,
                        tc.max_tick - tc.tick_spacing,
                        U128::MAX,
                        U128::MAX,
                    );
                    let collect_event_data: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
                    assert!(collect_event_data.amount0.eq(&316.into()));
                    assert!(collect_event_data.amount1.eq(&31.into()));
                }

                #[test]
                fn test_write_an_observation() {
                    let mut tc = before_each();
                    {
                        let ob0 = tc.get_observation(0);
                        assert!(ob0.tick_cumulative == 0);
                        assert!(ob0.block_timestamp == TEST_POOL_START_TIME);
                        assert!(ob0.initialized);
                        assert!(ob0.seconds_per_liquidity_cumulative_x128 == U256::zero());
                    }
                    tc.test_env.advance_block_time_by(1);
                    tc.mint(Key::from(wallet()), tc.min_tick, tc.max_tick, 100.into());
                    {
                        let ob0 = tc.get_observation(0);
                        assert!(ob0.tick_cumulative == -23028);
                        assert!(ob0.block_timestamp == TEST_POOL_START_TIME + 1);
                        assert!(ob0.initialized);
                        println!(
                            "ob0.seconds_per_liquidity_cumulative_x128 {:?}",
                            ob0.seconds_per_liquidity_cumulative_x128
                        );
                        assert!(
                            ob0.seconds_per_liquidity_cumulative_x128
                                == U256::from(107650226801941937191829992860413859_u128)
                        );
                    }
                }
            }

            #[cfg(test)]
            mod below_current_price {
                use std::ops::Sub;

                use casper_types::{U128, U256};
                use common::pool_events::Collect;

                use crate::pool::fixture::TEST_POOL_START_TIME;

                use super::*;
                #[test]
                fn test_transfer_token1_only() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -46080, -23040, 10000.into());
                    assert!(tc.test_env.balance_of(tc.token0, tc.pool).eq(&9996.into()));
                    assert!(tc
                        .test_env
                        .balance_of(tc.token1, tc.pool)
                        .eq(&(1000 + 2162).into()));
                }

                #[test]
                fn test_min_tick_with_max_leverage() {
                    let mut tc = before_each();
                    tc.mint(
                        Key::from(wallet()),
                        tc.min_tick,
                        tc.min_tick + tc.tick_spacing,
                        U128::from(2).pow(102.into()),
                    );
                    assert!(tc.test_env.balance_of(tc.token0, tc.pool).eq(&9996.into()));
                    assert!(tc
                        .test_env
                        .balance_of(tc.token1, tc.pool)
                        .eq(&(1000 + 828011520).into()));
                }

                #[test]
                fn test_works_for_min_tick() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), tc.min_tick, -23040, 10000.into());
                    assert!(tc.test_env.balance_of(tc.token0, tc.pool).eq(&9996.into()));
                    assert!(tc
                        .test_env
                        .balance_of(tc.token1, tc.pool)
                        .eq(&(1000 + 3161).into()));
                }

                #[test]
                fn test_removing_works() {
                    let mut tc = before_each();
                    tc.mint(Key::from(wallet()), -46080, -46020, 10000.into());
                    tc.burn(wallet(), -46080, -46020, 10000.into());
                    tc.collect(wallet().into(), -46080, -46020, U128::MAX, U128::MAX);
                    let collect_event_data: Collect = tc.test_env.get_last_event(tc.pool).unwrap();
                    assert!(collect_event_data.amount0.eq(&0.into()));
                    assert!(collect_event_data.amount1.eq(&3.into()));
                }

                #[test]
                fn test_does_not_write_an_observation() {
                    let mut tc = before_each();
                    {
                        let ob0 = tc.get_observation(0);
                        assert!(ob0.tick_cumulative == 0);
                        assert!(ob0.block_timestamp == TEST_POOL_START_TIME);
                        assert!(ob0.initialized);
                        assert!(ob0.seconds_per_liquidity_cumulative_x128 == U256::zero());
                    }
                    tc.test_env.advance_block_time_by(1);
                    tc.mint(Key::from(wallet()), -46080, -23040, 100.into());
                    {
                        let ob0 = tc.get_observation(0);
                        assert!(ob0.tick_cumulative == 0);
                        assert!(ob0.block_timestamp == TEST_POOL_START_TIME);
                        assert!(ob0.initialized);
                        assert!(ob0.seconds_per_liquidity_cumulative_x128.eq(&0.into()));
                    }
                }
            }
        }

        #[test]
        fn test_protocol_fees_accumulate_as_expected_during_swap() {
            let mut tc = before_each();
            tc.set_fee_protocol(6, 6);
            tc.mint(
                Key::from(wallet()),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
                expand_to_18_decimals(1).as_u128().into(),
            );

            tc.swap_exact_0_for_1(expand_to_18_decimals(1).div(10), wallet().into(), None);
            tc.swap_exact_1_for_0(expand_to_18_decimals(1).div(100), wallet().into(), None);
            let ptf = tc.get_protocol_fees();
            assert!(ptf.token0.as_u128() == 50000000000000);
            assert!(ptf.token1.as_u128() == 5000000000000);
        }

        #[test]
        fn test_positions_are_protected_before_protocol_fee_is_turned_on() {
            let mut tc = before_each();
            tc.mint(
                Key::from(wallet()),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
                expand_to_18_decimals(1).as_u128().into(),
            );
            tc.swap_exact_0_for_1(expand_to_18_decimals(1).div(10), wallet().into(), None);
            tc.swap_exact_1_for_0(expand_to_18_decimals(1).div(100), wallet().into(), None);
            let ptf = tc.get_protocol_fees();

            tc.set_fee_protocol(6, 6);

            assert!(ptf.token0.as_u128() == 0);
            assert!(ptf.token1.as_u128() == 0);

            let ptf_after = tc.get_protocol_fees();

            assert!(ptf_after.token0.as_u128() == 0);
            assert!(ptf_after.token1.as_u128() == 0);
        }

        #[test]
        fn test_poke_is_not_allowed_on_uninitialized_position() {
            let mut tc = before_each();
            tc.mint(
                Key::from(other()),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
                expand_to_18_decimals(1).as_u128().into(),
            );
            tc.swap_exact_0_for_1(expand_to_18_decimals(1).div(10), wallet().into(), None);
            tc.swap_exact_1_for_0(expand_to_18_decimals(1).div(100), wallet().into(), None);

            // burn
            tc.burn_fail(
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
                0.into(),
            );

            tc.mint(
                Key::from(wallet()),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
                1.into(),
            );

            let position = tc.get_position(
                wallet().into(),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
            );
            assert!(position.liquidity.as_u128() == 1);
            assert!(position.tokens_owed0.as_u128() == 0);
            assert!(position.tokens_owed1.as_u128() == 0);
            assert!(position
                .fee_growth_inside0_last_x128
                .eq(&102084710076281216349243831104605583_u128.into()));
            assert!(position
                .fee_growth_inside1_last_x128
                .eq(&10208471007628121634924383110460558_u128.into()));

            tc.burn(
                wallet(),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
                1.into(),
            );
            let position = tc.get_position(
                wallet().into(),
                tc.min_tick + tc.tick_spacing,
                tc.max_tick - tc.tick_spacing,
            );
            assert!(position.liquidity.as_u128() == 0);
            assert!(position
                .fee_growth_inside0_last_x128
                .eq(&102084710076281216349243831104605583_u128.into()));
            assert!(position
                .fee_growth_inside1_last_x128
                .eq(&10208471007628121634924383110460558_u128.into()));
            assert!(position.tokens_owed0.as_u128() == 3);
            assert!(position.tokens_owed1.as_u128() == 0);
        }
    }
}
