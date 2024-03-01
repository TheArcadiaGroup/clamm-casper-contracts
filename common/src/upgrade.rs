use crate::owner::only_owner;
use alloc::{format, string::String, vec};
use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_event_standard::EVENTS_DICT;
use casper_event_standard::EVENTS_LENGTH;
use casper_event_standard::EVENTS_SCHEMA;
use casper_event_standard::{Schemas, CES_VERSION_KEY};
use casper_types::EntryPoint;
use casper_types::{
    contracts::NamedKeys, runtime_args, CLType, ContractHash, ContractPackageHash,
    EntryPointAccess, EntryPointType, EntryPoints, Key, RuntimeArgs,
};

pub fn install_contract(
    contract_name: String,
    entry_points: EntryPoints,
    named_keys: NamedKeys,
) -> (ContractHash, ContractPackageHash) {
    let (contract_package_hash, access_token) = storage::create_contract_package_at_hash();

    let (contract_hash, _version) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);

    runtime::put_key(
        &format!("{}_package_hash", contract_name),
        contract_package_hash.into(),
    );

    runtime::put_key(
        &format!("{}_package_hash_wrapped", contract_name),
        storage::new_uref(contract_package_hash).into(),
    );

    runtime::put_key(
        &format!("{}_contract_hash", contract_name),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{}_contract_hash_wrapped", contract_name),
        storage::new_uref(contract_hash).into(),
    );
    runtime::put_key(
        &format!("{}_package_access_token", contract_name),
        access_token.into(),
    );

    (contract_hash, contract_package_hash)
}

pub fn upgrade(
    contract_name: String,
    entry_points: EntryPoints,
    named_keys: NamedKeys,
    schemas: Schemas,
) {
    let re_initialize_event: bool = runtime::get_named_arg("re_initialize_event");
    if !re_initialize_event {
        upgrade_contract(contract_name, entry_points, named_keys);
    } else {
        upgrade_with_reinitialize_event(contract_name, entry_points, named_keys, schemas);
    }
}

pub fn upgrade_with_reinitialize_event(
    contract_name: String,
    entry_points: EntryPoints,
    named_keys: NamedKeys,
    new_schemas: Schemas,
) {
    let mut entry_points = entry_points;
    entry_points.add_entry_point(re_initialize_event_entrypoint());
    upgrade_contract(contract_name.clone(), entry_points, named_keys);

    let this_package_hash: Key =
        runtime::get_key(&format!("{}_package_hash", &contract_name)).unwrap();

    let _: () = runtime::call_versioned_contract(
        this_package_hash.into_hash().unwrap().into(),
        None,
        "re_initialize_event",
        runtime_args! {
            "new_schemas" => new_schemas
        },
    );
}

pub fn upgrade_contract(contract_name: String, entry_points: EntryPoints, named_keys: NamedKeys) {
    let package_hash: ContractPackageHash =
        runtime::get_key(&format!("{}_package_hash", contract_name))
            .unwrap_or_revert()
            .into_hash()
            .unwrap()
            .into();
    let old_contract_hash: ContractHash =
        runtime::get_key(&format!("{}_contract_hash", contract_name))
            .unwrap_or_revert()
            .into_hash()
            .unwrap()
            .into();

    let (contract_hash, _) = storage::add_contract_version(package_hash, entry_points, named_keys);

    runtime::put_key(
        &format!("{}_contract_hash", contract_name),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{}_contract_hash_wrapped", contract_name),
        storage::new_uref(contract_hash).into(),
    );
    storage::disable_contract_version(package_hash, old_contract_hash).unwrap_or_revert();
}

pub fn re_initialize_event_entrypoint() -> EntryPoint {
    EntryPoint::new(
        String::from("re_initialize_event"),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

#[no_mangle]
pub extern "C" fn re_initialize_event() {
    only_owner();
    runtime::remove_key(EVENTS_LENGTH);
    runtime::remove_key(EVENTS_SCHEMA);
    runtime::remove_key(EVENTS_DICT);
    runtime::remove_key(CES_VERSION_KEY);
    let new_schemas: Schemas = runtime::get_named_arg("new_schemas");
    casper_event_standard::init(new_schemas);
}
