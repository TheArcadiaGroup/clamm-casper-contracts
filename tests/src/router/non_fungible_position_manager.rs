#![allow(dead_code)]
use casper_types::U256;

fn token_id() -> U256 {
    1.into()
}
#[cfg(test)]
mod create_and_initialize_pool_if_necessary {
    use casper_types::bytesrepr::Bytes;
    use contract_utilities::helpers;

    use crate::{
        pool::fixture::FEE_MEDIUM,
        router::fixture::setup_fixture,
        utils::{encode_price_sqrt, wallet},
    };

    #[test]
    fn test_works_if_pool_is_created_but_not_initialized() {
        let mut tc = setup_fixture();
        tc.create_pool(tc.token0, tc.token1, FEE_MEDIUM);
        tc.create_and_initialize_pool_if_necessary(
            tc.token0,
            tc.token1,
            FEE_MEDIUM,
            &encode_price_sqrt(2, 1),
        );
    }

    #[test]
    fn test_works_if_pool_is_created_and_initialized() {
        let mut tc = setup_fixture();
        tc.create_pool(tc.token0, tc.token1, FEE_MEDIUM);
        let pool = tc.get_pool(tc.token0, tc.token1, FEE_MEDIUM);
        tc.initialize_pool_price(pool, &encode_price_sqrt(3, 1));
        tc.create_and_initialize_pool_if_necessary(
            tc.token0,
            tc.token1,
            FEE_MEDIUM,
            &encode_price_sqrt(4, 1),
        );
    }

    #[test]
    fn test_could_use_multicall_session() {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary"],
            vec![Bytes::from(helpers::encode_4(
                &tc.token0,
                &tc.token1,
                &FEE_MEDIUM,
                &encode_price_sqrt(1, 1),
            ))],
            0.into(),
        );
    }
}

#[cfg(test)]
mod mint {
    use casper_types::{Key, U256};
    use contract_utilities::helpers;

    use crate::{
        pool::fixture::{get_tick_spacing, FEE_MEDIUM},
        router::fixture::setup_fixture,
        utils::{encode_price_sqrt, get_max_tick, get_min_tick, other, wallet},
    };

    #[test]
    #[should_panic]
    fn test_fails_if_pool_does_not_exist() {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["mint"],
            vec![helpers::encode_12(
                &tc.token0,
                &tc.token1,
                &false,
                &FEE_MEDIUM,
                &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                &U256::from(100),
                &U256::from(100),
                &U256::from(0),
                &U256::from(0),
                &Key::from(wallet()),
                &99999999999_u64,
            )
            .into()],
            0.into(),
        );
    }

    #[test]
    fn test_create_a_token() {
        let mut tc = setup_fixture();

        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary"],
            vec![helpers::encode_4(
                &tc.token0,
                &tc.token1,
                &FEE_MEDIUM,
                &encode_price_sqrt(1, 1),
            )
            .into()],
            0.into(),
        );

        tc.multicall_liquidity_session(
            wallet(),
            vec!["mint"],
            vec![helpers::encode_12(
                &tc.token0,
                &tc.token1,
                &false,
                &FEE_MEDIUM,
                &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                &U256::from(15),
                &U256::from(15),
                &U256::from(0),
                &U256::from(0),
                &Key::from(other()),
                &99999999999_u64,
            )
            .into()],
            0.into(),
        );

        assert!(tc.cep47_balance_of(other().into()).eq(&1.into()));
        assert!(tc
            .token_of_owner_by_index(other().into(), 0.into())
            .eq(&1.into()));

        let position = tc.position(1.into());
        assert!(position.token0 == tc.token0);
        assert!(position.token1 == tc.token1);
        assert!(position.fee == FEE_MEDIUM);
        assert!(position.tick_lower == get_min_tick(get_tick_spacing(FEE_MEDIUM)));
        assert!(position.tick_upper == get_max_tick(get_tick_spacing(FEE_MEDIUM)));
        assert!(position.liquidity.as_u128() == 15);
        assert!(position.tokens_owed0.as_u128() == 0);
        assert!(position.tokens_owed1.as_u128() == 0);
        assert!(position.fee_growth_inside0_last_x128.as_u128() == 0);
        assert!(position.fee_growth_inside1_last_x128.as_u128() == 0);
    }

    #[test]
    fn test_can_use_multicall_session() {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary", "mint"],
            vec![
                helpers::encode_4(
                    &tc.token0,
                    &tc.token1,
                    &FEE_MEDIUM,
                    &encode_price_sqrt(1, 1),
                )
                .into(),
                helpers::encode_12(
                    &tc.token0,
                    &tc.token1,
                    &false,
                    &FEE_MEDIUM,
                    &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                    &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                    &U256::from(100),
                    &U256::from(100),
                    &U256::from(0),
                    &U256::from(0),
                    &Key::from(other()),
                    &99999999999_u64,
                )
                .into(),
            ],
            0.into(),
        );
    }
}

