use std::convert::TryInto;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    system::CallStackElement,
    CLTyped,
};

use crate::ContractError;

/// Get AccountHash of the caller. If the caller is StoredContract if fails.
pub fn get_caller() -> AccountHash {
    let call_stack = runtime::get_call_stack();
    let caller = call_stack.get(call_stack.len() - 2);
    element_to_account_hash(caller.unwrap_or_revert())
}

/// Get AccountHash of the deployer at the deployment stage.
pub fn self_addr() -> AccountHash {
    element_to_account_hash(runtime::get_call_stack().last().unwrap_or_revert())
}

fn element_to_account_hash(element: &CallStackElement) -> AccountHash {
    match element {
        CallStackElement::Session { account_hash } => *account_hash,
        CallStackElement::StoredSession {
            account_hash,
            contract_package_hash: _,
            contract_hash: _,
        } => *account_hash,
        CallStackElement::StoredContract {
            contract_package_hash: _,
            contract_hash: _,
        } => runtime::revert(ContractError::ReadingCallerError),
    }
}

/// Getter function from context storage.
/// Returns the previously data previously stored under `name` key,
/// or returns the default value of the type expected at the end of the call.
pub fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> Option<T> {
    runtime::get_key(name).map(|value| {
        let key = value.try_into().unwrap_or_revert();
        storage::read(key).unwrap_or_revert().unwrap_or_revert()
    })
}

/// Creates new storage key `name` and stores `value` to it.
/// In case the key `name` already exists, overwrites it with the new data.
pub fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
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
