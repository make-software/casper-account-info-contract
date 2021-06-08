#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, SessionBuilder, TestContextBuilder};
    use casper_types::{
        account::AccountHash, runtime_args, PublicKey, RuntimeArgs, SecretKey, U512,
    };

    #[test]
    fn should_set_url() {
        // Prepare the account.
        let public_key: PublicKey = SecretKey::ed25519_from_bytes([7u8; 32]).unwrap().into();
        let account_addr = AccountHash::from(&public_key);

        let mut context = TestContextBuilder::new()
            .with_public_key(public_key, U512::from(500_000_000_000_000_000u64))
            .build();

        // Deploy the main contract.
        let session_code = Code::from("validator-info.wasm");
        let session = SessionBuilder::new(session_code, RuntimeArgs::new())
            .with_address(account_addr)
            .with_authorization_keys(&[account_addr])
            .build();
        context.run(session);

        // Call the manager contract to create a new contract.
        let session_code = Code::NamedKey(String::from("validator-info"), String::from("set_url"));
        let session_args = runtime_args! {
            "url" => String::from("http://localhost:80")
        };
        let session = SessionBuilder::new(session_code, session_args)
            .with_address(account_addr)
            .with_authorization_keys(&[account_addr])
            .build();
        context.run(session);

        // Read value from the network.
        let url: String = context
            .query(
                account_addr,
                &[String::from("validator-info"), account_addr.to_string()],
            )
            .unwrap()
            .into_t()
            .unwrap();

        // Expect the url is set.
        assert_eq!(url, String::from("http://localhost:80"));
    }
}

fn main() {
    panic!("The main should not be used here");
}
