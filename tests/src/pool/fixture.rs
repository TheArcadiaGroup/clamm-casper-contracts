#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use std::ops::{Add, Div, Mul, Sub};

use crate::constants;
use crate::utils::{
    self, call_and_get_any, encode_price_sqrt, exec_call, get_max_tick, get_min_tick,
    initialize_liquidity_amount, null_key, other, wallet,
};
use casper_engine_test_support::InMemoryWasmTestBuilder;
use casper_types::account::AccountHash;
use casper_types::{runtime_args, Key, RuntimeArgs, U128, U256};
use common::console;
use contract_utilities::helpers::default_cspr_key;
use math::tickmath::{max_sqrt_ratio, min_sqrt_ratio};
use test_env::env::TestEnv;
use types::{Observation, PositionInfo, ProtocolFees, Slot0, TickInfo};

pub const FEE_MEDIUM: u32 = 3000;
pub const FEE_LOW: u32 = 500;
pub const _FEE_HIGH: u32 = 10000;
pub const TEST_POOL_START_TIME: u64 = 1601906400;
pub fn get_tick_spacing(fee: u32) -> i32 {
    if fee == FEE_LOW {
        10
    } else if fee == FEE_MEDIUM {
        60
    } else {
        200
    }
}

pub struct TestContext {
    pub test_session: Key,
    pub test_env: TestEnv,
    pub factory: Key,
    pub token0: Key,
    pub token1: Key,
    pub token2: Key,
    pub swap_target: Key,
    pub min_tick: i32,
    pub max_tick: i32,
    pub fee_amount: u32,
    pub tick_spacing: i32,
    pub pool: Key,
}

pub fn setup_common() -> TestContext {
    let mut test_env = TestEnv::new(&[wallet(), other()], TEST_POOL_START_TIME);
    test_env.deploy_contract(Some(wallet()), constants::TEST_SESSION, runtime_args! {});

    let test_session = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key("test_session".to_string()),
    );

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

    test_env.deploy_contract(
        Some(wallet()),
        constants::TEST_CALLEE,
        runtime_args! {
            "contract_name" => "test_callee"
        },
    );
    let test_callee = test_env.get_contract_package_hash(
        wallet(),
        &utils::get_contract_package_hash_key("test_callee".to_string()),
    );
    println!("test_callee {}", test_callee);

    test_env.approve(token0, wallet(), test_callee, U256::MAX);
    test_env.approve(token1, wallet(), test_callee, U256::MAX);

    TestContext {
        test_session,
        test_env,
        factory,
        token0,
        token1,
        token2,
        swap_target: test_callee,
        min_tick: get_min_tick(get_tick_spacing(FEE_MEDIUM)),
        max_tick: get_max_tick(get_tick_spacing(FEE_MEDIUM)),
        fee_amount: FEE_MEDIUM,
        tick_spacing: get_tick_spacing(FEE_MEDIUM),
        pool: default_cspr_key(),
    }
}

pub fn setup() -> TestContext {
    let mut tc = setup_common();
    tc.test_env.call_contract(
        Some(wallet()),
        tc.factory.into_hash().unwrap().into(),
        "create_pool",
        runtime_args! {
            "token0" => tc.token0,
            "token1" => tc.token1,
            "fee" => FEE_MEDIUM,
        },
        true,
    );
    let pool: Key = tc.test_env.call_view_function(
        tc.factory,
        "get_pool_address",
        runtime_args! {
            "token0" => tc.token0,
            "token1" => tc.token1,
            "fee" => FEE_MEDIUM,
        },
    );
    tc.pool = pool;
    tc
}

pub fn setup_with_fee(fee: u32) -> TestContext {
    let mut tc = setup_common();
    tc.test_env.call_contract(
        Some(wallet()),
        tc.factory.into_hash().unwrap().into(),
        "create_pool",
        runtime_args! {
            "token0" => tc.token0,
            "token1" => tc.token1,
            "fee" => fee,
        },
        true,
    );
    let pool: Key = tc.test_env.call_view_function(
        tc.factory,
        "get_pool_address",
        runtime_args! {
            "token0" => tc.token0,
            "token1" => tc.token1,
            "fee" => fee,
        },
    );
    tc.pool = pool;
    tc.min_tick = get_min_tick(get_tick_spacing(fee));
    tc.max_tick = get_max_tick(get_tick_spacing(fee));
    tc.fee_amount = fee;
    tc.tick_spacing = get_tick_spacing(fee);
    tc
}

