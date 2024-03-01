#![allow(dead_code)]
extern crate alloc;
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    MINIMUM_ACCOUNT_CREATION_BALANCE, PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::U128;

use crate::gas;
use alloc::{format, string::String};
use casper_types::bytesrepr::{Bytes, ToBytes};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, contracts::NamedKeys, crypto::SecretKey,
    runtime_args, system::mint, CLTyped, ContractPackageHash, Key, PublicKey, RuntimeArgs, U256,
    U512,
};
use std::convert::TryInto;
use std::ops::Div;

pub fn wallet() -> AccountHash {
    get_any_account(1)
}

pub fn other() -> AccountHash {
    get_any_account(2)
}

pub fn owner() -> AccountHash {
    get_any_account(1)
}

pub fn alice() -> AccountHash {
    get_any_account(2)
}

pub fn bob() -> AccountHash {
    get_any_account(3)
}

pub fn eve() -> AccountHash {
    get_any_account(4)
}

pub fn get_minter_account_hash() -> AccountHash {
    *DEFAULT_ACCOUNT_ADDR
}

pub fn _get_contract_hash_key(contract_name: String) -> String {
    format!("{}_contract_hash", contract_name)
}

pub fn get_contract_package_hash_key(contract_name: String) -> String {
    format!("{}_package_hash", contract_name)
}

pub fn get_contract_package_hash_key_cep18(contract_name: String) -> String {
    format!("cep18_contract_package_{}", contract_name)
}

pub fn get_view_account() -> AccountHash {
    get_any_account(111)
}

pub fn deploy_contract(
    builder: &mut InMemoryWasmTestBuilder,
    deployer: AccountHash,
    wasm_path: &str,
    args: RuntimeArgs,
) {
    let install_request_1 =
        ExecuteRequestBuilder::standard(deployer, wasm_path, args.clone()).build();
    builder.exec(install_request_1).expect_success().commit();
    if args.get("entry_name").is_some() {
        let entry_name: String =
            String::from_bytes(&args.get("entry_name").unwrap().clone().to_bytes().unwrap())
                .unwrap()
                .0;
        gas::write_to(false, &entry_name, builder.last_exec_gas_cost());
    } else {
        gas::write_to(true, wasm_path, builder.last_exec_gas_cost());
    }
}

pub fn deploy_contract_with_testing_mode(
    builder: &mut InMemoryWasmTestBuilder,
    deployer: AccountHash,
    wasm_path: &str,
    args: RuntimeArgs,
) {
    let install_request_1 =
        ExecuteRequestBuilder::standard(deployer, wasm_path, args.clone()).build();
    builder.exec(install_request_1).expect_success().commit();
    if args.get("entry_name").is_some() {
        let entry_name: String =
            String::from_bytes(&args.get("entry_name").unwrap().clone().to_bytes().unwrap())
                .unwrap()
                .0;
        gas::write_to(false, &entry_name, builder.last_exec_gas_cost());
    } else {
        gas::write_to(true, wasm_path, builder.last_exec_gas_cost());
    }
}

pub fn get_contract_package_hash(
    builder: &InMemoryWasmTestBuilder,
    owner: AccountHash,
    contract_name: String,
) -> Key {
    let account = builder.get_account(owner).expect("should have account");

    let ret = account
        .named_keys()
        .get(&get_contract_package_hash_key(contract_name))
        .expect("should have contract package hash");
    *ret
}

pub fn get_contract_hash(
    builder: &InMemoryWasmTestBuilder,
    owner: AccountHash,
    contract_name: String,
) -> Key {
    let account = builder.get_account(owner).expect("should have account");

    let ret = account
        .named_keys()
        .get(&_get_contract_hash_key(contract_name))
        .expect("should have contract package hash");
    *ret
}

pub fn get_contract_package_hash_cep18(
    builder: &InMemoryWasmTestBuilder,
    owner: AccountHash,
    contract_name: String,
) -> Key {
    let account = builder.get_account(owner).expect("should have account");

    let ret = account
        .named_keys()
        .get(&get_contract_package_hash_key_cep18(contract_name))
        .expect("should have contract package hash");
    *ret
}

pub fn _get_named_keys(builder: &InMemoryWasmTestBuilder, owner: AccountHash) -> NamedKeys {
    let account = builder.get_account(owner).expect("should have account");
    account.named_keys().clone()
}

pub fn get_any_account(b: u8) -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes([b; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

pub fn get_account1_addr() -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes([221u8; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

pub fn get_account2_addr() -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes([212u8; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

pub fn get_test_result<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    test_session: ContractPackageHash,
) -> T {
    let contract_package = builder
        .get_contract_package(test_session)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();
    let (_version, contract_hash) = enabled_versions
        .iter()
        .rev()
        .next()
        .expect("should have latest version");

    builder.get_value(*contract_hash, "result_key")
}

pub fn get_test_result_with_name<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    test_session: ContractPackageHash,
    name: &str,
) -> T {
    let contract_package = builder
        .get_contract_package(test_session)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();
    let (_version, contract_hash) = enabled_versions
        .iter()
        .rev()
        .next()
        .expect("should have latest version");

    builder.get_value(*contract_hash, name)
}

pub fn call_and_get<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    test_session: ContractPackageHash,
    target_func_name: &str,
    test_session_func: &str,
    target_package_hash: Key,
    args: RuntimeArgs,
) -> T {
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        get_view_account(),
        test_session,
        None,
        test_session_func,
        runtime_args! {
            "func" => target_func_name.to_string(),
            "args" => hex::encode(args.to_bytes().unwrap()),
            "contract_package_hash" => target_package_hash
        },
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, test_session)
}

pub fn call_and_get_any<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    test_session: ContractPackageHash,
    target_func_name: &str,
    return_type_name: &str,
    target_package_hash: Key,
    args: RuntimeArgs,
) -> T {
    println!("here");
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        get_view_account(),
        test_session,
        None,
        "get_any_data",
        runtime_args! {
            "return_type_name" => return_type_name,
            "func" => target_func_name.to_string(),
            "args" => hex::encode(args.to_bytes().unwrap()),
            "contract_package_hash" => target_package_hash
        },
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result_with_name(builder, test_session, return_type_name)
}

pub fn exec_call(
    builder: &mut InMemoryWasmTestBuilder,
    account_hash: AccountHash,
    contract_package_hash: ContractPackageHash,
    fun_name: &str,
    args: RuntimeArgs,
    expect_success: bool,
) {
    let request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        account_hash,
        contract_package_hash,
        None,
        fun_name,
        args,
    )
    .build();
    if expect_success {
        builder.exec(request).expect_success().commit();
    } else {
        builder.exec(request).expect_failure();
    }
    gas::write_to(false, fun_name, builder.last_exec_gas_cost());
}

