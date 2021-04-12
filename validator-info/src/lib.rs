use std::convert::TryInto;
use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert
};

use types::{CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, bytesrepr::{FromBytes, ToBytes}};

#[no_mangle]
fn setUrl() {
    let url: String = runtime::get_named_arg("url");
    set_key(&get_caller_name(), url);
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