impl TestContext {
    pub fn increase_observation_cardinality_next(
        &mut self,
        caller: Option<AccountHash>,
        observation_cardinality_next: u32,
    ) {
        self.test_env.call_contract(
            caller,
            self.pool.into_hash().unwrap().into(),
            "increase_observation_cardinality_next",
            runtime_args! {
                "observation_cardinality_next" => observation_cardinality_next
            },
            true,
        );
    }

    pub fn increase_observation_cardinality_next_fail(
        &mut self,
        caller: Option<AccountHash>,
        observation_cardinality_next: u32,
    ) {
        self.test_env.call_contract(
            caller,
            self.pool.into_hash().unwrap().into(),
            "increase_observation_cardinality_next",
            runtime_args! {
                "observation_cardinality_next" => observation_cardinality_next
            },
            false,
        );
    }

    pub fn initialize_pool_price(&mut self, price: U256) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "init_pool_price",
            runtime_args! {
                "sqrt_price_x96" => price
            },
            true,
        );
    }

    pub fn initialize_pool_price_expect_fail(&mut self, price: U256) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "init_pool_price",
            runtime_args! {
                "sqrt_price_x96" => price
            },
            false,
        );
    }

    pub fn get_slot0(&mut self) -> Slot0 {
        self.test_env
            .call_view_function(self.pool, "get_slot0", runtime_args! {})
    }

    pub fn get_observation(&mut self, idx: u32) -> Observation {
        self.test_env.call_view_function(
            self.pool,
            "get_observation",
            runtime_args! {"index" => idx as u64},
        )
    }

    pub fn get_max_liquidity_per_tick(&mut self) -> U128 {
        self.test_env
            .call_view_function(self.pool, "get_max_liquidity_per_tick", runtime_args! {})
    }

    pub fn mint(&mut self, recipient: Key, tick_lower: i32, tick_upper: i32, liquidity: U128) {
        self.test_env
            .approve(self.token0, wallet(), self.swap_target, U256::MAX);
        self.test_env
            .approve(self.token1, wallet(), self.swap_target, U256::MAX);
        self.test_env.call_contract(
            Some(wallet()),
            self.swap_target.into_hash().unwrap().into(),
            "mint",
            runtime_args! {
                "pool" => self.pool,
                "recipient" => recipient,
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper,
                "amount" => liquidity
            },
            true,
        );
    }

    pub fn mint_fail(&mut self, recipient: Key, tick_lower: i32, tick_upper: i32, liquidity: U128) {
        self.test_env
            .approve(self.token0, wallet(), self.swap_target, U256::MAX);
        self.test_env
            .approve(self.token1, wallet(), self.swap_target, U256::MAX);
        self.test_env.call_contract(
            Some(wallet()),
            self.swap_target.into_hash().unwrap().into(),
            "mint",
            runtime_args! {
                "pool" => self.pool,
                "recipient" => recipient,
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper,
                "amount" => liquidity
            },
            false,
        );
    }

    pub fn burn(&mut self, owner: AccountHash, tick_lower: i32, tick_upper: i32, amount: U128) {
        self.test_env.call_contract(
            Some(owner),
            self.pool.into_hash().unwrap().into(),
            "burn",
            runtime_args! {
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper,
                "amount" => amount
            },
            true,
        );
    }

    pub fn burn_fail(&mut self, tick_lower: i32, tick_upper: i32, amount: U128) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "burn",
            runtime_args! {
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper,
                "amount" => amount
            },
            false,
        );
    }

    pub fn collect(
        &mut self,
        recipient: Key,
        tick_lower: i32,
        tick_upper: i32,
        amount0_requested: U128,
        amount1_requested: U128,
    ) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "collect",
            runtime_args! {
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper,
                "amount0_requested" => amount0_requested,
                "amount1_requested" => amount1_requested,
                "recipient" => recipient
            },
            true,
        );
    }

    pub fn collect_with(
        &mut self,
        owner: AccountHash,
        recipient: Key,
        tick_lower: i32,
        tick_upper: i32,
        amount0_requested: U128,
        amount1_requested: U128,
    ) {
        self.test_env.call_contract(
            Some(owner),
            self.pool.into_hash().unwrap().into(),
            "collect",
            runtime_args! {
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper,
                "amount0_requested" => amount0_requested,
                "amount1_requested" => amount1_requested,
                "recipient" => recipient
            },
            true,
        );
    }

    pub fn collect_protocol(
        &mut self,
        recipient: Key,
        amount0_requested: U128,
        amount1_requested: U128,
    ) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "collect_protocol",
            runtime_args! {
                "amount0_requested" => amount0_requested,
                "amount1_requested" => amount1_requested,
                "recipient" => recipient
            },
            true,
        );
    }

    pub fn get_tick(&mut self, tick: i32) -> TickInfo {
        self.test_env.call_view_function(
            self.pool,
            "get_tick",
            runtime_args! {
                "tick" => tick
            },
        )
    }

    pub fn set_fee_protocol(&mut self, fee_protocol0: u8, fee_protocol1: u8) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "set_fee_protocol",
            runtime_args! {
                "fee_protocol0" => fee_protocol0,
                "fee_protocol1" => fee_protocol1,
            },
            true,
        );
    }

    pub fn set_fee_protocol_fail(&mut self, fee_protocol0: u8, fee_protocol1: u8) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "set_fee_protocol",
            runtime_args! {
                "fee_protocol0" => fee_protocol0,
                "fee_protocol1" => fee_protocol1,
            },
            false,
        );
    }

    pub fn swap_exact_0_for_1(
        &mut self,
        amount: U256,
        to: Key,
        sqrt_price_limit_x96: Option<U256>,
    ) {
        self.swap(self.token0, amount, 0.into(), to, sqrt_price_limit_x96)
    }

    pub fn swap_0_for_exact_1(
        &mut self,
        amount: U256,
        to: Key,
        sqrt_price_limit_x96: Option<U256>,
    ) {
        self.swap(self.token0, 0.into(), amount, to, sqrt_price_limit_x96)
    }

    pub fn swap_exact_1_for_0(
        &mut self,
        amount: U256,
        to: Key,
        sqrt_price_limit_x96: Option<U256>,
    ) {
        self.swap(self.token1, amount, 0.into(), to, sqrt_price_limit_x96)
    }

    pub fn swap_1_for_exact_0(
        &mut self,
        amount: U256,
        to: Key,
        sqrt_price_limit_x96: Option<U256>,
    ) {
        self.swap(self.token1, 0.into(), amount, to, sqrt_price_limit_x96)
    }

    pub fn swap(
        &mut self,
        input_token: Key,
        amount_in: U256,
        amount_out: U256,
        to: Key,
        sqrt_price_limit_x96: Option<U256>,
    ) {
        let exact_input = amount_out.eq(&0.into());
        let sqrt_price_limit_x96_ = if let Some(..) = sqrt_price_limit_x96 {
            sqrt_price_limit_x96.unwrap()
        } else if input_token == self.token0 {
            min_sqrt_ratio().add(1)
        } else {
            max_sqrt_ratio().sub(1)
        };

        let (method, param_name) = if input_token == self.token0 {
            if exact_input {
                ("swap_exact_0_for_1", "amount0_in")
            } else {
                ("swap_0_for_exact_1", "amount1_out")
            }
        } else if exact_input {
            ("swap_exact_1_for_0", "amount1_in")
        } else {
            ("swap_1_for_exact_0", "amount0_out")
        };
        println!("method {:?}", method);
        let mut args = runtime_args! {
            "pool" => self.pool,
            "recipient" => to,
            "sqrt_price_limit_x96" => sqrt_price_limit_x96_,
        };
        args.insert(param_name, if exact_input { amount_in } else { amount_out })
            .unwrap();
        self.test_env.call_contract(
            Some(wallet()),
            self.swap_target.into_hash().unwrap().into(),
            method,
            args,
            true,
        );
    }

    pub fn get_protocol_fees(&mut self) -> ProtocolFees {
        self.test_env
            .call_view_function(self.pool, "get_protocol_fees", runtime_args! {})
    }

    pub fn get_position(&mut self, owner: Key, tick_lower: i32, tick_upper: i32) -> PositionInfo {
        let position_key: String = self.test_env.call_view_function(
            self.pool,
            "get_position_key",
            runtime_args! {
                "owner" => owner,
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper
            },
        );
        self.test_env.call_view_function(
            self.pool,
            "get_position",
            runtime_args! {
                "position_key" => position_key,
            },
        )
    }

    pub fn get_liquidity(&mut self) -> U128 {
        self.test_env
            .call_view_function(self.pool, "get_liquidity", runtime_args! {})
    }

    pub fn get_fee_growth_global0_x128(&mut self) -> U256 {
        self.test_env
            .call_view_function(self.pool, "get_fee_growth_global0_x128", runtime_args! {})
    }

    pub fn get_fee_growth_global1_x128(&mut self) -> U256 {
        self.test_env
            .call_view_function(self.pool, "get_fee_growth_global1_x128", runtime_args! {})
    }

    pub fn initialize_at_zero_tick(&mut self) {
        self.initialize_pool_price(encode_price_sqrt(1, 1));
        self.mint(
            wallet().into(),
            self.min_tick,
            self.max_tick,
            initialize_liquidity_amount().as_u128().into(),
        );
    }

    pub fn observe(&mut self, seconds_agos: Vec<u32>) -> (Vec<i64>, Vec<U256>) {
        self.test_env.call_view_function(
            self.pool,
            "observe",
            runtime_args! {
                "seconds_agos" => seconds_agos
            },
        )
    }

    pub fn snapshot_cumulatives_inside(&mut self, tick_lower: i32, tick_upper: i32) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "snapshot_cumulatives_inside",
            runtime_args! {
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper
            },
            true,
        );
    }

    pub fn snapshot_cumulatives_inside_fail(&mut self, tick_lower: i32, tick_upper: i32) {
        self.test_env.call_contract(
            Some(wallet()),
            self.pool.into_hash().unwrap().into(),
            "snapshot_cumulatives_inside",
            runtime_args! {
                "tick_lower" => tick_lower,
                "tick_upper" => tick_upper
            },
            false,
        );
    }

    pub fn swap_to_sqrt_price(&mut self, input_token: Key, target_price: U256, to: Key) {
        let method = if input_token == self.token0 {
            "swap_to_lower_sqrt_price"
        } else {
            "swap_to_higher_sqrt_price"
        };
        let args = runtime_args! {
            "pool" => self.pool,
            "recipient" => to,
            "sqrt_price_x96" => target_price,
        };

        self.test_env.call_contract(
            Some(wallet()),
            self.swap_target.into_hash().unwrap().into(),
            method,
            args,
            true,
        );
    }

    pub fn swap_to_lower_price(&mut self, sqrt_price_x96: U256, to: Key) {
        self.swap_to_sqrt_price(self.token0, sqrt_price_x96, to)
    }

    pub fn swap_to_higher_price(&mut self, sqrt_price_x96: U256, to: Key) {
        self.swap_to_sqrt_price(self.token1, sqrt_price_x96, to)
    }

    pub fn flash(
        &mut self,
        amount0: U256,
        amount1: U256,
        to: Key,
        pay0: Option<U256>,
        pay1: Option<U256>,
    ) {
        let fee = self.fee_amount;
        let pay0 = if let Some(..) = pay0 {
            pay0.unwrap()
        } else {
            amount0
                .mul(fee)
                .add(1_000_000 - 1)
                .div(1_000_000)
                .add(amount0)
        };

        let pay1 = if let Some(..) = pay1 {
            pay1.unwrap()
        } else {
            amount1
                .mul(fee)
                .add(1_000_000 - 1)
                .div(1_000_000)
                .add(amount1)
        };
        self.test_env.call_contract(
            Some(wallet()),
            self.swap_target.into_hash().unwrap().into(),
            "flash",
            runtime_args! {
                "pool" => self.pool,
                "recipient" => to,
                "amount0" => amount0,
                "amount1" => amount1,
                "pay0" => pay0,
                "pay1" => pay1,
            },
            true,
        );
    }
}
