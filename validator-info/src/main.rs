#![no_main]

use validator_info::{deploy_validator_info_contract, upgrade_validator_info_contract};

#[no_mangle]
fn call() {
    deploy_validator_info_contract(String::from("validator-info"));
    upgrade_validator_info_contract(String::from("validator-info"));
}
