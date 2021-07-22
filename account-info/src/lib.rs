extern crate alloc;

use admins::Admins;
use contract::{
    contract_api::{runtime, runtime::revert, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, contracts::NamedKeys, PublicKey};

use types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractPackageHash,
    ApiError, CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Parameter,
};

mod admins;
mod utils;

use utils::{get_key, set_key};

const ADMIN_FLAG: bool = true;

#[derive(Debug)]
pub enum ContractError {
    NotFound = 1,
    BadUrlFormat = 2,
    NotAllowed = 3,
    AdminCountToLow = 4,
    PermissionDenied = 5,
    AdminExists = 6,
    AdminDoesntExist = 7,
    CallerIsNotAccount = 8
}

impl From<ContractError> for ApiError {
    fn from(err: ContractError) -> ApiError {
        ApiError::User(err as u16)
    }
}

fn check_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
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
        "get_url",
        vec![Parameter::new("public_key", CLType::PublicKey)],
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
            Parameter::new("public_key", CLType::PublicKey),
            Parameter::new("url", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "delete_url_for_account",
        vec![Parameter::new("public_key", CLType::PublicKey)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "add_admin",
        vec![Parameter::new("public_key", CLType::PublicKey)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "remove_admin",
        vec![Parameter::new("public_key", CLType::PublicKey)],
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
        match runtime::get_key(&format!("{}-package-hash", name)) {
            Some(contract_package_hash) => {
                contract_package_hash.into_hash().unwrap_or_revert().into()
            }
            None => {
                let (contract_package_hash, access_token) =
                    storage::create_contract_package_at_hash();
                runtime::put_key(
                    &format!("{}-package-hash", name),
                    contract_package_hash.into(),
                );
                runtime::put_key(&format!("{}-access-uref", name), access_token.into());

                // Add deployer as the first admin.
                let admin = utils::get_caller().unwrap_or_revert();
                let dictionary_uref = storage::new_dictionary(admins::ADMINS_DICT).unwrap_or_revert();
                storage::dictionary_put(
                    dictionary_uref, 
                    &admin.to_string(), 
                    admins::ADMIN_ACTIVE
                );
                named_keys.insert(admins::ADMINS_DICT.to_string(), dictionary_uref.into());

                named_keys.insert(admins::ADMINS_COUNT.to_string(), storage::new_uref(1u32).into());

                contract_package_hash
            }
        };
    let entry_points = get_entry_points();
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);

    runtime::put_key(&name, contract_hash.into());
    runtime::put_key(
        &format!("{}-wrapped", name),
        storage::new_uref(contract_hash).into(),
    );
}

// Entry points

/// Stores the `url` parameter to the contract callers PublicKey.
/// Needs to start with `http://` or `https://`.
#[no_mangle]
fn set_url() {
    let url: String = runtime::get_named_arg("url");

    if !check_url(&url) {
        revert(ContractError::BadUrlFormat)
    }
    set_key(&runtime::get_caller().to_string(), url);
}

/// Getter function for stored URLs. Returns data stored under the `public_key` argument.
#[no_mangle]
fn get_url() {
    let public_key = runtime::get_named_arg::<PublicKey>("public_key");
    let url = get_key::<String>(&pubkey_to_string(&public_key));
    runtime::ret(CLValue::from_t(url).unwrap_or_revert());
}

/// Function so the caller can remove their stored URL from the contract.
#[no_mangle]
fn delete_url() {
    runtime::remove_key(&runtime::get_caller().to_string());
}

/// Administrator function that can create new or overwrite already existing urls stored under `PublicKey`es.
/// Can still only store URLs.
#[no_mangle]
fn set_url_for_account() {
    Admins::new().assert_caller_is_admin();

    let url: String = runtime::get_named_arg("url");
    let public_key = runtime::get_named_arg::<PublicKey>("public_key");
    if !check_url(&url) {
        revert(ContractError::BadUrlFormat)
    }
    set_key(&pubkey_to_string(&public_key), url);
}

/// Administrator function to remove stored data from the contract.
#[no_mangle]
fn delete_url_for_account() {
    Admins::new().assert_caller_is_admin();

    let public_key = runtime::get_named_arg::<PublicKey>("public_key");
    runtime::remove_key(&pubkey_to_string(&public_key));
}

/// Administrator function to add another administrators.
#[no_mangle]
fn add_admin() {
    let admins = Admins::new();
    admins.assert_caller_is_admin();

    // Fail if the admin is already registered.
    let address = runtime::get_named_arg::<PublicKey>("public_key");
    admins.add(&address.to_account_hash());
}

/// Administrator function to add another administrators.
#[no_mangle]
fn remove_admin() {
    let admins = Admins::new();
    admins.assert_caller_is_admin();

    let account = runtime::get_named_arg::<PublicKey>("public_key");
    admins.disable(&account.to_account_hash());
}

/// Retrieve AccountHash from public key.
fn pubkey_to_string(pubkey: &PublicKey) -> String {
    pubkey.to_account_hash().to_string()
}