#[cfg(test)]
mod increase_liquidity {
    use casper_types::{Key, U256, U512};
    use contract_utilities::helpers;

    use crate::{
        pool::fixture::{get_tick_spacing, FEE_MEDIUM},
        router::{
            fixture::{setup_fixture, TestContext},
            non_fungible_position_manager::token_id,
        },
        utils::{encode_price_sqrt, get_max_tick, get_min_tick, other, sort_tokens, wallet},
    };
    fn before_each() -> TestContext {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary", "mint"],
            vec![
                helpers::encode_4(
                    &tc.token0,
                    &tc.token1,
                    &FEE_MEDIUM,
                    &encode_price_sqrt(1, 1),
                )
                .into(),
                helpers::encode_12(
                    &tc.token0,
                    &tc.token1,
                    &false,
                    &FEE_MEDIUM,
                    &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                    &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                    &U256::from(1000),
                    &U256::from(1000),
                    &U256::from(0),
                    &U256::from(0),
                    &Key::from(other()),
                    &99999999999_u64,
                )
                .into(),
            ],
            0.into(),
        );
        tc
    }

    #[test]
    fn test_increase_position_liquidity() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["increase_liquidity"],
            vec![helpers::encode_9(
                &tc.token0,
                &tc.token1,
                &false,
                &token_id(),
                &U256::from(100),
                &U256::from(100),
                &U256::from(0),
                &U256::from(0),
                &99999999999_u64,
            )
            .into()],
            0.into(),
        );
        let position = tc.position(token_id());
        assert!(position.liquidity.as_u128() == 1100);
    }

    #[test]
    fn test_can_be_paid_with_cspr() {
        let mut tc = before_each();
        let (token0, token1) = sort_tokens(tc.token0, tc.wcspr);

        tc.multicall_liquidity_session(
            wallet(),
            vec![
                "create_and_initialize_pool_if_necessary",
                "mint",
                "unwrap_cspr",
            ],
            vec![
                helpers::encode_4(&token0, &token1, &FEE_MEDIUM, &encode_price_sqrt(1, 1)).into(),
                helpers::encode_12(
                    &token0,
                    &token1,
                    &true,
                    &FEE_MEDIUM,
                    &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                    &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                    &U256::from(100),
                    &U256::from(100),
                    &U256::from(0),
                    &U256::from(0),
                    &Key::from(other()),
                    &99999999999_u64,
                )
                .into(),
                helpers::encode_2(&U256::zero(), &Key::from(other())).into(),
            ],
            U512::from(100),
        );

        tc.multicall_liquidity_session(
            wallet(),
            vec!["increase_liquidity"],
            vec![helpers::encode_9(
                &tc.token0,
                &tc.token1,
                &false,
                &token_id(),
                &U256::from(100),
                &U256::from(100),
                &U256::from(0),
                &U256::from(0),
                &99999999999_u64,
            )
            .into()],
            U512::from(0),
        );
    }
}

#[cfg(test)]
mod decrease_liquidity {
    use casper_types::{Key, U128, U256};
    use contract_utilities::helpers;

    use crate::{
        pool::fixture::{get_tick_spacing, FEE_MEDIUM},
        router::fixture::{setup_fixture, TestContext},
        utils::{encode_price_sqrt, get_max_tick, get_min_tick, other, wallet},
    };

