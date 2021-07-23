#![no_main]

use contract::{contract_api::{account, runtime, system}, unwrap_or_revert::UnwrapOrRevert};
use types::{ContractPackageHash, U512, RuntimeArgs, runtime_args};

#[no_mangle]
fn call() {
    let url: String = runtime::get_named_arg("url");
    let amount: Option<U512> = runtime::get_named_arg("amount");
    let contract_address: ContractPackageHash = runtime::get_named_arg("contract_address");

    let purse = amount.map(|amount| {
        let purse = system::create_purse();
        system::transfer_from_purse_to_purse(account::get_main_purse(), purse, amount, None)
            .unwrap_or_revert();
        purse
    });
    
    let _: () = runtime::call_versioned_contract(contract_address, None, "set_url", runtime_args!{
        "url" => url,
        "purse" => purse
    });
}