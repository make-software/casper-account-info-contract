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

- set_url
- get_url
- delete_url
- set_url_for_validator
- delete_url_for_validator
