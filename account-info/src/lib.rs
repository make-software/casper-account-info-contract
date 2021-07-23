extern crate alloc;

use contract::{contract_api::{account, runtime, storage, system}, unwrap_or_revert::UnwrapOrRevert};
use types::{CLTyped, RuntimeArgs, U512, URef, account::AccountHash, contracts::NamedKeys, runtime_args};

use types::{
    contracts::ContractPackageHash,
    ApiError, CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Parameter,
};

mod admins;
mod urls;
mod utils;
mod deposits;

use admins::Admins;
use urls::Urls;
use deposits::Deposits;

#[derive(Debug)]
pub enum ContractError {
    NotFound = 1,
    BadUrlFormat = 2,
    NotAllowed = 3,
    AdminCountToLow = 4,
    PermissionDenied = 5,
    AdminExists = 6,
    AdminDoesntExist = 7,
    CallerIsNotAccount = 8,
    IncorrectDepositAmount = 9,
    PurseIsNone = 10,
    NoDeposit = 11
}

impl From<ContractError> for ApiError {
    fn from(err: ContractError) -> ApiError {
        ApiError::User(err as u16)
    }
}

/*
Register your entrypoints (contract methods) here.

Every entrypoint consists of:
- name of the function entry point it tries to load
- list of params: Vec<Parameter>
- return type: CLType
- access type: Public or group limited
- contract context type: Contract or Session(callers account)

For more, see: https://docs.rs/casper-types/1.2.0/casper_types/contracts/struct.EntryPoint.html
*/

/// Returns the list of the entry points in the contract with added group security.
pub fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "set_url",
        vec![Parameter::new("url", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "call_set_url",
        vec![
            Parameter::new("url", CLType::String),
            Parameter::new("purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_url",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "delete_url",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "set_url_for_account",
        vec![
            Parameter::new("account", AccountHash::cl_type()),
            Parameter::new("url", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "delete_url_for_account",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "add_admin",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "disable_admin",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "set_deposit_amount",
        vec![Parameter::new("amount", CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}

/// Deployer/upgrader function. Tries to retrieve any data presumably stored earlier
/// in the context associated to to `name`. If there is data, proceeds with that,
/// otherwise creates a new contract.
pub fn install_or_upgrade_contract(name: String) {
    let mut named_keys: NamedKeys = Default::default();
    let contract_package_hash: ContractPackageHash =
        match runtime::get_key(&format!("{}-package", name)) {
            Some(contract_package_hash) => {
                contract_package_hash.into_hash().unwrap_or_revert().into()
            }
            None => {
                let (contract_package_hash, access_token) =
                    storage::create_contract_package_at_hash();
                runtime::put_key(
                    &format!("{}-package", name),
                    contract_package_hash.into(),
                );
                runtime::put_key(
                    &format!("{}-package-access-uref", name),
                    access_token.into()
                );
                runtime::put_key(
                    &format!("{}-package-hash", name),
                    storage::new_uref(contract_package_hash).into()
                );

                // Add deployer as the first admin.
                let admin = utils::get_caller();
                let admins_dict = storage::new_dictionary(admins::ADMINS_DICT).unwrap_or_revert();
                storage::dictionary_put(
                    admins_dict, 
                    &admin.to_string(), 
                    admins::ADMIN_ACTIVE
                );
                named_keys.insert(admins::ADMINS_DICT.to_string(), admins_dict.into());
                named_keys.insert(admins::ADMINS_COUNT.to_string(), storage::new_uref(1u32).into());
                
                // Add empty dictionary for urls.
                let urls_dict = storage::new_dictionary(urls::URLS_DICT).unwrap_or_revert();
                named_keys.insert(urls::URLS_DICT.to_string(), urls_dict.into());
                
                // Add empty dictionary for deposits.
                let urls_dict = storage::new_dictionary(deposits::DEPOSITS_DICT).unwrap_or_revert();
                named_keys.insert(deposits::DEPOSITS_DICT.to_string(), urls_dict.into());

                // Add empty purse.
                let purse: URef = system::create_purse();
                named_keys.insert(deposits::PURSE.to_string(), purse.into());

                // Set initial deposit amount to 2 CSPR.
                let deposit_amount = U512::from(2_000_000_000);
                named_keys.insert(deposits::DEPOSIT_AMOUNT.to_string(), storage::new_uref(deposit_amount).into());

                contract_package_hash
            }
        };
    let entry_points = get_entry_points();
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);
    
    runtime::put_key(
        &format!("{}-latest-version-contract", name),
        contract_hash.into()
    );

    runtime::put_key(
        &format!("{}-latest-version-contract-hash", name),
        storage::new_uref(contract_hash).into()
    );
}

// Entry points

/// Stores the `url` parameter to the contract callers PublicKey.
/// Needs to start with `http://` or `https://`.
#[no_mangle]
fn set_url() {
    let caller = utils::get_caller();
    let url: String = runtime::get_named_arg("url");
    let purse: Option<URef> = runtime::get_named_arg("purse");
    Deposits::new().deposit_if_needed(&caller, purse);
    Urls::new().set(&caller, &url);
}

#[no_mangle]
fn call_set_url() {
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

/// Getter function for stored URLs. Returns data stored under the `account` argument.
#[no_mangle]
fn get_url() {
    let account = runtime::get_named_arg::<AccountHash>("account");
    let url = Urls::new().get(&account);
    runtime::ret(CLValue::from_t(url).unwrap_or_revert());
}

/// Function so the caller can remove their stored URL from the contract.
#[no_mangle]
fn delete_url() {
    let caller = utils::get_caller();
    Urls::new().delete(&caller);
    Deposits::new().withdraw(&caller);
}

/// Administrator function that can create new or overwrite already existing urls stored under `PublicKey`es.
/// Can still only store URLs.
#[no_mangle]
fn set_url_for_account() {
    Admins::new().assert_caller_is_admin();

    let url: String = runtime::get_named_arg("url");
    let account = runtime::get_named_arg("account");
    Urls::new().set(&account, &url);
}

/// Administrator function to remove stored data from the contract.
#[no_mangle]
fn delete_url_for_account() {
    Admins::new().assert_caller_is_admin();

    let account = runtime::get_named_arg("account");
    Urls::new().delete(&account);
    Deposits::new().withdraw(&account);
}

/// Administrator function to add another administrators.
#[no_mangle]
fn add_admin() {
    let admins = Admins::new();
    admins.assert_caller_is_admin();

    let account = runtime::get_named_arg("account");
    admins.add(&account);
}

/// Administrator function to add another administrators.
#[no_mangle]
fn disable_admin() {
    let admins = Admins::new();
    admins.assert_caller_is_admin();
    let account = runtime::get_named_arg("account");
    admins.disable(&account);
}

#[no_mangle]
fn set_deposit_amount() {
    Admins::new().assert_caller_is_admin();
    let amount = runtime::get_named_arg("amount");
    Deposits::new().set_deposit_amount(amount)
}