    use super::token_id;
    fn before_each() -> TestContext {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary", "mint"],
            vec![
                helpers::encode_4(
                    &tc.token0,
                    &tc.token1,
                    &FEE_MEDIUM,
                    &encode_price_sqrt(1, 1),
                )
                .into(),
                helpers::encode_12(
                    &tc.token0,
                    &tc.token1,
                    &false,
                    &FEE_MEDIUM,
                    &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                    &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                    &U256::from(100),
                    &U256::from(100),
                    &U256::from(0),
                    &U256::from(0),
                    &Key::from(other()),
                    &99999999999_u64,
                )
                .into(),
            ],
            0.into(),
        );
        tc
    }

    #[test]
    #[should_panic = "User(15049)"]
    fn test_fails_if_past_deadline() {
        let mut tc = before_each();
        tc.test_env.set_block_time(2);
        tc.multicall_liquidity_session(
            other(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(50),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
    }

    #[test]
    #[should_panic = "User(15052)"]
    fn test_cannot_be_called_by_other_addresses() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(50),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
    }

    #[test]
    fn test_decreases_position_liquidity() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            other(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(25),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
        let position = tc.position(token_id());
        assert!(position.liquidity.as_u128() == 75);
    }

    #[test]
    fn test_accounts_for_tokens_owed() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            other(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(25),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
        let position = tc.position(token_id());
        assert!(position.tokens_owed0.as_u128() == 24);
        assert!(position.tokens_owed1.as_u128() == 24);
    }

    #[test]
    fn test_can_decrease_for_all_the_liquidity() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            other(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(100),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
        let position = tc.position(token_id());
        assert!(position.liquidity.as_u128() == 0);
    }

    #[test]
    #[should_panic]
    fn test_cannot_decrease_for_more_than_the_liquidity_of_the_nft_position() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            other(),
            vec!["mint"],
            vec![helpers::encode_12(
                &tc.token0,
                &tc.token1,
                &false,
                &FEE_MEDIUM,
                &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                &U256::from(200),
                &U256::from(200),
                &U256::from(0),
                &U256::from(0),
                &Key::from(other()),
                &99999999999_u64,
            )
            .into()],
            0.into(),
        );
        tc.multicall_liquidity_session(
            other(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(101),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
    }
}

#[cfg(test)]
mod collect {
    use std::ops::Add;

    use casper_types::{Key, U128, U256};
    use contract_utilities::helpers;

    use crate::{
        pool::fixture::{get_tick_spacing, FEE_MEDIUM},
        router::fixture::{setup_fixture, TestContext},
        utils::{encode_price_sqrt, get_max_tick, get_min_tick, other, wallet},
    };