pub fn initialize_builder() -> InMemoryWasmTestBuilder {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

    let id: Option<u64> = None;

    let transfer_1_args = runtime_args! {
        mint::ARG_TARGET => get_account1_addr(),
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };
    let transfer_2_args = runtime_args! {
        mint::ARG_TARGET => get_account2_addr(),
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };

    let transfer_request_1 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_1_args).build();
    let transfer_request_2 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_2_args).build();

    builder.exec(transfer_request_1).expect_success().commit();
    builder.exec(transfer_request_2).expect_success().commit();

    fund_account(&mut builder, get_view_account());

    builder
}

pub fn fund_account(builder: &mut InMemoryWasmTestBuilder, account: AccountHash) {
    let id: Option<u64> = None;

    let transfer_1_args = runtime_args! {
        mint::ARG_TARGET => account,
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };
    let transfer_request_1 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_1_args).build();
    builder.exec(transfer_request_1).expect_success().commit();
}

pub fn fund_accounts(builder: &mut InMemoryWasmTestBuilder, accounts: &[AccountHash]) {
    for acc in accounts {
        fund_account(builder, *acc);
    }
}

pub fn null_key() -> Key {
    let null_hash: [u8; 32] = vec![0u8; 32].try_into().unwrap();
    Key::from(ContractPackageHash::new(null_hash))
}

pub fn pow(x: u64, y: u64) -> U256 {
    U256::from(x).pow(U256::from(y))
}

pub fn u256_to_u512(nb: U256) -> U512 {
    let mut b = [0u8; 32];
    nb.to_big_endian(&mut b);
    U512::from_big_endian(&b)
}

pub fn u512_to_u256(nb: U512) -> U256 {
    let mut b = [0u8; 64];
    nb.to_big_endian(&mut b);
    U256::from_big_endian(&b[32..64])
}

pub fn encode_price_sqrt(reserve1: u128, reserve0: u128) -> U256 {
    u512_to_u256(
        (U512::from(reserve1) * U512::from(2).pow(192.into()) / U512::from(reserve0))
            .integer_sqrt(),
    )
}

pub fn expand_to_18_decimals(x: i32) -> U256 {
    U256::from(x) * U256::from(10).pow(18.into())
}

pub fn get_min_tick(tick_spacing: i32) -> i32 {
    (-887272 / tick_spacing) * tick_spacing
}

pub fn get_max_tick(tick_spacing: i32) -> i32 {
    (887272 / tick_spacing) * tick_spacing
}

pub fn get_max_liquidity_per_tick(tick_spacing: i32) -> U128 {
    U128::from(u128::MAX).div(U128::from(
        (get_max_tick(tick_spacing) - get_min_tick(tick_spacing)) / tick_spacing + 1,
    ))
}

pub fn warp_fake_timestamp(
    builder: &mut InMemoryWasmTestBuilder,
    contract: Key,
    caller: AccountHash,
    fake_timestamp: u64,
) {
    exec_call(
        builder,
        caller,
        contract.into_hash().unwrap().into(),
        "warp_fake_timestamp",
        runtime_args! {
            "warp_timestamp" => fake_timestamp,
        },
        true,
    );
}
pub fn roll_timestamp(
    builder: &mut InMemoryWasmTestBuilder,
    contract: Key,
    caller: AccountHash,
    roll_timestamp: u64,
) {
    exec_call(
        builder,
        caller,
        contract.into_hash().unwrap().into(),
        "roll_timestamp",
        runtime_args! {
            "roll_timestamp" => roll_timestamp,
        },
        true,
    );
}

pub fn initialize_liquidity_amount() -> U256 {
    expand_to_18_decimals(2)
}

pub fn sort_tokens(token0: Key, token1: Key) -> (Key, Key) {
    if token0.into_hash().unwrap() < token1.into_hash().unwrap() {
        (token0, token1)
    } else {
        (token1, token0)
    }
}

pub fn encode_path(path: Vec<Key>, fees: Vec<u32>) -> Bytes {
    if path.len() != fees.len() + 1 {
        return vec![].into();
    }

    let mut encoded: Vec<u8> = vec![];
    for i in 0..fees.len() {
        encoded.append(&mut (path[i].into_hash().unwrap()).to_vec());
        encoded.append(&mut (fees[i].to_le_bytes()).to_vec());
    }
    encoded.append(&mut (path.last().unwrap().into_hash().unwrap()).to_vec());
    encoded.into()
}