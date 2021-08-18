use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, Key, URef};

use crate::utils;

use super::ContractError;

pub const ADMINS_DICT: &str = "admins";
pub const ADMINS_COUNT: &str = "admins_count";
pub const ADMIN_ACTIVE: bool = true;
pub const ADMIN_DISABLED: bool = false;

pub struct Admins {
    dict_uref: URef,
}

impl Admins {
    pub fn new() -> Admins {
        let dict_key: Key = runtime::get_key(ADMINS_DICT).unwrap_or_revert();
        let dict_uref: &URef = dict_key.as_uref().unwrap_or_revert();
        Admins {
            dict_uref: *dict_uref,
        }
    }

    pub fn is_admin(&self, account: &AccountHash) -> bool {
        let result: Option<bool> =
            storage::dictionary_get(self.dict_uref, &account.to_string()).unwrap_or_revert();
        match result {
            Some(value) => value == ADMIN_ACTIVE,
            None => false,
        }
    }

    pub fn add(&self, account: &AccountHash) {
        if self.is_admin(account) {
            runtime::revert(ContractError::AdminExists);
        } else {
            storage::dictionary_put(self.dict_uref, &account.to_string(), ADMIN_ACTIVE);

            // Increment the admin count.
            let admins_count: u32 = utils::get_key(ADMINS_COUNT).unwrap_or_revert();
            utils::set_key(ADMINS_COUNT, admins_count + 1);
        }
    }

    pub fn disable(&self, account: &AccountHash) {
        // Make sure the last admin can't remove itself.
        let admins_count: u32 = utils::get_key(ADMINS_COUNT).unwrap_or_revert();
        if admins_count == 1 {
            runtime::revert(ContractError::AdminCountToLow);
        }

        // Decrement admins count.
        utils::set_key(ADMINS_COUNT, admins_count - 1);

        if self.is_admin(account) {
            storage::dictionary_put(self.dict_uref, &account.to_string(), ADMIN_DISABLED);
        } else {
            runtime::revert(ContractError::AdminDoesntExist);
        }
    }

    pub fn assert_caller_is_admin(&self) {
        if !self.is_admin(&utils::get_caller()) {
            runtime::revert(ContractError::PermissionDenied);
        }
    }
}
