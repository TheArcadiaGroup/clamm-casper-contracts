#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use alloc::vec::Vec;
use alloc::{string::String, vec};
use casper_contract::{
    self,
    contract_api::{runtime, storage},
};
use casper_types::bytesrepr::FromBytes;
use casper_types::U128;
use casper_types::{
    bytesrepr::ToBytes, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Key, Parameter, RuntimeArgs, U256,
};
use types::{Observation, Slot0};

const RESULT_KEY: &str = "result";
const TEST_SESSION: &str = "test_session";

fn store_result<T: CLTyped + ToBytes>(result: T) {
    match runtime::get_key(RESULT_KEY) {
        Some(Key::URef(uref)) => storage::write(uref, result),
        Some(_) => unreachable!(),
        None => {
            let new_uref = storage::new_uref(result);
            runtime::put_key(RESULT_KEY, new_uref.into());
        }
    }
}

fn store_result_with_name<T: CLTyped + ToBytes>(result: T, name: String) {
    match runtime::get_key(&name) {
        Some(Key::URef(uref)) => storage::write(uref, result),
        Some(_) => unreachable!(),
        None => {
            let new_uref = storage::new_uref(result);
            runtime::put_key(&name, new_uref.into());
        }
    }
}

#[no_mangle]
extern "C" fn get_u256_result() {
    let package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let func: String = runtime::get_named_arg("func");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;
    let b = if func == *"get_last_price" {
        let (x, _): (U256, u64) = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );
        x
    } else {
        let x: U256 = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );
        x
    };

    store_result(b);
}

#[no_mangle]
extern "C" fn get_u128_result() {
    let package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let func: String = runtime::get_named_arg("func");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;
    let b = if func == *"get_last_price" {
        let (x, _): (U128, u64) = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );
        x
    } else {
        let x: U128 = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );
        x
    };

    store_result(b);
}

#[no_mangle]
extern "C" fn get_multi_u256_result() {
    let package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let func: String = runtime::get_named_arg("func");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;
    let b: Vec<U256> = runtime::call_versioned_contract(
        package_hash.into_hash().unwrap().into(),
        None,
        &func,
        args,
    );

    store_result(b);
}

#[no_mangle]
extern "C" fn get_multi_u128_result() {
    let package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let func: String = runtime::get_named_arg("func");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;
    let b: Vec<U128> = runtime::call_versioned_contract(
        package_hash.into_hash().unwrap().into(),
        None,
        &func,
        args,
    );

    store_result(b);
}

#[no_mangle]
extern "C" fn get_two_u256_result() {
    let package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let func: String = runtime::get_named_arg("func");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;
    let (a, _b): (U256, U256) = runtime::call_versioned_contract(
        package_hash.into_hash().unwrap().into(),
        None,
        &func,
        args,
    );

    store_result(a);
}

#[no_mangle]
extern "C" fn get_any_data() {
    let package_hash: Key = runtime::get_named_arg("contract_package_hash");
    let func: String = runtime::get_named_arg("func");
    let return_type_name: String = runtime::get_named_arg("return_type_name");
    let args: String = runtime::get_named_arg("args");
    let args = hex::decode(args).unwrap();
    let args = RuntimeArgs::from_bytes(&args).unwrap().0;

    if return_type_name == "u64" {
        let b: u64 = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );

        store_result_with_name(b, return_type_name);
    } else if return_type_name == "U256" {
        let b: U256 = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );

        store_result_with_name(b, return_type_name);
    } else if return_type_name == "U128" {
        let b: U128 = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );

        store_result_with_name(b, return_type_name);
    } else if return_type_name == "Key" {
        let b: Key = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );

        store_result_with_name(b, return_type_name);
    } else if return_type_name == "Slot0" {
        let b: Slot0 = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );

        store_result_with_name(b, return_type_name);
    } else if return_type_name == "Observation" {
        let b: Observation = runtime::call_versioned_contract(
            package_hash.into_hash().unwrap().into(),
            None,
            &func,
            args,
        );

        store_result_with_name(b, return_type_name);
    }
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();
    let get_u256_result = EntryPoint::new(
        String::from("get_u256_result"),
        vec![
            Parameter::new("contract_package_hash", Key::cl_type()),
            Parameter::new("func", String::cl_type()),
            Parameter::new("args", CLType::String),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    let get_u128_result = EntryPoint::new(
        String::from("get_u128_result"),
        vec![
            Parameter::new("contract_package_hash", Key::cl_type()),
            Parameter::new("func", String::cl_type()),
            Parameter::new("args", CLType::String),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_multi_u256_result = EntryPoint::new(
        String::from("get_multi_u256_result"),
        vec![
            Parameter::new("contract_package_hash", Key::cl_type()),
            Parameter::new("func", String::cl_type()),
            Parameter::new("args", CLType::String),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_multi_u128_result = EntryPoint::new(
        String::from("get_multi_u128_result"),
        vec![
            Parameter::new("contract_package_hash", Key::cl_type()),
            Parameter::new("func", String::cl_type()),
            Parameter::new("args", CLType::String),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_two_u256_result = EntryPoint::new(
        String::from("get_two_u256_result"),
        vec![
            Parameter::new("contract_package_hash", Key::cl_type()),
            Parameter::new("func", String::cl_type()),
            Parameter::new("args", CLType::String),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_any_data = EntryPoint::new(
        String::from("get_any_data"),
        vec![],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(get_u256_result);
    entry_points.add_entry_point(get_multi_u256_result);
    entry_points.add_entry_point(get_multi_u128_result);
    entry_points.add_entry_point(get_two_u256_result);
    entry_points.add_entry_point(get_any_data);
    entry_points.add_entry_point(get_u128_result);

    let (_contract_hash, _version) = storage::new_contract(
        entry_points,
        None,
        Some(format!("{}_package_hash", TEST_SESSION)),
        None,
    );
}
