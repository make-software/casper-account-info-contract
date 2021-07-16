extern crate alloc;

use contract::{
    contract_api::{runtime, runtime::revert, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use std::convert::TryInto;
use types::{account::AccountHash, contracts::NamedKeys, PublicKey};

use types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractPackageHash,
    ApiError, CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Parameter,
};

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
                let admin = admin_account_hash_to_string(&runtime::get_caller());
                let admin_flag = storage::new_uref(ADMIN_FLAG).into();
                named_keys.insert(admin, admin_flag);
                named_keys.insert("admins_count".to_string(), storage::new_uref(1u32).into());

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
    assert_admin_rights();

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
    assert_admin_rights();

    let public_key = runtime::get_named_arg::<PublicKey>("public_key");
    runtime::remove_key(&pubkey_to_string(&public_key));
}

/// Administrator function to add another administrators.
#[no_mangle]
fn add_admin() {
    assert_admin_rights();

    // Fail if the admin is already registered.
    let admin = runtime::get_named_arg::<PublicKey>("public_key");
    if is_admin(&admin) {
        revert(ContractError::AdminExists);
    };

    // Set admin.
    set_key(&admin_pubkey_to_string(&admin), ADMIN_FLAG);

    // Increment admins count.
    let admins_count: u32 = get_key("admins_count");
    set_key("admins_count", admins_count + 1);
}

/// Administrator function to add another administrators.
#[no_mangle]
fn remove_admin() {
    assert_admin_rights();

    // Fail if admin doesn't exists.
    let admin = runtime::get_named_arg::<PublicKey>("public_key");
    if !is_admin(&admin) {
        revert(ContractError::AdminDoesntExist);
    };

    // Make sure the last admin can't remove itself.
    let admins_count: u32 = get_key("admins_count");
    if admins_count == 1 {
        revert(ContractError::AdminCountToLow);
    }

    // Decrement admins count.
    set_key("admins_count", admins_count - 1);

    // Remove admin from the admins list.
    runtime::remove_key(&admin_pubkey_to_string(&admin));
}

// Utility functions

/// Check if given account is an admin.
fn is_admin(account: &PublicKey) -> bool {
    runtime::has_key(&admin_pubkey_to_string(&account))
}

/// Check if the caller has admin rights.
/// Revert otherwise.
fn assert_admin_rights() {
    let admin = runtime::get_caller();
    let has_key = runtime::has_key(&admin_account_hash_to_string(&admin));
    if !has_key {
        revert(ContractError::PermissionDenied);
    };
}

/// Retrieve admins named key from account hash.
fn admin_account_hash_to_string(account_hash: &AccountHash) -> String {
    format!("admin-{}", account_hash.to_string())
}

/// Retrieve admins named key from public key.
fn admin_pubkey_to_string(pubkey: &PublicKey) -> String {
    admin_account_hash_to_string(&pubkey.to_account_hash())
}

/// Retrieve AccountHash from public key.
fn pubkey_to_string(pubkey: &PublicKey) -> String {
    pubkey.to_account_hash().to_string()
}

/// Getter function from context storage.
/// Returns the previously data previously stored under `name` key,
/// or returns the default value of the type expected at the end of the call.
fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

/// Creates new storage key `name` and stores `value` to it.
/// In case the key `name` already exists, overwrites it with the new data.
fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}
