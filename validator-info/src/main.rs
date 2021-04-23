#![no_main]

use validator_info::{deploy_validator_info_contract, upgrade};

#[no_mangle]
fn call() {
    deploy_validator_info_contract(String::from("validator-info"));
    upgrade(String::from("validator-info"));
}

