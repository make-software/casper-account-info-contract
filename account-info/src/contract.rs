#![no_main]

#[no_mangle]
fn call() {
    account_info::install_or_upgrade_contract(String::from("account-info"));
}
