use super::ContractError;
use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, Key, URef};

pub const URLS_DICT: &str = "account-info-urls";

pub struct Urls {
    dict_uref: URef,
}

impl Urls {
    pub fn new() -> Urls {
        let dict_key: Key = runtime::get_key(URLS_DICT).unwrap_or_revert();
        let dict_uref: &URef = dict_key.as_uref().unwrap_or_revert();
        Urls {
            dict_uref: *dict_uref,
        }
    }

    pub fn set(&self, address: &AccountHash, url: &str) {
        if !is_valid_url(url) {
            runtime::revert(ContractError::BadUrlFormat);
        }
        storage::dictionary_put(self.dict_uref, &address.to_string(), url);
    }

    pub fn delete(&self, address: &AccountHash) {
        storage::dictionary_put(self.dict_uref, &address.to_string(), "");
    }

    pub fn get(&self, address: &AccountHash) -> Option<String> {
        storage::dictionary_get(self.dict_uref, &address.to_string()).unwrap_or_revert()
    }
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}
