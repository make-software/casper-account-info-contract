use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use std::convert::TryInto;

use types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractPackageHash,
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_URL: Regex = Regex::new(r"^https?://.*$").unwrap();
}

#[derive(Debug)]
pub enum ContractError {
    NotFound,
    BadUrlFormat,
    NotAllowed,
}

/*
Register your entrypoints (contract methods) here.

Every entrypoint consists of:
- name of the function entry point it tries to load
- list of params
- return type
- access type
- type

For more, see: https://docs.rs/casper-types/0.1.0/casper_types/contracts/struct.EntryPoint.html
*/
pub fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        String::from("setUrl"),
        vec![Parameter::new("url", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("getUrl"),
        vec![Parameter::new("public_key", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("deleteUrl"),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("setUrlForValidator"),
        vec![
            Parameter::new("public_hash", CLType::String),
            Parameter::new("url", CLType::String),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("deleteUrlForValidator"),
        vec![Parameter::new("public_hash", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}

/*
Deploy of the contract by it's name.

See: https://docs.casperlabs.io/en/latest/dapp-dev-guide/deploying-contracts.html
*/
pub fn deploy_validator_info_contract(name: String) {
    let entry_points = get_entry_points();
    let (contract_hash, _) = storage::new_contract(
        entry_points,
        None,
        Some(format!("{}-package-hash", name)),
        Some(format!("{}-access-uref", name)),
    );
    runtime::put_key(&name, contract_hash.into());
}

/*
Upgrade of the contract by it's name.
*/
pub fn upgrade_validator_info_contract(name: String) {
    let entry_points = get_entry_points();
    let package_hash: ContractPackageHash = runtime::get_key(&format!("{}-package-hash", name))
        .unwrap()
        .into_hash()
        .unwrap()
        .into();
    let (contract_hash, _) = storage::add_contract_version(
        package_hash,
        entry_points,
        Default::default(),
    );
    runtime::put_key(&name, contract_hash.into());
}

/*
The method can perform basic checks on the input parameters url.
Is it not empty, is it correctly typed (I believe this will be checked by the runtime), does it match a URL regex rule.
Error can be thrown. We should have explicit error codes/messages if we have multiple error possibilities

The method persists to the contract context on the blockchain, a key that is named after the caller of the contract,
and a value that is a URef to a String that contains the url.
*/
#[no_mangle]
fn setUrl() {
    let url: String = runtime::get_named_arg("url");

    if !RE_URL.is_match(&url) {
        panic!(ContractError::BadUrlFormat)
    }
    set_key(&get_caller_name(), url);
}

/*
The method will check the contract’s storage scope for a key named the value of public_key.
If none is found, a NotFound error is thrown. If one is found, the value of the associated URef is returned (ie. the stored URL belonging to the Public Hash).
*/
#[no_mangle]
fn getUrl() {
    let public_key: String = runtime::get_named_arg("public_key");

    get_key::<String>(&public_key);
}

/*
The method deletes from the contract’s storage scope on the blockchain a key that is named for the Public Hash of the caller.
*/
#[no_mangle]
fn deleteUrl() {
    del_key(&get_caller_name());
}

/*
The method can only be called by a caller who has admin rights to the contract (ie. is in the list of privileged public hashes).
This should be checked as a first step when the method is invoked. If the caller should not have access, a NotAllowed error should be thrown.

the method can perform basic checks on the input parameters url.
Is it not empty, is it correctly typed (I believe this will be checked by the runtime), does it match a URL regex rule.
Error can be thrown. We should have explicit error codes/messages if we have multiple error possibilities.
Similarly checks can be performed on the public_hash input parameter.

The method persists to the contract context on the blockchain, a key that is named after the public_hash and a value that is a URef to a String that contains the url.
*/
#[no_mangle]
fn setUrlForValidator() {
    assert_admin_rights();

    let public_hash: String = runtime::get_named_arg("public_hash");
    let url: String = runtime::get_named_arg("url");

    if !RE_URL.is_match(&url) {
        panic!(ContractError::BadUrlFormat)
    }
    set_key(&public_hash, url);
}

/*
The method can only be called by a caller who has admin rights to the contract (ie. is in the list of privileged public hashes).
This should be checked as a first step when the method is invoked.
If the caller should not have access, a NotAllowed error should be thrown.

The method deletes from the contract’s storage scope on the blockchain a key that is named for the public_hash input parameter.
*/
#[no_mangle]
fn deleteUrlForValidator() {
    assert_admin_rights();

    let public_hash: String = runtime::get_named_arg("public_hash");

    del_key(&public_hash);
}

// Helper functions, reused between entrypoints (nice to have in other contracts).

fn assert_admin_rights() {
    if !has_admin_rights(&get_caller_name()) {
        panic!(ContractError::NotAllowed)
    }
}

fn has_admin_rights(caller: &str) -> bool {
    get_privileged_hashes().iter().any(|&x| x == caller)
}

fn get_privileged_hashes() -> Vec<&'static str> {
    // TODO: get_key::<Vec<&'static str>>(&"_admins")
    [].to_vec()
}

fn get_caller_name() -> String {
    runtime::get_caller().to_string()
}

/*
Get key by it's name.
Automatically converts value to a type T known during compilation time.
*/
fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

/*
Set given key-value.
*/
fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

/*
Deletes a key by it's name.

Added for clarity and unified API.
*/
fn del_key(name: &str) {
    runtime::remove_key(name)
}
