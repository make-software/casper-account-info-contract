use std::convert::TryInto;
use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert
};

use types::{CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, bytesrepr::{FromBytes, ToBytes}};
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

#[no_mangle]
fn setUrl() {
    let url: String = runtime::get_named_arg("url");

    if !RE_URL.is_match(&url) {
        panic!(ContractError::BadUrlFormat)
    }
    set_key(&get_caller_name(), url);
}

#[no_mangle]
fn getUrl() {
    let public_key: String = runtime::get_named_arg("public_key");

    get_key::<String>(&get_caller_name());
}

#[no_mangle]
fn deleteUrl() {
    del_key(&get_caller_name());
}

#[no_mangle]
fn setUrlForValidator(public_hash: String, url: String) {
    assertAdminRights();

    let public_hash: String = runtime::get_named_arg("public_hash");
    let url: String = runtime::get_named_arg("url");
    
    if !RE_URL.is_match(&url) {
        panic!(ContractError::BadUrlFormat)
    }
    set_key(&public_hash, url);
}
    
#[no_mangle]
fn deleteUrlForValidator(public_hash: String) {
    assertAdminRights();

    let public_hash: String = runtime::get_named_arg("public_hash");

    del_key(&public_hash);
}

fn assertAdminRights() {
    if !hasAdminRights(&get_caller_name()) {
        panic!(ContractError::NotAllowed)
    }
}

fn hasAdminRights(caller: &str) -> bool {
    todo!()
    // TODO: get_privileged_hashes
}

fn get_privileged_hashes() -> Vec<&'static str> {
    todo!()
}

pub fn deploy_validator_info_contract(name: String) {
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
    let (contract_hash, _) = storage::new_contract(
        entry_points, 
        None, 
        Some(format!("{}-package-hash", name)), 
        Some(format!("{}-access-uref", name))
    );
    runtime::put_key(&name, contract_hash.into());
}

fn get_caller_name() -> String {
    runtime::get_caller().to_string()
}

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

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

fn del_key(name: &str) {
    runtime::remove_key(name)
}
