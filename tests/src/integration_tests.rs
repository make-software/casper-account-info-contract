#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, PublicKey, RuntimeArgs,
        SecretKey, U512,
    };

    pub struct AccountInfoContract {
        pub context: TestContext,
        pub contract_hash: Hash,
        pub admin: AccountHash,
        pub admin_pk: PublicKey,
        pub admin_url: String,
        pub user: AccountHash,
        pub user_pk: PublicKey,
        pub user_url: String,
    }

    impl AccountInfoContract {
        pub fn deploy() -> Self {
            // Create admin.
            let admin_key: PublicKey = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap().into();
            let admin_addr = AccountHash::from(&admin_key);

            // Create plain user.
            let user_key: PublicKey = SecretKey::ed25519_from_bytes([2u8; 32]).unwrap().into();
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
                .query(admin_addr, &["account-info-wrapped".to_string()])
                .unwrap_or_else(|_| panic!("account-info contract not found"))
                .into_t()
                .unwrap_or_else(|_| panic!("account-info has wrong type"));

            Self {
                context,
                contract_hash,
                admin: admin_addr,
                admin_pk: admin_key,
                admin_url: "https://127.0.0.1:90".to_string(),
                user: user_addr,
                user_pk: user_key,
                user_url: "http://localhost:80".to_string(),
            }
        }

        fn query<T: FromBytes + CLTyped>(&self, key: &str) -> T {
            self.context
                .query(self.admin, &["account-info".to_string(), key.to_string()])
                .unwrap()
                .into_t()
                .unwrap()
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
                    "url" => url
                },
            );
        }

        pub fn delete_url(&mut self, caller: &AccountHash) {
            self.call(caller, "delete_url", runtime_args! {});
        }

        pub fn get_url(&self, account: &AccountHash) -> String {
            self.query(&account.to_string())
        }

        pub fn set_url_for_account(
            &mut self,
            caller: &AccountHash,
            account: &PublicKey,
            url: &str,
        ) {
            self.call(
                caller,
                "set_url_for_account",
                runtime_args! {
                    "url" => url,
                    "public_key" => account.clone(),
                },
            );
        }

        pub fn delete_url_for_account(&mut self, caller: &AccountHash, account: &PublicKey) {
            self.call(
                caller,
                "delete_url_for_account",
                runtime_args! {
                    "public_key" => account.clone(),
                },
            );
        }

        pub fn add_admin(&mut self, caller: &AccountHash, account: &PublicKey) {
            self.call(
                caller,
                "add_admin",
                runtime_args! {
                    "public_key" => account.clone(),
                },
            );
        }

        pub fn remove_admin(&mut self, caller: &AccountHash, account: &PublicKey) {
            self.call(
                caller,
                "remove_admin",
                runtime_args! {
                    "public_key" => account.clone(),
                },
            );
        }

        pub fn admins_count(&self) -> u32 {
            self.query("admins_count")
        }

        pub fn is_admin(&self, account: &AccountHash) -> bool {
            let admin_key = format!("admin-{}", account.to_string());
            self.context
                .query(self.admin, &["account-info".to_string(), admin_key])
                .is_ok()
        }
    }

    #[test]
    fn test_url_set_get() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // Set URL of the user.
        let user = contract.user;
        let url = contract.user_url.clone();
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
    fn test_administrator_set() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // User sets their URL.
        let user = contract.user;
        let user_pk = contract.user_pk.clone();
        let url = contract.user_url.clone();
        contract.set_url(&user, &url);

        // Change the URL of the user to that of the admin
        let admin = contract.admin;
        let new_url = contract.admin_url.clone();
        contract.set_url_for_account(&admin, &user_pk, &new_url);

        // Check if the URL is updated.
        assert_eq!(new_url, contract.get_url(&user));
    }

    #[test]
    #[should_panic(expected = "ValueNotFound")]
    fn test_administrator_delete() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();

        // User sets their URL.
        let user = contract.user;
        let user_pk = contract.user_pk.clone();
        let url = contract.user_url.clone();
        contract.set_url(&user, &url);

        // Change the URL of the user to that of the admin
        let admin = contract.admin;
        contract.delete_url_for_account(&admin, &user_pk);

        // This call will panic as we have deleted the URL belonging to the user and as such there is no data.
        contract.get_url(&user);
    }

    #[test]
    #[should_panic]
    fn test_administrator_set_url_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let user_url = contract.user_url.clone();
        let admin_pk = contract.admin_pk.clone();

        // Should fail, as the user doesn't have admin rights.
        contract.set_url_for_account(&user, &admin_pk, &user_url);
    }

    #[test]
    #[should_panic]
    fn test_administrator_delete_url_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let admin = contract.admin;
        let admin_pk = contract.admin_pk.clone();
        let admin_url = contract.admin_url.clone();

        // Admin sets their URL.
        contract.set_url(&admin, &admin_url);

        // Check if we have stored the URL.
        assert_eq!(admin_url, contract.get_url(&admin));

        // Should fail, as the user doesn't have admin rights.
        contract.delete_url_for_account(&user, &admin_pk);
    }

    #[test]
    fn test_administrator_add_remove_admin() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let user_pk = contract.user_pk.clone();
        let admin = contract.admin;
        let admin_pk = contract.admin_pk.clone();

        // Should have one admin.
        assert_eq!(1, contract.admins_count());
        assert!(contract.is_admin(&admin));
        assert!(!contract.is_admin(&user));

        // Add new admin.
        contract.add_admin(&admin, &user_pk);

        // Should have a new admin.
        assert_eq!(2, contract.admins_count());
        assert!(contract.is_admin(&admin));
        assert!(contract.is_admin(&user));

        // Remove original admin.
        contract.remove_admin(&user, &admin_pk);

        // Should have only new admin.
        assert_eq!(1, contract.admins_count());
        assert!(!contract.is_admin(&admin));
        assert!(contract.is_admin(&user));
    }

    #[test]
    #[should_panic]
    fn test_administrator_remove_last_admin() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let admin = contract.admin;
        let admin_pk = contract.admin_pk.clone();

        // Remove current admin.
        contract.remove_admin(&admin, &admin_pk);
    }

    #[test]
    #[should_panic]
    fn test_administrator_remove_non_existing_admin() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let admin = contract.admin;
        let user_pk = contract.user_pk.clone();

        // Remove current admin.
        contract.remove_admin(&admin, &user_pk);
    }

    #[test]
    #[should_panic]
    fn test_administrator_add_admin_twice() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let admin = contract.admin;
        let admin_pk = contract.admin_pk.clone();

        // Add admin again.
        contract.add_admin(&admin, &admin_pk);
    }

    #[test]
    #[should_panic]
    fn test_administrator_add_admin_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let user_pk = contract.user_pk.clone();

        // Should fail, as the user doesn't have admin rights.
        contract.add_admin(&user, &user_pk);
    }

    #[test]
    #[should_panic]
    fn test_administrator_remove_admin_security() {
        // Deploy contract.
        let mut contract = AccountInfoContract::deploy();
        let user = contract.user;
        let user_pk = contract.user_pk.clone();

        // Should fail, as the user doesn't have admin rights.
        contract.add_admin(&user, &user_pk);
    }
}

fn main() {
    panic!("The main should not be used here");
}
