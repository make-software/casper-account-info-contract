extern crate alloc;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, contracts::NamedKeys, CLTyped};

use types::{
    contracts::ContractPackageHash, ApiError, CLType, CLValue, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter,
};

mod admins;
mod urls;
mod utils;

use admins::Admins;
use urls::Urls;

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
    ReadingCallerError = 9,
}

impl From<ContractError> for ApiError {
    fn from(err: ContractError) -> ApiError {
        ApiError::User(err as u16)
    }
}

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
        "burn_one_cspr",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "set_cspr_to_burn",
        vec![Parameter::new("cspr_to_burn", CLType::U32)],
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
                runtime::put_key(&format!("{}-package", name), contract_package_hash.into());
                runtime::put_key(
                    &format!("{}-package-access-uref", name),
                    access_token.into(),
                );
                runtime::put_key(
                    &format!("{}-package-hash", name),
                    storage::new_uref(contract_package_hash).into(),
                );

                // Add deployer as the first admin.
                let admin = utils::self_addr();
                let admins_dict = storage::new_dictionary(admins::ADMINS_DICT).unwrap_or_revert();
                storage::dictionary_put(admins_dict, &admin.to_string(), admins::ADMIN_ACTIVE);
                named_keys.insert(admins::ADMINS_DICT.to_string(), admins_dict.into());
                named_keys.insert(
                    admins::ADMINS_COUNT.to_string(),
                    storage::new_uref(1u32).into(),
                );

                // Add empty dictionary for urls.
                let urls_dict = storage::new_dictionary(urls::URLS_DICT).unwrap_or_revert();
                named_keys.insert(urls::URLS_DICT.to_string(), urls_dict.into());

                // Set initial gas_burn to 10 CSPR.
                named_keys.insert("cspr_to_burn".to_string(), storage::new_uref(10u32).into());

                // Store package hash.
                named_keys.insert(
                    "package_hash".to_string(),
                    storage::new_uref(contract_package_hash).into(),
                );

                contract_package_hash
            }
        };

    let entry_points = get_entry_points();
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);

    runtime::put_key(
        &format!("{}-latest-version-contract", name),
        contract_hash.into(),
    );

    runtime::put_key(
        &format!("{}-latest-version-contract-hash", name),
        storage::new_uref(contract_hash).into(),
    );
}

// Entry points

/// Stores the `url` parameter to the contract callers PublicKey.
/// Needs to start with `http://` or `https://`.
#[no_mangle]
fn set_url() {
    let caller = utils::get_caller();
    let url: String = runtime::get_named_arg("url");
    let urls = Urls::new();
    let first_time = urls.get(&caller).is_none();

    // Burn CSPR if never done that before.
    if first_time {
        let package_hash: ContractPackageHash = utils::get_key("package_hash").unwrap_or_revert();
        let cspr_to_burn: u32 = utils::get_key("cspr_to_burn").unwrap_or_revert();
        for _ in 0..cspr_to_burn {
            let _: Vec<u8> = runtime::call_versioned_contract(
                package_hash,
                None,
                "burn_one_cspr",
                Default::default(),
            );
        }
    }

    Urls::new().set(&caller, &url);
}

/// Getter function for stored URLs. Returns data stored under the `account` argument.
#[no_mangle]
fn get_url() {
    let account = runtime::get_named_arg::<AccountHash>("account");
    let url = Urls::new()
        .get(&account)
        .unwrap_or_revert_with(ContractError::NotFound);
    runtime::ret(CLValue::from_t(url).unwrap_or_revert());
}

/// Function so the caller can remove their stored URL from the contract.
#[no_mangle]
fn delete_url() {
    let caller = utils::get_caller();
    Urls::new().delete(&caller);
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

/// Adminstrator function to change amount of CSPR to burn when
/// calling set_url.
#[no_mangle]
fn set_cspr_to_burn() {
    Admins::new().assert_caller_is_admin();
    let cspr_to_burn: u32 = runtime::get_named_arg("cspr_to_burn");
    utils::set_key("cspr_to_burn", cspr_to_burn);
}

/// Burn tokens.
#[no_mangle]
fn burn_one_cspr() {
    let bytes = Vec::from([255u8; 1575]);
    runtime::ret(CLValue::from_t(bytes).unwrap_or_revert());
}
