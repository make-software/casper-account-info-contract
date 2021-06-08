extern crate alloc;

use contract::{
    contract_api::{runtime, storage, runtime::revert},
    unwrap_or_revert::UnwrapOrRevert,
};
use std::convert::TryInto;

use types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractPackageHash, ApiError, Key,
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_URL: Regex = Regex::new(r"^https?://.*$").unwrap();
}

#[derive(Debug)]
pub enum ContractError {
    NotFound = 1,
    BadUrlFormat = 2,
    NotAllowed = 3,
}

impl Into<ApiError> for ContractError{
    fn into(self) -> ApiError{
        ApiError::User(self as u16)
    }
}

/*
Register your entrypoints (contract methods) here.

Every entrypoint consists of:
- name of the function entry point it tries to load
- list of params
- return type
- access type
- type

For more, see: https://docs.rs/casper-types/1.2.0/casper_types/contracts/struct.EntryPoint.html
*/
pub fn get_entry_points(contract_package_hash: &ContractPackageHash) -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    let deployer_group = storage::create_contract_user_group(
        *contract_package_hash,
        "admin",
        1,
        alloc::collections::BTreeSet::default(),
    )
    .unwrap_or_revert();
    runtime::put_key("admin_access", Key::URef(deployer_group[0]));
    entry_points.add_entry_point(EntryPoint::new(
        "set_url",
        vec![Parameter::new("url", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_url",
        vec![Parameter::new("public_key", CLType::PublicKey)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "delete_url",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "set_url_for_validator",
        vec![
            Parameter::new("public_hash", CLType::String),
            Parameter::new("url", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::groups(&["admin"]),
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "delete_url_for_validator",
        vec![Parameter::new("public_hash", CLType::String)],
        CLType::Unit,
        EntryPointAccess::groups(&["admin"]),
        EntryPointType::Contract,
    ));
    entry_points
}

/*
Deploy or upgrade of the contract by it's name.
*/
pub fn install_or_upgrade_contract(name: String) {
    let contract_package_hash : ContractPackageHash = match runtime::get_key(&format!("{}-package-hash", name)){
        Some(contract_package_hash)=>{
            contract_package_hash
            .into_hash()
            .unwrap_or_revert()
            .into()
        },None=>{
            let (contract_package_hash, access_token) = storage::create_contract_package_at_hash();
            runtime::put_key(&format!("{}-package-hash", name), contract_package_hash.into());
            runtime::put_key(&format!("{}-access-uref", name), access_token.into());
            contract_package_hash
        }
    };
    let entry_points = get_entry_points(&contract_package_hash);
    let (contract_hash, _) =
    storage::add_contract_version(contract_package_hash, entry_points, Default::default());
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
fn set_url() {
    let url: String = runtime::get_named_arg("url");

    if !RE_URL.is_match(&url) {
        revert(ContractError::BadUrlFormat)
    }
    set_key(&get_caller_name(), url);
}

/*
The method will check the contract’s storage scope for a key named the value of public_key.
If none is found, a NotFound error is thrown. If one is found, the value of the associated URef is returned (ie. the stored URL belonging to the Public Hash).
*/
#[no_mangle]
fn get_url() {
    get_key::<String>(&runtime::get_named_arg::<String>("public_key"));
}

/*
The method deletes from the contract’s storage scope on the blockchain a key that is named for the Public Hash of the caller.
*/
#[no_mangle]
fn delete_url() {
    runtime::remove_key(&get_caller_name());
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
fn set_url_for_validator() {
    let url: String = runtime::get_named_arg("url");

    if !RE_URL.is_match(&url) {
        revert(ContractError::BadUrlFormat)
    }
    set_key(&runtime::get_named_arg::<String>("public_hash"), url);
}

/*
The method can only be called by a caller who has admin rights to the contract (ie. is in the list of privileged public hashes).
This should be checked as a first step when the method is invoked.
If the caller should not have access, a NotAllowed error should be thrown.

The method deletes from the contract’s storage scope on the blockchain a key that is named for the public_hash input parameter.
*/
#[no_mangle]
fn delete_url_for_validator() {
    runtime::remove_key(&runtime::get_named_arg::<String>("public_hash"));
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
