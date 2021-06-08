#![no_main]

#[no_mangle]
fn call() {
    validator_info::install_or_upgrade_contract(String::from("validator-info"));
}
