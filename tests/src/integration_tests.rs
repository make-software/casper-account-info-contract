#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, Key, PublicKey,
        RuntimeArgs, SecretKey, URef, U512,
    };

    pub struct AccountInfoContract {
        pub context: TestContext,
        pub contract_hash: Hash,
        pub contract_package_hash: Hash,
        pub admin: AccountHash,
        pub admin_pk: PublicKey,
        pub admin_url: String,
        pub user: AccountHash,
        pub user_pk: PublicKey,
        pub user_url: String,
        pub deposit_amount: U512,
    }

    impl AccountInfoContract {
        pub fn deploy() -> Self {
            // Create admin.
            let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
            let admin_key: PublicKey = (&admin_secret).into();
            let admin_addr = AccountHash::from(&admin_key);

            // Create plain user.
            let user_secret = SecretKey::ed25519_from_bytes([2u8; 32]).unwrap();
            let user_key: PublicKey = (&user_secret).into();
            let user_addr = AccountHash::from(&user_key);

            // Create context.
            let mut context = TestContextBuilder::new()
                .with_public_key(admin_key.clone(), U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_key.clone(), U512::from(500_000_000_000_000_000u64))
                .build();

            // Deploy the main contract onto the context.
            let session_code = Code::from("account-info.wasm");
            let session = SessionBuilder::new(session_code, RuntimeArgs::new())
                .with_address(admin_addr)
                .with_authorization_keys(&[admin_addr])
                .build();
            context.run(session);

            let contract_hash = context
                .query(
                    admin_addr,
                    &["account-info-latest-version-contract-hash".to_string()],
                )
                .unwrap()
                .into_t()
                .unwrap();

            let contract_package_hash = context
                .query(admin_addr, &["account-info-package-hash".to_string()])
                .unwrap()
                .into_t()
                .unwrap();

            Self {
                context,
                contract_hash,
                contract_package_hash,
                admin: admin_addr,
                admin_pk: admin_key,
                admin_url: "https://127.0.0.1:90".to_string(),
                user: user_addr,
                user_pk: user_key,
                user_url: "http://localhost:80".to_string(),
                deposit_amount: U512::from(2_000_000_000),
            }
        }

        fn query<T: FromBytes + CLTyped>(&self, key: &str) -> T {
            println!("{:?}", key);
            self.context
                .query(
                    self.admin,
                    &[
                        "account-info-latest-version-contract".to_string(),
                        key.to_string(),
                    ],
                )
                .unwrap()
                .into_t()
                .unwrap()
        }

        fn query_dictionary_value<T: CLTyped + FromBytes>(
            &self,
            dict_name: &str,
            key: &str,
        ) -> Option<T> {
            match self.context.query_dictionary_item(
                Key::Hash(self.contract_hash),
                Some(dict_name.to_string()),
                key.to_string(),
            ) {
                Err(_) => None,
                Ok(maybe_value) => {
                    let value = maybe_value
                        .into_t()
                        .unwrap_or_else(|_| panic!("is not expected type."));
                    Some(value)
                }
            }
        }

        fn call(&mut self, caller: &AccountHash, function: &str, args: RuntimeArgs) {
            let session_code = Code::Hash(self.contract_hash, function.to_string());
            let session = SessionBuilder::new(session_code, args)
                .with_address(*caller)
                .with_authorization_keys(&[*caller])
                .build();
            self.context.run(session);
        }

        pub fn set_url(&mut self, caller: &AccountHash, url: &str) {
            self.call(
                caller,
                "set_url",
                runtime_args! {
                    "url" => url,
                    "purse" => Option::<URef>::None
                },
            );
        }

        pub fn delete_url(&mut self, caller: &AccountHash) {
            self.call(caller, "delete_url", runtime_args! {});
        }

        pub fn get_url(&self, account: &AccountHash) -> String {
            let url: String = self
                .query_dictionary_value("urls", &account.to_string())
                .unwrap();
            if url.is_empty() {
                panic!("ValueNotFound");
            } else {
                url
            }
        }

        pub fn set_url_for_account(
            &mut self,
            caller: &AccountHash,
            account: &AccountHash,
            url: &str,
        ) {
            self.call(
                caller,
                "set_url_for_account",
                runtime_args! {
                    "url" => url,
                    "account" => *account,
                },
            );
        }

        pub fn delete_url_for_account(&mut self, caller: &AccountHash, account: &AccountHash) {
            self.call(
                caller,
                "delete_url_for_account",
                runtime_args! {
                    "account" => *account,
                },
            );
        }

        pub fn add_admin(&mut self, caller: &AccountHash, account: &AccountHash) {
            self.call(
                caller,
                "add_admin",
                runtime_args! {
                    "account" => *account,
                },
            );
        }

        pub fn disable_admin(&mut self, caller: &AccountHash, account: &AccountHash) {
            self.call(
                caller,
                "disable_admin",
                runtime_args! {
                    "account" => *account,
                },
            );
        }

        pub fn admins_count(&self) -> u32 {
            self.query("admins_count")
        }

        pub fn is_admin(&self, account: &AccountHash) -> bool {
            let value: Option<bool> = self.query_dictionary_value("admins", &account.to_string());
            value.unwrap_or(false)
        }
    }

    #[test]
    fn test_set_url() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // Set URL of the user.
        let user = contract.user;
        let url = contract.user_url.clone();
        // contract.set_url_via_hash(&user, &ur);
        contract.set_url(&user, &url);

        // Check if we have stored the URL.
        assert_eq!(url, contract.get_url(&user));

        // Override the URL.
        let new_url = String::from("http://test.com");
        contract.set_url(&user, &new_url);

        // Check if the URL is updated.
        assert_eq!(new_url, contract.get_url(&user));
    }

    #[test]
    #[should_panic(expected = "ValueNotFound")]
    fn test_delete() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // Set URL of the user.
        let user = contract.user;
        let url = contract.user_url.clone();
        contract.set_url(&user, &url);

        // Delete URL of the user
        contract.delete_url(&user);

        // This call will panic as we have deleted the URL belonging to the user and as such there is no data.
        contract.get_url(&user);
    }

    #[test]
    fn test_set_url_for_account() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // User sets their URL.
        let user = contract.user;
        let url = contract.user_url.clone();
        contract.set_url(&user, &url);

        // Change the URL of the user to that of the admin
        let admin = contract.admin;
        let new_url = contract.admin_url.clone();
        contract.set_url_for_account(&admin, &user, &new_url);

        // Check if the URL is updated.
        assert_eq!(new_url, contract.get_url(&user));
    }

    #[test]
    #[should_panic(expected = "ValueNotFound")]
    fn test_delete_url_for_account() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // User sets their URL.
        let user = contract.user;
        let url = contract.user_url.clone();
        contract.set_url(&user, &url);

        // Change the URL of the user to that of the admin
        let admin = contract.admin;
        contract.delete_url_for_account(&admin, &user);

        // This call will panic as we have deleted the URL belonging to the user and as such there is no data.
        contract.get_url(&user);
    }

    #[test]
    #[should_panic]
    fn test_set_url_for_account_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let user_url = contract.user_url.clone();
        let admin = contract.admin;

        // Should fail, as the user doesn't have admin rights.
        contract.set_url_for_account(&user, &admin, &user_url);
    }

    #[test]
    #[should_panic]
    fn test_delete_url_for_account_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let admin = contract.admin;
        let admin_url = contract.admin_url.clone();

        // Admin sets their URL.
        contract.set_url(&user, &admin_url);

        // Check if we have stored the URL.
        assert_eq!(admin_url, contract.get_url(&admin));

        // Should fail, as the user doesn't have admin rights.
        contract.delete_url_for_account(&user, &admin);
    }

    #[test]
    fn test_add_and_disable_admin() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let admin = contract.admin;

        // Should have one admin.
        assert_eq!(1, contract.admins_count());
        assert!(contract.is_admin(&admin));
        assert!(!contract.is_admin(&user));

        // Add new admin.
        contract.add_admin(&admin, &user);

        // Should have a new admin.
        assert_eq!(2, contract.admins_count());
        assert!(contract.is_admin(&admin));
        assert!(contract.is_admin(&user));

        // Remove original admin.
        contract.disable_admin(&user, &admin);

        // Should have only new admin.
        assert_eq!(1, contract.admins_count());
        assert!(!contract.is_admin(&admin));
        assert!(contract.is_admin(&user));
    }

    #[test]
    #[should_panic]
    fn test_remove_last_admin() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let admin = contract.admin;

        // Remove current admin.
        contract.disable_admin(&admin, &admin);
    }

    #[test]
    #[should_panic]
    fn test_remove_non_existing_admin() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let admin = contract.admin;
        let user = contract.user;

        // Remove current admin.
        contract.disable_admin(&admin, &user);
    }

    #[test]
    #[should_panic]
    fn test_add_admin_twice() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let admin = contract.admin;

        // Add admin again.
        contract.add_admin(&admin, &admin);
    }

    #[test]
    #[should_panic]
    fn test_add_admin_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;

        // Should fail, as the user doesn't have admin rights.
        contract.add_admin(&user, &user);
    }

    #[test]
    #[should_panic]
    fn test_disable_admin_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;

        // Should fail, as the user doesn't have admin rights.
        contract.add_admin(&user, &user);
    }
}

fn main() {
    panic!("The main should not be used here");
}
