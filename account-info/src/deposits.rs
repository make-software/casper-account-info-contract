use contract::{contract_api::{runtime, storage, system}, unwrap_or_revert::UnwrapOrRevert};
use types::{Key, U512, URef, account::AccountHash};

use crate::{ContractError, utils};

pub const DEPOSITS_DICT: &str = "deposits";
pub const DEPOSIT_AMOUNT: &str = "deposit_amount";
pub const PURSE: &str = "purse";

pub struct Deposits {
    dict_uref: URef,
    purse: URef
}

impl Deposits {
    pub fn new() -> Deposits {
        let dict_key: Key = runtime::get_key(DEPOSITS_DICT).unwrap_or_revert();
        let dict_uref: &URef = dict_key.as_uref().unwrap_or_revert();
        let purse_key = runtime::get_key(PURSE).unwrap_or_revert();
        let purse = purse_key.as_uref().unwrap_or_revert();
        Deposits {
            dict_uref: *dict_uref,
            purse: *purse
        }
    }

    pub fn deposit_if_needed(&self, account: &AccountHash, purse: Option<URef>) {
        let deposit = self.get_deposit(account);
        if deposit.is_zero() {
            if purse.is_none() {
                runtime::revert(ContractError::PurseIsNone);
            } else {
                let purse = purse.unwrap_or_revert();
                let balance = system::get_purse_balance(purse).unwrap_or_revert();
                let deposit_amount: U512 = utils::get_key(DEPOSIT_AMOUNT);
                if balance != deposit_amount {
                    runtime::revert(ContractError::IncorrectDepositAmount);
                } else {
                    system::transfer_from_purse_to_purse(purse, self.purse, deposit_amount, None)
                        .unwrap_or_revert();
                    self.set_deposit(&account, deposit_amount);
                }
            }
        }
    }

    pub fn withdraw(&self, account: &AccountHash) {
        let deposit = self.get_deposit(account);
        if deposit.is_zero() {
            runtime::revert(ContractError::NoDeposit);
        } else {
            system::transfer_from_purse_to_account(self.purse, *account, deposit, None)
                .unwrap_or_revert();
            self.set_deposit(account, U512::zero());
        }
    }

    pub fn set_deposit_amount(&self, amount: U512) {
        utils::set_key(DEPOSIT_AMOUNT, amount);
    }

    fn get_deposit(&self, account: &AccountHash) -> U512 {
        storage::dictionary_get(self.dict_uref, &account.to_string())
            .unwrap_or_default()
            .unwrap_or_default()
    }

    fn set_deposit(&self, account: &AccountHash, amount: U512) {
        storage::dictionary_put(self.dict_uref, &account.to_string(), amount);
    }
}
