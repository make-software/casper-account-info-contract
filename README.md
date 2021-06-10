# Casper Validator Info Contract

## Install
Make sure `wasm32-unknown-unknown` is installed.
```bash
$ make prepare
```

## Build Smart Contract
```bash
$ make build-contract
```

## Test
Test logic and smart contract.
```bash
$ make test
```

## Deploy

See Casper documentation: [Deploying Contracts](https://docs.casperlabs.io/en/latest/dapp-dev-guide/deploying-contracts.html) and [Contracts on the Blockchain](https://docs.casperlabs.io/en/latest/dapp-dev-guide/calling-contracts.html).

## Contract entrypoints

- set_url:
    - Arguments:
        - `url` - String
    - Description: Sets a new storage key in the contract. The key name is the callers `AccountHash` hexed, the value is the argument `url`.

- get_url:
    - Arguments:
        - `public_key` - PublicKey
    - Description: Getter for a stored URL. Argument `public_key` is the `PublicKey` that the URL belongs to, and is stored under.

- delete_url:
    - Arguments: None
    - Description: Function that allows the caller to remove the URL that is stored under their `AccountHash`.

- set_url_for_validator:
    - Arguments:
        - `public_key` - PublicKey
        - `url` - String
    - Description: Administrator function. Same function as `set_url` but can overwrite data set by others.
    The key is an `AccountHash` derived from the `PublicKey` argument. 

- delete_url_for_validator:
    - Arguments:
        - `public_key` - PublicKey
    - Description: Administrator function. Same function as `delete_url` but can delete data set by others.
    The key is an `AccountHash` derived from the `PublicKey` argument. 

