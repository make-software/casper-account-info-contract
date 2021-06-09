#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, PublicKey, RuntimeArgs,
        SecretKey, U512,
    };

    pub struct ValidatorContract {
        pub context: TestContext,
        pub contract_hash: Hash,
        pub admin: AccountHash,
        pub admin_url: String,
        pub user: AccountHash,
        pub user_url: String,
    }

    impl ValidatorContract {
        pub fn deploy() -> Self {
            // Create admin.
            let admin_key: PublicKey = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap().into();
            let admin_addr = AccountHash::from(&admin_key);

            // Create plain user.
            let user_key: PublicKey = SecretKey::ed25519_from_bytes([2u8; 32]).unwrap().into();
            let user_addr = AccountHash::from(&user_key);

            // Create context.
            let mut context = TestContextBuilder::new()
                .with_public_key(admin_key, U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_key, U512::from(500_000_000_000_000_000u64))
                .build();

            // Deploy the main contract onto the context.
            let session_code = Code::from("validator-info.wasm");
            let session = SessionBuilder::new(session_code, RuntimeArgs::new())
                .with_address(admin_addr)
                .with_authorization_keys(&[admin_addr])
                .build();
            context.run(session);

            let contract_hash = context
                .query(admin_addr, &["validator-info-wrapped".to_string()])
                .unwrap_or_else(|_| panic!("validator-info contract not found"))
                .into_t()
                .unwrap_or_else(|_| panic!("validator-info has wrong type"));

            Self {
                context,
                contract_hash,
                admin: admin_addr,
                admin_url: "https://127.0.0.1:90".to_string(),
                user: user_addr,
                user_url: "http://localhost:80".to_string(),
            }
        }

        pub fn query<T: FromBytes + CLTyped>(&self, key: &str) -> T {
            self.context
                .query(self.admin, &["validator-info".to_string(), key.to_string()])
                .unwrap()
                .into_t()
                .unwrap()
        }

        pub fn call(&mut self, caller: &AccountHash, function: &str, args: RuntimeArgs) {
            let session_code = Code::Hash(self.contract_hash, function.to_string());
            let session = SessionBuilder::new(session_code, args)
                .with_address(*caller)
                .with_authorization_keys(&[*caller])
                .build();
            self.context.run(session);
        }
    }

    #[test]
    fn test_url_set_get() {
        // Deploy contract.
        let mut contract = ValidatorContract::deploy();

        // Set URL of the user.
        let user = contract.user;
        let set_args = runtime_args! {
            "url" => contract.user_url.clone()
        };
        contract.call(&user, "set_url", set_args);

        // Read URL of user from the context.
        let user_url: String = contract.query(&user.to_string());

        // Check if we have stored the URL.
        assert_eq!(user_url, contract.user_url);
    }

    #[test]
    #[should_panic(expected = "ValueNotFound")]
    fn test_delete() {
        // Deploy contract.
        let mut contract = ValidatorContract::deploy();
        // Set URL of the user.
        let user = contract.user;
        let set_args = runtime_args! {
            "url" => contract.user_url.clone()
        };
        contract.call(&user, "set_url", set_args);
        // Read URL of user from the context.
        let user_url: String = contract.query(&contract.user.to_string());

        // Check if we have stored the URL.
        assert_eq!(user_url, contract.user_url);

        // Delete URL of the user
        contract.call(&user, "delete_url", runtime_args! {});

        // This call will panic as we have deleted the URL belonging to the user and as such there is no data.
        contract.query::<String>(&user.to_string());
    }

    #[test]
    fn test_administrator_set() {
        // Deploy contract.
        let mut contract = ValidatorContract::deploy();
        // User sets their URL.
        let user = contract.user;
        let set_args = runtime_args! {
            "url" => contract.user_url.clone()
        };

        contract.call(&user, "set_url", set_args);

        // Read URL of user from the context.
        let user_url: String = contract.query(&user.to_string());

        // Check if we have stored the URL.
        assert_eq!(user_url, contract.user_url);

        // Delete URL of the user
        let admin = contract.admin;
        let admin_set_args = runtime_args! {
            "url" => contract.admin_url.clone(),
            "account_hash" => user.to_string(),
        };
        contract.call(&admin, "set_url_for_validator", admin_set_args);

        // This call will panic as we have deleted the URL belonging to the user and as such there is no data.
        let overwritten_url = contract.query::<String>(&user.to_string());
        assert_eq!(overwritten_url, contract.admin_url);
    }

    #[test]
    #[should_panic(expected = "InvalidContext")]
    fn test_administrator_security() {
        // Deploy contract.
        let mut contract = ValidatorContract::deploy();
        let user = contract.user;
        let admin = contract.admin;

        // Admin sets their URL.
        let set_args = runtime_args! {
            "url" => contract.admin_url.clone()
        };
        contract.call(&admin, "set_url", set_args);

        // Read URL of admin from the context.
        let admin_url: String = contract.query(&admin.to_string());

        // Check if we have stored the URL.
        assert_eq!(admin_url, contract.admin_url);

        // Delete URL of the admin, via administrator delete_url function.
        let admin_delete_args = runtime_args! {
            "account_hash" => admin.to_string(),
        };

        // This line should fail as user should not have access to this function.
        contract.call(&user, "delete_url_for_validator", admin_delete_args);
    }
}

fn main() {
    panic!("The main should not be used here");
}
