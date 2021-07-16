# Casper Account Info Contract

Casper Account Info Contract allows account owners to provide information about themselves to the public by specifying a URL to a [Casper Account Info Standard](https://github.com/make-software/casper-account-info-standard) file.   

## Setup

```bash
make prepare
```

## Build Smart Contract

```bash
make build-contract
```

The WASM file will be available in the target directory:
```bash
target/wasm32-unknown-unknown/release/account-info.wasm
```

## Test

```bash
make test
```

## Deploy

See Casper documentation about [Deploying Contracts](https://docs.casperlabs.io/en/latest/dapp-dev-guide/deploying-contracts.html) and [Contracts on the Blockchain](https://docs.casperlabs.io/en/latest/dapp-dev-guide/calling-contracts.html).

After it's deployed, the account that deployed the contract is assigned as an admin.

## Contract entry points

### get_url

Returns a URL to the account information standard file for the given public key

Arguments:

Name | Type | Description
---- | ---- | -----------
```public_key``` | ```PublicKey``` | The public key of the account, which information standard file URL is requested

### set_url

Sets a URL to the account information standard file for the contract caller 

Arguments:

Name | Type | Description
---- | ---- | -----------
```url``` | ```String``` | A URL to the account information standard file

### delete_url

Deletes contract caller's URL to the account information standard file

Arguments: this entry point has no arguments 

### set_url_for_account

Sets a URL to the account information standard file for the provided account. This entry point is available only to the accounts defined as admins.

Arguments:

Name | Type | Description
---- | ---- | -----------
```url``` | ```String``` | A URL to the account information standard file
```public_key``` | ```PublicKey``` | The public key of the account, the information standard file URL should be set for

### delete_url_for_account

Deletes the account information standard file URL from the provided account. This entry point is available only to the accounts defined as admins.

Arguments: 

Name | Type | Description
---- | ---- | -----------
```public_key``` | ```PublicKey``` | The public key of the account, the URL should be deleted from

### add_admin

Add another admin account.
If fails if the account already exists.
This entry point is available only to the accounts defined as admins.

Arguments: 

Name | Type | Description
---- | ---- | -----------
```public_key``` | ```PublicKey``` | The public key of the new admin account.

### remove_admin

Remove existing admin account.
If fails if the account is not an admin.
If fails if there's only one admin account.
This entry point is available only to the accounts defined as admins.

Arguments: 

Name | Type | Description
---- | ---- | -----------
```public_key``` | ```PublicKey``` | The public key of the existing admin account.