    use super::token_id;
    fn before_each() -> TestContext {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary", "mint"],
            vec![
                helpers::encode_4(
                    &tc.token0,
                    &tc.token1,
                    &FEE_MEDIUM,
                    &encode_price_sqrt(1, 1),
                )
                .into(),
                helpers::encode_12(
                    &tc.token0,
                    &tc.token1,
                    &false,
                    &FEE_MEDIUM,
                    &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                    &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                    &U256::from(100),
                    &U256::from(100),
                    &U256::from(0),
                    &U256::from(0),
                    &Key::from(other()),
                    &99999999999_u64,
                )
                .into(),
            ],
            0.into(),
        );
        tc
    }

    #[test]
    #[should_panic = "User(15052)"]
    fn test_cannot_be_called_by_other_addresses() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["collect"],
            vec![helpers::encode_5(
                &false,
                &token_id(),
                &Key::from(wallet()),
                &U128::from(0),
                &U128::from(0),
            )
            .into()],
            0.into(),
        );
    }
    #[test]
    #[should_panic = "User(15054)"]
    fn test_cannot_be_called_with_0_for_both_amounts() {
        let mut tc: TestContext = before_each();
        tc.multicall_liquidity_session(
            other(),
            vec!["collect"],
            vec![helpers::encode_5(
                &false,
                &token_id(),
                &Key::from(wallet()),
                &U128::from(0),
                &U128::from(0),
            )
            .into()],
            0.into(),
        );
    }

    #[test]
    fn test_no_op_if_no_tokens_are_owed() {
        let mut tc: TestContext = before_each();
        let bal0_before = tc.test_env.balance_of(tc.token0, wallet().into());
        let bal1_before = tc.test_env.balance_of(tc.token1, wallet().into());
        tc.multicall_liquidity_session(
            other(),
            vec!["collect"],
            vec![helpers::encode_5(
                &false,
                &token_id(),
                &Key::from(wallet()),
                &U128::MAX,
                &U128::MAX,
            )
            .into()],
            0.into(),
        );
        assert!(tc
            .test_env
            .balance_of(tc.token0, wallet().into())
            .eq(&bal0_before));
        assert!(tc
            .test_env
            .balance_of(tc.token1, wallet().into())
            .eq(&bal1_before));
    }

    #[test]
    fn test_transfers_tokens_owed_from_burn() {
        let mut tc: TestContext = before_each();
        tc.multicall_liquidity_session(
            other(),
            vec!["decrease_liquidity"],
            vec![helpers::encode_5(
                &token_id(),
                &U128::from(50),
                &U256::from(0),
                &U256::from(0),
                &1_u64,
            )
            .into()],
            0.into(),
        );
        let bal0_before = tc.test_env.balance_of(tc.token0, wallet().into());
        let bal1_before = tc.test_env.balance_of(tc.token1, wallet().into());
        tc.multicall_liquidity_session(
            other(),
            vec!["collect"],
            vec![helpers::encode_5(
                &false,
                &token_id(),
                &Key::from(wallet()),
                &U128::MAX,
                &U128::MAX,
            )
            .into()],
            0.into(),
        );
        assert!(tc
            .test_env
            .balance_of(tc.token0, wallet().into())
            .eq(&bal0_before.add(49)));
        assert!(tc
            .test_env
            .balance_of(tc.token1, wallet().into())
            .eq(&bal1_before.add(49)));
    }
}

#[cfg(test)]
mod burn {
    use casper_types::{Key, U256};
    use contract_utilities::helpers;

    use crate::{
        pool::fixture::{get_tick_spacing, FEE_MEDIUM},
        router::fixture::{setup_fixture, TestContext},
        utils::{encode_price_sqrt, get_max_tick, get_min_tick, other, wallet},
    };

    use super::token_id;
    fn before_each() -> TestContext {
        let mut tc = setup_fixture();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["create_and_initialize_pool_if_necessary", "mint"],
            vec![
                helpers::encode_4(
                    &tc.token0,
                    &tc.token1,
                    &FEE_MEDIUM,
                    &encode_price_sqrt(1, 1),
                )
                .into(),
                helpers::encode_12(
                    &tc.token0,
                    &tc.token1,
                    &false,
                    &FEE_MEDIUM,
                    &get_min_tick(get_tick_spacing(FEE_MEDIUM)),
                    &get_max_tick(get_tick_spacing(FEE_MEDIUM)),
                    &U256::from(100),
                    &U256::from(100),
                    &U256::from(0),
                    &U256::from(0),
                    &Key::from(other()),
                    &99999999999_u64,
                )
                .into(),
            ],
            0.into(),
        );
        tc
    }

    #[test]
    #[should_panic = "User(15052)"]
    fn test_cannot_be_called_by_other_addresses() {
        let mut tc = before_each();
        tc.multicall_liquidity_session(
            wallet(),
            vec!["burn"],
            vec![helpers::encode_1(&token_id()).into()],
            0.into(),
        );
    }

    #[test]
    // #[should_panic="User(15052)"]
    fn test_cannot_be_called_while_there_is_still_liquidity() {
        // let mut tc = before_each();
        // tc.multicall_liquidity_session(
        //     wallet(),
        //     vec!["burn"],
        //     vec![
        //         helpers::encode_1(
        //             &token_id(),
        //         )
        //         .into(),
        //     ],
        //     0.into()
        // );
    }
}
