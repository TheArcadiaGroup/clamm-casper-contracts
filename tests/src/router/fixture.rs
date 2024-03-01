#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use casper_types::{
    account::AccountHash, bytesrepr::Bytes, runtime_args, Key, RuntimeArgs, U128, U256, U512,
};
use std::{collections::BTreeMap, ops::Div};
use test_env::env::TestEnv;
use types::Position;

use crate::{
    constants,
    utils::{self, expand_to_18_decimals, other, wallet},
};

pub struct TestContext {
    pub test_env: TestEnv,
    pub factory: Key,
    pub token0: Key,
    pub token1: Key,
    pub token2: Key,
    pub router: Key,
    pub wcspr: Key,
}

pub fn setup_fixture() -> TestContext {
    let mut test_env = TestEnv::new(&[wallet(), other()], 0);
    // test_env.deploy_contract(Some(wallet()), constants::TEST_SESSION, runtime_args! {});

    // let test_session = test_env.get_contract_package_hash(
    //     wallet(),
    //     &utils::get_contract_package_hash_key("test_session".to_string()),
    // );

    test_env.deploy_contract(
        Some(wallet()),
        constants::FACTORY,
        runtime_args! {
            "contract_name" => "factory",
        },
    );
    let factory = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key("factory".to_string()),
    );

    test_env.deploy_contract(
        Some(wallet()),
        constants::ERC20_TOKEN,
        runtime_args! {
            "name" => "T0",
            "symbol" => "T0",
            "decimals" => 18u8,
            "total_supply" =>
            U256::MAX.div(2),
            "minter_list" => vec![Key::from(wallet())],
            "admin_list" => vec![Key::from(wallet())],
            "enable_mint_burn" => 1u8,
            "contract_name" => "T0",
        },
    );
    let token0 = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key_cep18("T0".to_string()),
    );

    test_env.deploy_contract(
        Some(wallet()),
        constants::ERC20_TOKEN,
        runtime_args! {
            "name" => "T1",
            "symbol" => "T1",
            "decimals" => 18u8,
            "total_supply" =>
            U256::MAX.div(2),
            "minter_list" => vec![Key::from(wallet())],
            "admin_list" => vec![Key::from(wallet())],
            "enable_mint_burn" => 1u8,
            "contract_name" => "T1",
        },
    );
    let token1 = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key_cep18("T1".to_string()),
    );
    let (token0, token1) = if token0.into_hash().unwrap() < token1.into_hash().unwrap() {
        (token0, token1)
    } else {
        (token1, token0)
    };

    test_env.deploy_contract(
        Some(wallet()),
        constants::ERC20_TOKEN,
        runtime_args! {
            "name" => "T2",
            "symbol" => "T2",
            "decimals" => 18u8,
            "total_supply" =>
            U256::MAX.div(2),
            "minter_list" => vec![Key::from(wallet())],
            "admin_list" => vec![Key::from(wallet())],
            "enable_mint_burn" => 1u8,
            "contract_name" => "T2",
        },
    );
    let token2 = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key_cep18("T2".to_string()),
    );

    println!("token0 {}", token0);

    for token in &vec![token0, token1, token2] {
        test_env.transfer(
            *token,
            wallet(),
            other().into(),
            expand_to_18_decimals(1000000),
        );
    }

    test_env.deploy_contract(
        Some(wallet()),
        constants::ROUTER,
        runtime_args! {
            "contract_name" => "router",
            "factory" => factory,
            "meta" => BTreeMap::<String, String>::new(),
        },
    );
    let router = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key("router".to_string()),
    );
    println!("router {}", router);

    test_env.deploy_contract(
        Some(wallet()),
        constants::WCSPR,
        runtime_args! {
            "name" => "wcspr",
            "symbol" => "WCSPR",
            "decimals" => 9_u8,
            "initial_supply" => U256::from(0),
            "contract_name" => "wcspr"
        },
    );

    let wcspr = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key("wcspr".to_string()),
    );

    TestContext {
        test_env,
        factory,
        token0,
        token1,
        token2,
        router,
        wcspr,
    }
}

