#![no_main]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::convert::TryInto;

use casperlabs_contract_macro::{casperlabs_constructor, casperlabs_contract, casperlabs_method};
use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::result::Result;
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,
};

#[derive(Debug)]
pub enum ContractError {
    NotFound,
    BadUrlFormat,
    NotAllowed,
}

lazy_static! {
    static ref RE_URL: Regex = Regex::new(r"^https?://.*$").unwrap();
}

#[casperlabs_contract]
mod CVIC {

    // 
    // Initialization
    // 

    #[casperlabs_constructor]
    fn constructor(admins: Vec<String>) {
        set_key("_admins", admins);
    }

    // 
    // Methods
    // 

    /*
    Implement an EntryPoint (method) setUrl that takes one parameter url.

    the method can perform basic checks on the input parameters url.
    Is it not empty, is it correctly typed (I believe this will be checked by the runtime), does it match a URL regex rule.
    Error can be thrown. We should have explicit error codes/messages if we have multiple error possibilities

    the method persists to the contract context on the blockchain, a key that is named after the caller of the contract, and a value that is a URef to a String that contains the url
    */
    #[casperlabs_method]
    fn setUrl(url: String) {
        if !RE_URL.is_match(&url) {
            panic!(ContractError::BadUrlFormat)
        }
        set_key(&get_caller_name(), url);
    }

    /*
    Implement an EntryPoint (method) getUrl that takes one parameter public_key.

    the method will check the contract’s storage scope for a key named the value of public_key. If none is found, a NotFound error is thrown.
    If one is found, the value of the associated URef is returned (ie. the stored URL belonging to the Public Hash)
    */
    #[casperlabs_method]
    fn getUrl(public_key: String) -> String {
        match get_key(&public_key) {
            Some(key) => key,
            None => panic!(ContractError::NotFound),
        }
    }

    /*
    Implement an EntryPoint (method) deleteUrl that takes no parameter.

    the method deletes from the contract’s storage scope on the blockchain a key that is named for the Public Hash of the caller
    */
    #[casperlabs_method]
    fn deleteUrl() {
        del_key(&get_caller_name());
    }

    /*
    Implement an EntryPoint (method) setUrlForValidator that takes two parameters: public_hash and url.

    The method can only be called by a caller who has admin rights to the contract (ie. is in the list of privileged public hashes).
    This should be checked as a first step when the method is invoked. If the caller should not have access, a NotAllowed error should be thrown.

    the method can perform basic checks on the input parameters url.

    Is it not empty, is it correctly typed (I believe this will be checked by the runtime), does it match a URL regex rule.
    Error can be thrown. We should have explicit error codes/messages if we have multiple error possibilities.
    Similarly checks can be performed on the public_hash input parameter.

    the method persists to the contract context on the blockchain, a key that is named after the public_hash and a value that is a URef to a String that contains the url
    */
    #[casperlabs_method]
    fn setUrlForValidator(public_hash: String, url: String) {
        assertAdminRights();

        if !RE_URL.is_match(&url) {
            panic!(ContractError::BadUrlFormat)
        }
        set_key(&public_hash, url);
    }

    /*
    Implement an EntryPoint (method) deleteUrlForValidator that takes one parameter public_hash.

    The method can only be called by a caller who has admin rights to the contract (ie. is in the list of privileged public hashes).
    This should be checked as a first step when the method is invoked. If the caller should not have access, a NotAllowed error should be thrown.

    the method deletes from the contract’s storage scope on the blockchain a key that is named for the public_hash input parameter
    */
    #[casperlabs_method]
    fn deleteUrlForValidator(public_hash: String) {
        assertAdminRights();

        del_key(&public_hash);
    }

    // 
    // Helpers
    // 

    fn assertAdminRights() {
        if !hasAdminRights(&get_caller_name()) {
            panic!(ContractError::NotAllowed)
        }
    }

    fn hasAdminRights(caller: &str) -> bool {
        // TODO: get_privileged_hashes
        let vec: Vec<String> = match get_key(&String::from("_admins")) {
            Some(key) => key,
            None => panic!(ContractError::NotFound),
        };
        vec.iter().any(|i| i == caller)
    }

    fn get_privileged_hashes() -> Vec<&'static str> {
        todo!()
    }
}

fn get_caller_name() -> String {
    let caller = runtime::get_caller();
    let caller_bytes = caller.as_bytes().to_vec();
    match String::from_utf8(caller_bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
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