impl TestContext {
    pub fn get_liquidity_for_amounts(
        &mut self,
        sqrt_ratio_x96: &U256,
        sqrt_ratio_a_x96: &U256,
        sqrt_ratio_b_x96: &U256,
        amount0: &U256,
        amount1: &U256,
    ) -> U128 {
        self.test_env.call_view_function(
            self.router,
            "get_liquidity_for_amounts",
            runtime_args! {
                "sqrt_ratio_x96" => sqrt_ratio_x96,
                "sqrt_ratio_a_x96" => sqrt_ratio_a_x96,
                "sqrt_ratio_b_x96" => sqrt_ratio_b_x96,
                "amount0" => amount0,
                "amount1" => amount1,
            },
        )
    }

    pub fn get_amounts_for_liquidity(
        &mut self,
        sqrt_ratio_x96: &U256,
        sqrt_ratio_a_x96: &U256,
        sqrt_ratio_b_x96: &U256,
        liquidity: u128,
    ) -> (U256, U256) {
        self.test_env.call_view_function(
            self.router,
            "get_amounts_for_liquidity",
            runtime_args! {
                "sqrt_ratio_x96" => sqrt_ratio_x96,
                "sqrt_ratio_a_x96" => sqrt_ratio_a_x96,
                "sqrt_ratio_b_x96" => sqrt_ratio_b_x96,
                "liquidity" => U128::from(liquidity),
            },
        )
    }

    pub fn create_pool(&mut self, token0: Key, token1: Key, fee: u32) {
        self.test_env.call_contract(
            Some(wallet()),
            self.factory.into_hash().unwrap().into(),
            "create_pool",
            runtime_args! {
                "token0" => token0,
                "token1" => token1,
                "fee" => fee,
            },
            true,
        );
    }

    pub fn create_and_initialize_pool_if_necessary(
        &mut self,
        token0: Key,
        token1: Key,
        fee: u32,
        sqrt_price_x96: &U256,
    ) {
        self.test_env.call_contract(
            Some(wallet()),
            self.router.into_hash().unwrap().into(),
            "create_and_initialize_pool_if_necessary",
            runtime_args! {
                "token0" => token0,
                "token1" => token1,
                "fee" => fee,
                "sqrt_price_x96" => sqrt_price_x96
            },
            true,
        );
    }

    pub fn initialize_pool_price(&mut self, pool: Key, price: &U256) {
        self.test_env.call_contract(
            Some(wallet()),
            pool.into_hash().unwrap().into(),
            "init_pool_price",
            runtime_args! {
                "sqrt_price_x96" => price
            },
            true,
        );
    }

    pub fn get_pool(&mut self, token0: Key, token1: Key, fee: u32) -> Key {
        self.test_env.call_view_function(
            self.factory,
            "get_pool_address",
            runtime_args! {
                "token0" => token0,
                "token1" => token1,
                "fee" => fee,
            },
        )
    }

    pub fn multicall_liquidity_session(
        &mut self,
        caller: AccountHash,
        entry_points: Vec<&str>,
        datas: Vec<Bytes>,
        amount: U512,
    ) {
        self.test_env.deploy_contract(
            Some(caller),
            constants::LIQUIDITY_SESSION,
            runtime_args! {
                "entry_points" => entry_points,
                "datas" => datas,
                "router" => self.router,
                "wcspr" => self.wcspr,
                "amount" => amount
            },
        );
    }

    pub fn cep47_balance_of(&mut self, owner: Key) -> U256 {
        self.test_env.call_view_function(
            self.router,
            "balance_of",
            runtime_args! {
                "owner" => owner,
            },
        )
    }

    pub fn token_of_owner_by_index(&mut self, owner: Key, index: U256) -> U256 {
        let ret: Option<U256> = self.test_env.call_view_function(
            self.router,
            "get_token_by_index",
            runtime_args! {
                "owner" => owner,
                "index" => index,
            },
        );
        ret.unwrap()
    }

    pub fn position(&mut self, token_id: U256) -> Position {
        self.test_env.call_view_function(
            self.router,
            "get_position",
            runtime_args! {
                "token_id" => token_id,
            },
        )
    }
}
